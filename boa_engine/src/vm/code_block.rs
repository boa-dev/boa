//! `CodeBlock`
//!
//! This module is for the `CodeBlock` which implements a function representation in the VM

use crate::{
    builtins::{
        async_generator::{AsyncGenerator, AsyncGeneratorState},
        function::{arguments::Arguments, ConstructorKind, Function, FunctionKind, ThisMode},
        generator::{Generator, GeneratorContext, GeneratorState},
        promise::PromiseCapability,
    },
    context::intrinsics::StandardConstructors,
    environments::{BindingLocator, CompileTimeEnvironment, FunctionSlots, ThisBindingStatus},
    error::JsNativeError,
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData, PROTOTYPE},
    property::PropertyDescriptor,
    string::utf16,
    vm::CallFrame,
    Context, JsError, JsResult, JsString, JsValue,
};
use bitflags::bitflags;
use boa_ast::function::FormalParameterList;
use boa_gc::{empty_trace, Finalize, Gc, Trace};
use boa_profiler::Profiler;
use std::{cell::Cell, collections::VecDeque, mem::size_of, rc::Rc};
use thin_vec::ThinVec;

#[cfg(any(feature = "trace", feature = "flowgraph"))]
use crate::vm::Opcode;
#[cfg(any(feature = "trace", feature = "flowgraph"))]
use boa_interner::{Interner, ToInternedString};

/// This represents whether a value can be read from [`CodeBlock`] code.
///
/// # Safety
///
/// This trait is safe to implement as long as the type doesn't implement `Drop`.
/// At some point, if [negative impls][negative_impls] are stabilized, we might be able to remove
/// the unsafe bound.
///
/// [negative_impls]: https://doc.rust-lang.org/beta/unstable-book/language-features/negative-impls.html
pub(crate) unsafe trait Readable {}

unsafe impl Readable for u8 {}
unsafe impl Readable for i8 {}
unsafe impl Readable for u16 {}
unsafe impl Readable for i16 {}
unsafe impl Readable for u32 {}
unsafe impl Readable for i32 {}
unsafe impl Readable for u64 {}
unsafe impl Readable for i64 {}
unsafe impl Readable for f32 {}
unsafe impl Readable for f64 {}

bitflags! {
    /// Flags for [`CodeBlock`].
    #[derive(Clone, Copy, Debug, Finalize)]
    pub(crate) struct CodeBlockFlags: u8 {
        /// Is this function in strict mode.
        const STRICT = 0b0000_0001;

        /// Indicates if the function is an expression and has a binding identifier.
        const HAS_BINDING_IDENTIFIER = 0b0000_0010;

        /// The `[[IsClassConstructor]]` internal slot.
        const IS_CLASS_CONSTRUCTOR = 0b0000_0100;

        /// Does this function have a parameters environment.
        const PARAMETERS_ENV_BINDINGS = 0b0000_1000;

        /// Does this function need a `"arguments"` object.
        ///
        /// The `"arguments"` binding is the first binding.
        const NEEDS_ARGUMENTS_OBJECT = 0b0001_0000;

        /// The `[[ClassFieldInitializerName]]` internal slot.
        const IN_CLASS_FIELD_INITIALIZER = 0b0010_0000;

        /// Trace instruction execution to `stdout`.
        #[cfg(feature = "trace")]
        const TRACEABLE = 0b1000_0000;
    }
}

// SAFETY: Nothing in CodeBlockFlags needs tracing, so this is safe.
unsafe impl Trace for CodeBlockFlags {
    empty_trace!();
}

/// This represents a range in the code that handles exception throws.
///
/// When a throw happens, we search for handler in the [`CodeBlock`] using
/// the [`CodeBlock::find_handler()`] method.
///
/// If any exception happens and gets cought by this handler, the `pc` will be set to `end` of the
/// [`Handler`] and remove any environments or stack values that where pushed after the handler.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Handler {
    pub(crate) start: u32,
    pub(crate) end: u32,

    pub(crate) stack_count: u32,
    pub(crate) environment_count: u32,
}

impl Handler {
    /// Get the handler address.
    pub(crate) const fn handler(&self) -> u32 {
        self.end
    }

    /// Check if the provided `pc` is contained in the handler range.
    pub(crate) const fn contains(&self, pc: u32) -> bool {
        pc < self.end && pc >= self.start
    }
}

/// The internal representation of a JavaScript function.
///
/// A `CodeBlock` is generated for each function compiled by the
/// [`ByteCompiler`](crate::bytecompiler::ByteCompiler). It stores the bytecode and the other
/// attributes of the function.
#[derive(Clone, Debug, Trace, Finalize)]
pub struct CodeBlock {
    /// Name of this function
    #[unsafe_ignore_trace]
    pub(crate) name: JsString,

    #[unsafe_ignore_trace]
    pub(crate) flags: Cell<CodeBlockFlags>,

    /// The number of arguments expected.
    pub(crate) length: u32,

    /// \[\[ThisMode\]\]
    pub(crate) this_mode: ThisMode,

    /// Parameters passed to this function.
    #[unsafe_ignore_trace]
    pub(crate) params: FormalParameterList,

    /// Bytecode
    pub(crate) bytecode: Box<[u8]>,

    /// Literals
    pub(crate) literals: Box<[JsValue]>,

    /// Property field names and private names `[[description]]`s.
    pub(crate) names: Box<[JsString]>,

    /// Locators for all bindings in the codeblock.
    #[unsafe_ignore_trace]
    pub(crate) bindings: Box<[BindingLocator]>,

    /// Functions inside this function
    pub(crate) functions: Box<[Gc<Self>]>,

    /// Exception [`Handler`]s.
    #[unsafe_ignore_trace]
    pub(crate) handlers: ThinVec<Handler>,

    /// Compile time environments in this function.
    ///
    // Safety: Nothing in CompileTimeEnvironment needs tracing, so this is safe.
    //
    // TODO(#3034): Maybe changing this to Gc after garbage collection would be better than Rc.
    #[unsafe_ignore_trace]
    pub(crate) compile_environments: Box<[Rc<CompileTimeEnvironment>]>,
}

/// ---- `CodeBlock` public API ----
impl CodeBlock {
    /// Creates a new `CodeBlock`.
    #[must_use]
    pub fn new(name: JsString, length: u32, strict: bool) -> Self {
        let mut flags = CodeBlockFlags::empty();
        flags.set(CodeBlockFlags::STRICT, strict);
        Self {
            bytecode: Box::default(),
            literals: Box::default(),
            names: Box::default(),
            bindings: Box::default(),
            functions: Box::default(),
            name,
            flags: Cell::new(flags),
            length,
            this_mode: ThisMode::Global,
            params: FormalParameterList::default(),
            handlers: ThinVec::default(),
            compile_environments: Box::default(),
        }
    }

    /// Retrieves the name associated with this code block.
    #[must_use]
    pub const fn name(&self) -> &JsString {
        &self.name
    }

    /// Check if the function is traced.
    #[cfg(feature = "trace")]
    pub(crate) fn traceable(&self) -> bool {
        self.flags.get().contains(CodeBlockFlags::TRACEABLE)
    }
    /// Enable or disable instruction tracing to `stdout`.
    #[cfg(feature = "trace")]
    #[inline]
    pub fn set_traceable(&self, value: bool) {
        let mut flags = self.flags.get();
        flags.set(CodeBlockFlags::TRACEABLE, value);
        self.flags.set(flags);
    }

    /// Check if the function is a class constructor.
    pub(crate) fn is_class_constructor(&self) -> bool {
        self.flags
            .get()
            .contains(CodeBlockFlags::IS_CLASS_CONSTRUCTOR)
    }

    /// Check if the function is in strict mode.
    pub(crate) fn strict(&self) -> bool {
        self.flags.get().contains(CodeBlockFlags::STRICT)
    }

    /// Indicates if the function is an expression and has a binding identifier.
    pub(crate) fn has_binding_identifier(&self) -> bool {
        self.flags
            .get()
            .contains(CodeBlockFlags::HAS_BINDING_IDENTIFIER)
    }

    /// Does this function have a parameters environment.
    pub(crate) fn has_parameters_env_bindings(&self) -> bool {
        self.flags
            .get()
            .contains(CodeBlockFlags::PARAMETERS_ENV_BINDINGS)
    }

    /// Does this function need a `"arguments"` object.
    pub(crate) fn needs_arguments_object(&self) -> bool {
        self.flags
            .get()
            .contains(CodeBlockFlags::NEEDS_ARGUMENTS_OBJECT)
    }

    /// Does this function have the `[[ClassFieldInitializerName]]` internal slot set to non-empty value.
    pub(crate) fn in_class_field_initializer(&self) -> bool {
        self.flags
            .get()
            .contains(CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER)
    }

    /// Find exception [`Handler`] in the code block given the current program counter (`pc`).
    #[inline]
    pub(crate) fn find_handler(&self, pc: u32) -> Option<(usize, &Handler)> {
        self.handlers
            .iter()
            .enumerate()
            .rev()
            .find(|(_, handler)| handler.contains(pc))
    }
}

/// ---- `CodeBlock` private API ----
impl CodeBlock {
    /// Read type T from code.
    ///
    /// # Safety
    ///
    /// Does not check if read happens out-of-bounds.
    pub(crate) unsafe fn read_unchecked<T>(&self, offset: usize) -> T
    where
        T: Readable,
    {
        // Safety:
        // The function caller must ensure that the read is in bounds.
        //
        // This has to be an unaligned read because we can't guarantee that
        // the types are aligned.
        unsafe {
            self.bytecode
                .as_ptr()
                .add(offset)
                .cast::<T>()
                .read_unaligned()
        }
    }

    /// Read type T from code.
    #[track_caller]
    pub(crate) fn read<T>(&self, offset: usize) -> T
    where
        T: Readable,
    {
        assert!(offset + size_of::<T>() - 1 < self.bytecode.len());

        // Safety: We checked that it is not an out-of-bounds read,
        // so this is safe.
        unsafe { self.read_unchecked(offset) }
    }

    /// Get the operands after the `Opcode` pointed to by `pc` as a `String`.
    /// Modifies the `pc` to point to the next instruction.
    ///
    /// Returns an empty `String` if no operands are present.
    #[cfg(any(feature = "trace", feature = "flowgraph"))]
    pub(crate) fn instruction_operands(&self, pc: &mut usize, interner: &Interner) -> String {
        let opcode: Opcode = self.bytecode[*pc].into();
        *pc += size_of::<Opcode>();
        match opcode {
            Opcode::SetFunctionName => {
                let operand = self.read::<u8>(*pc);
                *pc += size_of::<u8>();
                match operand {
                    0 => "prefix: none",
                    1 => "prefix: get",
                    2 => "prefix: set",
                    _ => unreachable!(),
                }
                .to_owned()
            }
            Opcode::RotateLeft | Opcode::RotateRight => {
                let result = self.read::<u8>(*pc).to_string();
                *pc += size_of::<u8>();
                result
            }
            Opcode::PushInt8 => {
                let result = self.read::<i8>(*pc).to_string();
                *pc += size_of::<i8>();
                result
            }
            Opcode::PushInt16 => {
                let result = self.read::<i16>(*pc).to_string();
                *pc += size_of::<i16>();
                result
            }
            Opcode::PushInt32 => {
                let result = self.read::<i32>(*pc).to_string();
                *pc += size_of::<i32>();
                result
            }
            Opcode::PushRational => {
                let operand = self.read::<f64>(*pc);
                *pc += size_of::<f64>();
                ryu_js::Buffer::new().format(operand).to_string()
            }
            Opcode::PushLiteral
            | Opcode::ThrowNewTypeError
            | Opcode::Jump
            | Opcode::JumpIfTrue
            | Opcode::JumpIfFalse
            | Opcode::JumpIfNotUndefined
            | Opcode::JumpIfNullOrUndefined
            | Opcode::Case
            | Opcode::Default
            | Opcode::LogicalAnd
            | Opcode::LogicalOr
            | Opcode::Coalesce
            | Opcode::CallEval
            | Opcode::Call
            | Opcode::New
            | Opcode::SuperCall
            | Opcode::ConcatToString => {
                let result = self.read::<u32>(*pc).to_string();
                *pc += size_of::<u32>();
                result
            }
            Opcode::PushDeclarativeEnvironment | Opcode::PushFunctionEnvironment => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!("{operand}")
            }
            Opcode::CopyDataProperties
            | Opcode::GeneratorDelegateNext
            | Opcode::GeneratorDelegateResume => {
                let operand1 = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                let operand2 = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!("{operand1}, {operand2}")
            }
            Opcode::TemplateLookup | Opcode::TemplateCreate => {
                let operand1 = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                let operand2 = self.read::<u64>(*pc);
                *pc += size_of::<u64>();
                format!("{operand1}, {operand2}")
            }
            Opcode::GetArrowFunction
            | Opcode::GetAsyncArrowFunction
            | Opcode::GetFunction
            | Opcode::GetFunctionAsync => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>() + size_of::<u8>();
                format!(
                    "{operand:04}: '{}' (length: {})",
                    self.functions[operand as usize]
                        .name()
                        .to_std_string_escaped(),
                    self.functions[operand as usize].length
                )
            }
            Opcode::GetGenerator | Opcode::GetGeneratorAsync => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!(
                    "{operand:04}: '{}' (length: {})",
                    self.functions[operand as usize]
                        .name()
                        .to_std_string_escaped(),
                    self.functions[operand as usize].length
                )
            }
            Opcode::DefVar
            | Opcode::DefInitVar
            | Opcode::PutLexicalValue
            | Opcode::GetName
            | Opcode::GetLocator
            | Opcode::GetNameAndLocator
            | Opcode::GetNameOrUndefined
            | Opcode::SetName
            | Opcode::DeleteName => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!(
                    "{:04}: '{}'",
                    operand,
                    interner.resolve_expect(self.bindings[operand as usize].name().sym()),
                )
            }
            Opcode::GetPropertyByName
            | Opcode::GetMethod
            | Opcode::SetPropertyByName
            | Opcode::DefineOwnPropertyByName
            | Opcode::DefineClassStaticMethodByName
            | Opcode::DefineClassMethodByName
            | Opcode::SetPropertyGetterByName
            | Opcode::DefineClassStaticGetterByName
            | Opcode::DefineClassGetterByName
            | Opcode::SetPropertySetterByName
            | Opcode::DefineClassStaticSetterByName
            | Opcode::DefineClassSetterByName
            | Opcode::DeletePropertyByName
            | Opcode::SetPrivateField
            | Opcode::DefinePrivateField
            | Opcode::SetPrivateMethod
            | Opcode::SetPrivateSetter
            | Opcode::SetPrivateGetter
            | Opcode::GetPrivateField
            | Opcode::PushClassFieldPrivate
            | Opcode::PushClassPrivateGetter
            | Opcode::PushClassPrivateSetter
            | Opcode::PushClassPrivateMethod
            | Opcode::InPrivate
            | Opcode::ThrowMutateImmutable => {
                let operand = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                format!(
                    "{operand:04}: '{}'",
                    self.names[operand as usize].to_std_string_escaped(),
                )
            }
            Opcode::PushPrivateEnvironment => {
                let count = self.read::<u32>(*pc);
                *pc += size_of::<u32>() * (count as usize + 1);
                String::new()
            }
            Opcode::JumpTable => {
                let count = self.read::<u32>(*pc);
                *pc += size_of::<u32>();
                let default = self.read::<u32>(*pc);
                *pc += size_of::<u32>();

                let mut operands = format!("#{count}: Default: {default:4}");
                for i in 1..=count {
                    let address = self.read::<u32>(*pc);
                    *pc += size_of::<u32>();

                    operands += &format!(", {i}: {address}");
                }
                operands
            }
            Opcode::JumpIfNotResumeKind => {
                let exit = self.read::<u32>(*pc);
                *pc += size_of::<u32>();

                let resume_kind = self.read::<u8>(*pc);
                *pc += size_of::<u8>();

                format!(
                    "ResumeKind: {:?}, exit: {exit}",
                    JsValue::new(resume_kind).to_generator_resume_kind()
                )
            }
            Opcode::CreateIteratorResult => {
                let done = self.read::<u8>(*pc) != 0;
                *pc += size_of::<u8>();
                format!("done: {done}")
            }
            Opcode::Pop
            | Opcode::Dup
            | Opcode::Swap
            | Opcode::PushZero
            | Opcode::PushOne
            | Opcode::PushNaN
            | Opcode::PushPositiveInfinity
            | Opcode::PushNegativeInfinity
            | Opcode::PushNull
            | Opcode::PushTrue
            | Opcode::PushFalse
            | Opcode::PushUndefined
            | Opcode::PushEmptyObject
            | Opcode::PushClassPrototype
            | Opcode::SetClassPrototype
            | Opcode::SetHomeObject
            | Opcode::SetHomeObjectClass
            | Opcode::Add
            | Opcode::Sub
            | Opcode::Div
            | Opcode::Mul
            | Opcode::Mod
            | Opcode::Pow
            | Opcode::ShiftRight
            | Opcode::ShiftLeft
            | Opcode::UnsignedShiftRight
            | Opcode::BitOr
            | Opcode::BitAnd
            | Opcode::BitXor
            | Opcode::BitNot
            | Opcode::In
            | Opcode::Eq
            | Opcode::StrictEq
            | Opcode::NotEq
            | Opcode::StrictNotEq
            | Opcode::GreaterThan
            | Opcode::GreaterThanOrEq
            | Opcode::LessThan
            | Opcode::LessThanOrEq
            | Opcode::InstanceOf
            | Opcode::TypeOf
            | Opcode::Void
            | Opcode::LogicalNot
            | Opcode::Pos
            | Opcode::Neg
            | Opcode::Inc
            | Opcode::IncPost
            | Opcode::Dec
            | Opcode::DecPost
            | Opcode::GetPropertyByValue
            | Opcode::GetPropertyByValuePush
            | Opcode::SetPropertyByValue
            | Opcode::DefineOwnPropertyByValue
            | Opcode::DefineClassStaticMethodByValue
            | Opcode::DefineClassMethodByValue
            | Opcode::SetPropertyGetterByValue
            | Opcode::DefineClassStaticGetterByValue
            | Opcode::DefineClassGetterByValue
            | Opcode::SetPropertySetterByValue
            | Opcode::DefineClassStaticSetterByValue
            | Opcode::DefineClassSetterByValue
            | Opcode::DeletePropertyByValue
            | Opcode::DeleteSuperThrow
            | Opcode::ToPropertyKey
            | Opcode::ToBoolean
            | Opcode::Throw
            | Opcode::ReThrow
            | Opcode::Exception
            | Opcode::MaybeException
            | Opcode::This
            | Opcode::Super
            | Opcode::Return
            | Opcode::PopEnvironment
            | Opcode::IncrementLoopIteration
            | Opcode::CreateForInIterator
            | Opcode::GetIterator
            | Opcode::GetAsyncIterator
            | Opcode::IteratorNext
            | Opcode::IteratorNextWithoutPop
            | Opcode::IteratorFinishAsyncNext
            | Opcode::IteratorValue
            | Opcode::IteratorValueWithoutPop
            | Opcode::IteratorResult
            | Opcode::IteratorDone
            | Opcode::IteratorToArray
            | Opcode::IteratorPop
            | Opcode::IteratorReturn
            | Opcode::IteratorStackEmpty
            | Opcode::RequireObjectCoercible
            | Opcode::ValueNotNullOrUndefined
            | Opcode::RestParameterInit
            | Opcode::RestParameterPop
            | Opcode::PushValueToArray
            | Opcode::PushElisionToArray
            | Opcode::PushIteratorToArray
            | Opcode::PushNewArray
            | Opcode::GeneratorYield
            | Opcode::AsyncGeneratorYield
            | Opcode::GeneratorNext
            | Opcode::PushClassField
            | Opcode::SuperCallDerived
            | Opcode::Await
            | Opcode::NewTarget
            | Opcode::ImportMeta
            | Opcode::SuperCallPrepare
            | Opcode::CallEvalSpread
            | Opcode::CallSpread
            | Opcode::NewSpread
            | Opcode::SuperCallSpread
            | Opcode::SetPrototype
            | Opcode::PushObjectEnvironment
            | Opcode::IsObject
            | Opcode::SetNameByLocator
            | Opcode::PopPrivateEnvironment
            | Opcode::ImportCall
            | Opcode::GetReturnValue
            | Opcode::SetReturnValue
            | Opcode::Nop => String::new(),
            Opcode::Reserved1
            | Opcode::Reserved2
            | Opcode::Reserved3
            | Opcode::Reserved4
            | Opcode::Reserved5
            | Opcode::Reserved6
            | Opcode::Reserved7
            | Opcode::Reserved8
            | Opcode::Reserved9
            | Opcode::Reserved10
            | Opcode::Reserved11
            | Opcode::Reserved12
            | Opcode::Reserved13
            | Opcode::Reserved14
            | Opcode::Reserved15
            | Opcode::Reserved16
            | Opcode::Reserved17
            | Opcode::Reserved18
            | Opcode::Reserved19
            | Opcode::Reserved20
            | Opcode::Reserved21
            | Opcode::Reserved22
            | Opcode::Reserved23
            | Opcode::Reserved24
            | Opcode::Reserved25
            | Opcode::Reserved26
            | Opcode::Reserved27
            | Opcode::Reserved28
            | Opcode::Reserved29
            | Opcode::Reserved30
            | Opcode::Reserved31
            | Opcode::Reserved32
            | Opcode::Reserved33
            | Opcode::Reserved34
            | Opcode::Reserved35
            | Opcode::Reserved36
            | Opcode::Reserved37
            | Opcode::Reserved38
            | Opcode::Reserved39
            | Opcode::Reserved40
            | Opcode::Reserved41
            | Opcode::Reserved42
            | Opcode::Reserved43
            | Opcode::Reserved44
            | Opcode::Reserved45
            | Opcode::Reserved46
            | Opcode::Reserved47
            | Opcode::Reserved48
            | Opcode::Reserved49
            | Opcode::Reserved50
            | Opcode::Reserved51
            | Opcode::Reserved52
            | Opcode::Reserved53
            | Opcode::Reserved54
            | Opcode::Reserved55
            | Opcode::Reserved56
            | Opcode::Reserved57
            | Opcode::Reserved58
            | Opcode::Reserved59
            | Opcode::Reserved60
            | Opcode::Reserved61
            | Opcode::Reserved62
            | Opcode::Reserved63 => unreachable!("Reserved opcodes are unrechable"),
        }
    }
}

#[cfg(any(feature = "trace", feature = "flowgraph"))]
impl ToInternedString for CodeBlock {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let name = self.name();
        let mut f = if name == "<main>" {
            String::new()
        } else {
            "\n".to_owned()
        };

        f.push_str(&format!(
            "{:-^70}\nLocation  Count    Handler    Opcode                     Operands\n\n",
            format!("Compiled Output: '{}'", name.to_std_string_escaped()),
        ));

        let mut pc = 0;
        let mut count = 0;
        while pc < self.bytecode.len() {
            let instruction_start_pc = pc;

            let opcode: Opcode = self.bytecode[instruction_start_pc].into();
            let opcode = opcode.as_str();
            let previous_pc = pc;
            let operands = self.instruction_operands(&mut pc, interner);

            let handler = if let Some((i, handler)) = self.find_handler(instruction_start_pc as u32)
            {
                let border_char = if instruction_start_pc as u32 == handler.start {
                    '>'
                } else if pc as u32 == handler.end {
                    '<'
                } else {
                    ' '
                };
                format!("{border_char}{i:2}: {:04}", handler.handler())
            } else {
                "   none  ".to_string()
            };

            f.push_str(&format!(
                "{previous_pc:06}    {count:04}   {handler}    {opcode:<27}{operands}\n",
            ));
            count += 1;
        }

        f.push_str("\nLiterals:\n");

        if self.literals.is_empty() {
            f.push_str("    <empty>\n");
        } else {
            for (i, value) in self.literals.iter().enumerate() {
                f.push_str(&format!(
                    "    {i:04}: <{}> {}\n",
                    value.type_of(),
                    value.display()
                ));
            }
        }

        f.push_str("\nBindings:\n");
        if self.bindings.is_empty() {
            f.push_str("    <empty>\n");
        } else {
            for (i, binding_locator) in self.bindings.iter().enumerate() {
                f.push_str(&format!(
                    "    {i:04}: {}\n",
                    interner.resolve_expect(binding_locator.name().sym())
                ));
            }
        }

        f.push_str("\nFunctions:\n");
        if self.functions.is_empty() {
            f.push_str("    <empty>\n");
        } else {
            for (i, code) in self.functions.iter().enumerate() {
                f.push_str(&format!(
                    "    {i:04}: name: '{}' (length: {})\n",
                    code.name().to_std_string_escaped(),
                    code.length
                ));
            }
        }

        f.push_str("\nHandlers:\n");
        if self.handlers.is_empty() {
            f.push_str("    <empty>\n");
        } else {
            for (i, handler) in self.handlers.iter().enumerate() {
                f.push_str(&format!(
                    "    {i:04}: Range: [{:04}, {:04}): Handler: {:04}, Stack: {:02}, Environment: {:02}\n",
                    handler.start,
                    handler.end,
                    handler.handler(),
                    handler.stack_count,
                    handler.environment_count,
                ));
            }
        }

        f
    }
}

/// Creates a new function object.
///
/// This is used in cases that the prototype is not known if it's [`None`] or [`Some`].
///
/// If the prototype given is [`None`] it will use [`create_function_object_fast`]. Otherwise
/// it will construct the function from template objects that have all the fields except the
/// prototype, and will perform a prototype transition change to set the prototype.
///
/// This is slower than direct object template construction that is done in [`create_function_object_fast`].
pub(crate) fn create_function_object(
    code: Gc<CodeBlock>,
    r#async: bool,
    prototype: JsObject,
    context: &mut Context<'_>,
) -> JsObject {
    let _timer = Profiler::global().start_event("create_function_object", "vm");

    let name: JsValue = code.name().clone().into();
    let length: JsValue = code.length.into();

    let script_or_module = context.vm.active_runnable.clone();

    let function = if r#async {
        Function::new(
            FunctionKind::Async {
                code,
                environments: context.vm.environments.clone(),
                home_object: None,
                class_object: None,
                script_or_module,
            },
            context.realm().clone(),
        )
    } else {
        Function::new(
            FunctionKind::Ordinary {
                code,
                environments: context.vm.environments.clone(),
                constructor_kind: ConstructorKind::Base,
                home_object: None,
                fields: ThinVec::new(),
                private_methods: ThinVec::new(),
                class_object: None,
                script_or_module,
            },
            context.realm().clone(),
        )
    };

    let data = ObjectData::function(function, !r#async);

    let templates = context.intrinsics().templates();

    let (mut template, storage, constructor_prototype) = if r#async {
        (
            templates.function_without_proto().clone(),
            vec![length, name],
            None,
        )
    } else {
        let constructor_prototype = templates
            .function_prototype()
            .create(ObjectData::ordinary(), vec![JsValue::undefined()]);

        let template = templates.function_with_prototype_without_proto();

        (
            template.clone(),
            vec![length, name, constructor_prototype.clone().into()],
            Some(constructor_prototype),
        )
    };

    template.set_prototype(prototype);

    let constructor = template.create(data, storage);

    if let Some(constructor_prototype) = &constructor_prototype {
        constructor_prototype.borrow_mut().properties_mut().storage[0] = constructor.clone().into();
    }
    constructor
}

/// Creates a new function object.
///
/// This is prefered over [`create_function_object`] if prototype is [`None`],
/// because it constructs the funtion from a pre-initialized object template,
/// with all the properties and prototype set.
pub(crate) fn create_function_object_fast(
    code: Gc<CodeBlock>,
    r#async: bool,
    arrow: bool,
    method: bool,
    context: &mut Context<'_>,
) -> JsObject {
    let _timer = Profiler::global().start_event("create_function_object_fast", "vm");

    let name: JsValue = code.name().clone().into();
    let length: JsValue = code.length.into();

    let script_or_module = context.vm.active_runnable.clone();

    let function = if r#async {
        FunctionKind::Async {
            code,
            environments: context.vm.environments.clone(),
            home_object: None,
            class_object: None,
            script_or_module,
        }
    } else {
        FunctionKind::Ordinary {
            code,
            environments: context.vm.environments.clone(),
            constructor_kind: ConstructorKind::Base,
            home_object: None,
            fields: ThinVec::new(),
            private_methods: ThinVec::new(),
            class_object: None,
            script_or_module,
        }
    };

    let function = Function::new(function, context.realm().clone());

    let data = ObjectData::function(function, !method && !arrow && !r#async);

    if r#async {
        context
            .intrinsics()
            .templates()
            .async_function()
            .create(data, vec![length, name])
    } else if arrow || method {
        context
            .intrinsics()
            .templates()
            .function()
            .create(data, vec![length, name])
    } else {
        let prototype = context
            .intrinsics()
            .templates()
            .function_prototype()
            .create(ObjectData::ordinary(), vec![JsValue::undefined()]);

        let constructor = context
            .intrinsics()
            .templates()
            .function_with_prototype()
            .create(data, vec![length, name, prototype.clone().into()]);

        prototype.borrow_mut().properties_mut().storage[0] = constructor.clone().into();

        constructor
    }
}

/// Creates a new generator function object.
pub(crate) fn create_generator_function_object(
    code: Gc<CodeBlock>,
    r#async: bool,
    prototype: Option<JsObject>,
    context: &mut Context<'_>,
) -> JsObject {
    let function_prototype = if let Some(prototype) = prototype {
        prototype
    } else if r#async {
        context
            .intrinsics()
            .constructors()
            .async_generator_function()
            .prototype()
    } else {
        context
            .intrinsics()
            .constructors()
            .generator_function()
            .prototype()
    };

    let name_property = PropertyDescriptor::builder()
        .value(code.name().clone())
        .writable(false)
        .enumerable(false)
        .configurable(true)
        .build();

    let length_property = PropertyDescriptor::builder()
        .value(code.length)
        .writable(false)
        .enumerable(false)
        .configurable(true)
        .build();

    let prototype = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        if r#async {
            context.intrinsics().objects().async_generator()
        } else {
            context.intrinsics().objects().generator()
        },
        ObjectData::ordinary(),
    );

    let script_or_module = context.vm.active_runnable.clone();

    let constructor = if r#async {
        let function = Function::new(
            FunctionKind::AsyncGenerator {
                code,
                environments: context.vm.environments.clone(),
                home_object: None,
                class_object: None,
                script_or_module,
            },
            context.realm().clone(),
        );
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            function_prototype,
            ObjectData::async_generator_function(function),
        )
    } else {
        let function = Function::new(
            FunctionKind::Generator {
                code,
                environments: context.vm.environments.clone(),
                home_object: None,
                class_object: None,
                script_or_module,
            },
            context.realm().clone(),
        );
        JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            function_prototype,
            ObjectData::generator_function(function),
        )
    };

    let prototype_property = PropertyDescriptor::builder()
        .value(prototype)
        .writable(true)
        .enumerable(false)
        .configurable(false)
        .build();

    constructor
        .define_property_or_throw(PROTOTYPE, prototype_property, context)
        .expect("failed to define the prototype property of the generator function");
    constructor
        .define_property_or_throw(utf16!("name"), name_property, context)
        .expect("failed to define the name property of the generator function");
    constructor
        .define_property_or_throw(utf16!("length"), length_property, context)
        .expect("failed to define the length property of the generator function");

    constructor
}

impl JsObject {
    pub(crate) fn call_internal(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let old_realm = context.realm().clone();
        let old_active_fn = context.vm.active_function.clone();

        let context = &mut context.guard(move |ctx| {
            ctx.enter_realm(old_realm);
            ctx.vm.active_function = old_active_fn;
        });

        let this_function_object = self.clone();
        let active_function = self.clone();
        let object = self.borrow();
        let function_object = object.as_function().expect("not a function");
        let realm = function_object.realm().clone();

        context.enter_realm(realm);
        context.vm.active_function = Some(active_function);

        let (code, mut environments, class_object, mut script_or_module, async_, gen) =
            match function_object.kind() {
                FunctionKind::Native {
                    function,
                    constructor,
                } => {
                    let function = function.clone();
                    let constructor = *constructor;
                    drop(object);

                    return if constructor.is_some() {
                        function.call(&JsValue::undefined(), args, context)
                    } else {
                        function.call(this, args, context)
                    }
                    .map_err(|err| err.inject_realm(context.realm().clone()));
                }
                FunctionKind::Ordinary {
                    code,
                    environments,
                    class_object,
                    script_or_module,
                    ..
                } => {
                    let code = code.clone();
                    if code.is_class_constructor() {
                        return Err(JsNativeError::typ()
                            .with_message("class constructor cannot be invoked without 'new'")
                            .with_realm(context.realm().clone())
                            .into());
                    }
                    (
                        code,
                        environments.clone(),
                        class_object.clone(),
                        script_or_module.clone(),
                        false,
                        false,
                    )
                }
                FunctionKind::Async {
                    code,
                    environments,
                    class_object,

                    script_or_module,
                    ..
                } => (
                    code.clone(),
                    environments.clone(),
                    class_object.clone(),
                    script_or_module.clone(),
                    true,
                    false,
                ),
                FunctionKind::Generator {
                    code,
                    environments,
                    class_object,
                    script_or_module,
                    ..
                } => (
                    code.clone(),
                    environments.clone(),
                    class_object.clone(),
                    script_or_module.clone(),
                    false,
                    true,
                ),
                FunctionKind::AsyncGenerator {
                    code,
                    environments,
                    class_object,

                    script_or_module,
                    ..
                } => (
                    code.clone(),
                    environments.clone(),
                    class_object.clone(),
                    script_or_module.clone(),
                    true,
                    true,
                ),
            };

        drop(object);

        let promise_capability = (async_ && !gen).then(|| {
            PromiseCapability::new(
                &context.intrinsics().constructors().promise().constructor(),
                context,
            )
            .expect("cannot  fail per spec")
        });

        std::mem::swap(&mut environments, &mut context.vm.environments);

        let lexical_this_mode = code.this_mode == ThisMode::Lexical;

        let this = if lexical_this_mode {
            ThisBindingStatus::Lexical
        } else if code.strict() {
            ThisBindingStatus::Initialized(this.clone())
        } else if this.is_null_or_undefined() {
            ThisBindingStatus::Initialized(context.realm().global_this().clone().into())
        } else {
            ThisBindingStatus::Initialized(
                this.to_object(context)
                    .expect("conversion cannot fail")
                    .into(),
            )
        };

        let env_fp = context.vm.environments.len() as u32;

        let mut last_env = code.compile_environments.len() - 1;

        if let Some(class_object) = class_object {
            let index = context
                .vm
                .environments
                .push_lexical(code.compile_environments[last_env].clone());
            context
                .vm
                .environments
                .put_lexical_value(index, 0, class_object.into());
            last_env -= 1;
        }

        if code.has_binding_identifier() {
            let index = context
                .vm
                .environments
                .push_lexical(code.compile_environments[last_env].clone());
            context
                .vm
                .environments
                .put_lexical_value(index, 0, self.clone().into());
            last_env -= 1;
        }

        context.vm.environments.push_function(
            code.compile_environments[last_env].clone(),
            FunctionSlots::new(this, self.clone(), None),
        );

        if code.has_parameters_env_bindings() {
            last_env -= 1;
            context
                .vm
                .environments
                .push_lexical(code.compile_environments[last_env].clone());
        }

        // Taken from: `FunctionDeclarationInstantiation` abstract function.
        //
        // Spec: https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
        //
        // 22. If argumentsObjectNeeded is true, then
        if code.needs_arguments_object() {
            // a. If strict is true or simpleParameterList is false, then
            //     i. Let ao be CreateUnmappedArgumentsObject(argumentsList).
            // b. Else,
            //     i. NOTE: A mapped argument object is only provided for non-strict functions
            //              that don't have a rest parameter, any parameter
            //              default value initializers, or any destructured parameters.
            //     ii. Let ao be CreateMappedArgumentsObject(func, formals, argumentsList, env).
            let arguments_obj = if code.strict() || !code.params.is_simple() {
                Arguments::create_unmapped_arguments_object(args, context)
            } else {
                let env = context.vm.environments.current();
                Arguments::create_mapped_arguments_object(
                    &this_function_object,
                    &code.params,
                    args,
                    env.declarative_expect(),
                    context,
                )
            };
            let env_index = context.vm.environments.len() as u32 - 1;
            context
                .vm
                .environments
                .put_lexical_value(env_index, 0, arguments_obj.into());
        }

        let argument_count = args.len();

        // Push function arguments to the stack.
        let mut args = if code.params.as_ref().len() > args.len() {
            let mut v = args.to_vec();
            v.extend(vec![
                JsValue::Undefined;
                code.params.as_ref().len() - args.len()
            ]);
            v
        } else {
            args.to_vec()
        };
        args.reverse();
        let mut stack = args;

        std::mem::swap(&mut context.vm.stack, &mut stack);

        let mut frame = CallFrame::new(code)
            .with_argument_count(argument_count as u32)
            .with_env_fp(env_fp);
        frame.promise_capability = promise_capability.clone();

        std::mem::swap(&mut context.vm.active_runnable, &mut script_or_module);

        context.vm.push_frame(frame);

        let result = context
            .run()
            .consume()
            .map_err(|err| err.inject_realm(context.realm().clone()));

        let call_frame = context.vm.pop_frame().expect("frame must exist");
        std::mem::swap(&mut environments, &mut context.vm.environments);
        std::mem::swap(&mut context.vm.stack, &mut stack);
        std::mem::swap(&mut context.vm.active_runnable, &mut script_or_module);

        if let Some(promise_capability) = promise_capability {
            Ok(promise_capability.promise().clone().into())
        } else if gen {
            result?;
            let proto = this_function_object
                .get(PROTOTYPE, context)
                .expect("generator must have a prototype property")
                .as_object()
                .map_or_else(
                    || {
                        if async_ {
                            context.intrinsics().objects().async_generator()
                        } else {
                            context.intrinsics().objects().generator()
                        }
                    },
                    Clone::clone,
                );

            let data = if async_ {
                ObjectData::async_generator(AsyncGenerator {
                    state: AsyncGeneratorState::SuspendedStart,
                    context: Some(GeneratorContext::new(
                        environments,
                        stack,
                        context.vm.active_function.clone(),
                        call_frame,
                        context.realm().clone(),
                    )),
                    queue: VecDeque::new(),
                })
            } else {
                ObjectData::generator(Generator {
                    state: GeneratorState::SuspendedStart {
                        context: GeneratorContext::new(
                            environments,
                            stack,
                            context.vm.active_function.clone(),
                            call_frame,
                            context.realm().clone(),
                        ),
                    },
                })
            };

            let generator =
                Self::from_proto_and_data_with_shared_shape(context.root_shape(), proto, data);

            if async_ {
                let gen_clone = generator.clone();
                let mut generator_mut = generator.borrow_mut();
                let gen = generator_mut
                    .as_async_generator_mut()
                    .expect("must be object here");
                let gen_context = gen.context.as_mut().expect("must exist");
                // TODO: try to move this to the context itself.
                gen_context
                    .call_frame
                    .as_mut()
                    .expect("should have a call frame initialized")
                    .async_generator = Some(gen_clone);
            }

            Ok(generator.into())
        } else {
            result
        }
    }

    pub(crate) fn construct_internal(
        &self,
        args: &[JsValue],
        this_target: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let old_realm = context.realm().clone();
        let old_active_fn = context.vm.active_function.clone();
        let context = &mut context.guard(move |ctx| {
            ctx.enter_realm(old_realm);
            ctx.vm.active_function = old_active_fn;
        });

        let this_function_object = self.clone();
        let active_function = self.clone();
        let object = self.borrow();
        let function_object = object.as_function().expect("not a function");
        let realm = function_object.realm().clone();

        context.enter_realm(realm);
        context.vm.active_function = Some(active_function);

        match function_object.kind() {
            FunctionKind::Native {
                function,
                constructor,
                ..
            } => {
                let function = function.clone();
                let constructor = *constructor;
                drop(object);

                function
                    .call(this_target, args, context)
                    .map_err(|err| err.inject_realm(context.realm().clone()))
                    .and_then(|v| match v {
                        JsValue::Object(ref o) => Ok(o.clone()),
                        val => {
                            if constructor.expect("must be a constructor").is_base()
                                || val.is_undefined()
                            {
                                let prototype = get_prototype_from_constructor(
                                    this_target,
                                    StandardConstructors::object,
                                    context,
                                )?;
                                Ok(Self::from_proto_and_data_with_shared_shape(
                                    context.root_shape(),
                                    prototype,
                                    ObjectData::ordinary(),
                                ))
                            } else {
                                Err(JsNativeError::typ()
                                .with_message(
                                    "derived constructor can only return an Object or undefined",
                                )
                                .into())
                            }
                        }
                    })
            }
            FunctionKind::Ordinary {
                code,
                environments,
                constructor_kind,
                script_or_module,
                ..
            } => {
                let code = code.clone();
                let mut environments = environments.clone();
                let mut script_or_module = script_or_module.clone();
                let constructor_kind = *constructor_kind;
                drop(object);

                let this = if constructor_kind.is_base() {
                    // If the prototype of the constructor is not an object, then use the default object
                    // prototype as prototype for the new object
                    // see <https://tc39.es/ecma262/#sec-ordinarycreatefromconstructor>
                    // see <https://tc39.es/ecma262/#sec-getprototypefromconstructor>
                    let prototype = get_prototype_from_constructor(
                        this_target,
                        StandardConstructors::object,
                        context,
                    )?;
                    let this = Self::from_proto_and_data_with_shared_shape(
                        context.root_shape(),
                        prototype,
                        ObjectData::ordinary(),
                    );

                    this.initialize_instance_elements(self, context)?;

                    Some(this)
                } else {
                    None
                };

                let environments_len = environments.len();
                std::mem::swap(&mut environments, &mut context.vm.environments);

                let new_target = this_target.as_object().expect("must be object");

                let mut last_env = code.compile_environments.len() - 1;

                if code.has_binding_identifier() {
                    let index = context
                        .vm
                        .environments
                        .push_lexical(code.compile_environments[last_env].clone());
                    context
                        .vm
                        .environments
                        .put_lexical_value(index, 0, self.clone().into());
                    last_env -= 1;
                }

                context.vm.environments.push_function(
                    code.compile_environments[last_env].clone(),
                    FunctionSlots::new(
                        this.clone().map_or(ThisBindingStatus::Uninitialized, |o| {
                            ThisBindingStatus::Initialized(o.into())
                        }),
                        self.clone(),
                        Some(new_target.clone()),
                    ),
                );

                if code.has_parameters_env_bindings() {
                    last_env -= 1;
                    context
                        .vm
                        .environments
                        .push_lexical(code.compile_environments[last_env].clone());
                }

                // Taken from: `FunctionDeclarationInstantiation` abstract function.
                //
                // Spec: https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
                //
                // 22. If argumentsObjectNeeded is true, then
                if code.needs_arguments_object() {
                    // a. If strict is true or simpleParameterList is false, then
                    //     i. Let ao be CreateUnmappedArgumentsObject(argumentsList).
                    // b. Else,
                    //     i. NOTE: A mapped argument object is only provided for non-strict functions
                    //              that don't have a rest parameter, any parameter
                    //              default value initializers, or any destructured parameters.
                    //     ii. Let ao be CreateMappedArgumentsObject(func, formals, argumentsList, env).
                    let arguments_obj = if code.strict() || !code.params.is_simple() {
                        Arguments::create_unmapped_arguments_object(args, context)
                    } else {
                        let env = context.vm.environments.current();
                        Arguments::create_mapped_arguments_object(
                            &this_function_object,
                            &code.params,
                            args,
                            env.declarative_expect(),
                            context,
                        )
                    };

                    let env_index = context.vm.environments.len() as u32 - 1;
                    context
                        .vm
                        .environments
                        .put_lexical_value(env_index, 0, arguments_obj.into());
                }

                let argument_count = args.len();

                // Push function arguments to the stack.
                let args = if code.params.as_ref().len() > args.len() {
                    let mut v = args.to_vec();
                    v.extend(vec![
                        JsValue::Undefined;
                        code.params.as_ref().len() - args.len()
                    ]);
                    v
                } else {
                    args.to_vec()
                };

                for arg in args.iter().rev() {
                    context.vm.push(arg.clone());
                }

                let has_binding_identifier = code.has_binding_identifier();

                std::mem::swap(&mut context.vm.active_runnable, &mut script_or_module);

                context.vm.push_frame(
                    CallFrame::new(code)
                        .with_argument_count(argument_count as u32)
                        .with_env_fp(environments_len as u32),
                );

                let record = context.run();

                context.vm.pop_frame();

                std::mem::swap(&mut environments, &mut context.vm.environments);
                std::mem::swap(&mut context.vm.active_runnable, &mut script_or_module);

                let environment = if has_binding_identifier {
                    environments.truncate(environments_len + 2);
                    let environment = environments.pop();
                    environments.pop();
                    environment
                } else {
                    environments.truncate(environments_len + 1);
                    environments.pop()
                };

                let result = record
                    .consume()
                    .map_err(|err| err.inject_realm(context.realm().clone()))?;

                if let Some(result) = result.as_object() {
                    Ok(result.clone())
                } else if let Some(this) = this {
                    Ok(this)
                } else if !result.is_undefined() {
                    Err(JsNativeError::typ()
                        .with_message("derived constructor can only return an Object or undefined")
                        .into())
                } else {
                    let function_env = environment
                        .declarative_expect()
                        .kind()
                        .as_function()
                        .expect("must be function environment");
                    function_env
                        .get_this_binding()
                        .map(|v| {
                            v.expect("constructors cannot be arrow functions")
                                .as_object()
                                .expect("this binding must be object")
                                .clone()
                        })
                        .map_err(JsError::from)
                }
            }
            FunctionKind::Generator { .. }
            | FunctionKind::Async { .. }
            | FunctionKind::AsyncGenerator { .. } => {
                unreachable!("not a constructor")
            }
        }
    }
}
