//! `CodeBlock`
//!
//! This module is for the `CodeBlock` which implements a function representation in the VM

use crate::{
    builtins::{
        function::{OrdinaryFunction, ThisMode},
        OrdinaryObject,
    },
    environments::{BindingLocator, CompileTimeEnvironment},
    object::JsObject,
    Context, JsBigInt, JsString, JsValue,
};
use bitflags::bitflags;
use boa_gc::{empty_trace, Finalize, Gc, Trace};
use boa_profiler::Profiler;
use std::{cell::Cell, fmt::Display, mem::size_of, rc::Rc};
use thin_vec::ThinVec;

use super::{InlineCache, Instruction, InstructionIterator};

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
    pub(crate) struct CodeBlockFlags: u16 {
        /// Is this function in strict mode.
        const STRICT = 0b0000_0001;

        /// Indicates if the function is an expression and has a binding identifier.
        const HAS_BINDING_IDENTIFIER = 0b0000_0010;

        /// The `[[IsClassConstructor]]` internal slot.
        const IS_CLASS_CONSTRUCTOR = 0b0000_0100;

        /// The `[[ClassFieldInitializerName]]` internal slot.
        const IN_CLASS_FIELD_INITIALIZER = 0b0000_1000;

        /// `[[ConstructorKind]]`
        const IS_DERIVED_CONSTRUCTOR = 0b0001_0000;

        const IS_ASYNC = 0b0010_0000;
        const IS_GENERATOR = 0b0100_0000;

        /// Arrow and method functions don't have `"prototype"` property.
        const HAS_PROTOTYPE_PROPERTY = 0b1000_0000;

        /// Trace instruction execution to `stdout`.
        #[cfg(feature = "trace")]
        const TRACEABLE = 0b1000_0000_0000_0000;
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

#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) enum Constant {
    /// Property field names and private names `[[description]]`s.
    String(JsString),
    Function(Gc<CodeBlock>),
    BigInt(#[unsafe_ignore_trace] JsBigInt),

    /// Compile time environments in this function.
    // Safety: Nothing in CompileTimeEnvironment needs tracing, so this is safe.
    //
    // TODO(#3034): Maybe changing this to Gc after garbage collection would be better than Rc.
    CompileTimeEnvironment(#[unsafe_ignore_trace] Rc<CompileTimeEnvironment>),
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

    pub(crate) parameter_length: u32,

    pub(crate) register_count: u32,

    /// `[[ThisMode]]`
    pub(crate) this_mode: ThisMode,

    /// Used for constructing a `MappedArguments` object.
    #[unsafe_ignore_trace]
    pub(crate) mapped_arguments_binding_indices: ThinVec<Option<u32>>,

    /// Bytecode
    #[unsafe_ignore_trace]
    pub(crate) bytecode: Box<[u8]>,

    pub(crate) constants: ThinVec<Constant>,

    /// Locators for all bindings in the codeblock.
    #[unsafe_ignore_trace]
    pub(crate) bindings: Box<[BindingLocator]>,

    /// Exception [`Handler`]s.
    #[unsafe_ignore_trace]
    pub(crate) handlers: ThinVec<Handler>,

    /// inline caching
    pub(crate) ic: Box<[InlineCache]>,
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
            constants: ThinVec::default(),
            bindings: Box::default(),
            name,
            flags: Cell::new(flags),
            length,
            register_count: 0,
            this_mode: ThisMode::Global,
            mapped_arguments_binding_indices: ThinVec::new(),
            parameter_length: 0,
            handlers: ThinVec::default(),
            ic: Box::default(),
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

    /// Does this function have the `[[ClassFieldInitializerName]]` internal slot set to non-empty value.
    pub(crate) fn in_class_field_initializer(&self) -> bool {
        self.flags
            .get()
            .contains(CodeBlockFlags::IN_CLASS_FIELD_INITIALIZER)
    }

    /// Returns true if this function is a derived constructor.
    pub(crate) fn is_derived_constructor(&self) -> bool {
        self.flags
            .get()
            .contains(CodeBlockFlags::IS_DERIVED_CONSTRUCTOR)
    }

    /// Returns true if this function an async function.
    pub(crate) fn is_async(&self) -> bool {
        self.flags.get().contains(CodeBlockFlags::IS_ASYNC)
    }

    /// Returns true if this function an generator function.
    pub(crate) fn is_generator(&self) -> bool {
        self.flags.get().contains(CodeBlockFlags::IS_GENERATOR)
    }

    /// Returns true if this function a async generator function.
    pub(crate) fn is_async_generator(&self) -> bool {
        self.flags
            .get()
            .contains(CodeBlockFlags::IS_ASYNC | CodeBlockFlags::IS_GENERATOR)
    }

    /// Returns true if this function an async function.
    pub(crate) fn is_ordinary(&self) -> bool {
        !self.is_async() && !self.is_generator()
    }

    /// Returns true if this function has the `"prototype"` property when function object is created.
    pub(crate) fn has_prototype_property(&self) -> bool {
        self.flags
            .get()
            .contains(CodeBlockFlags::HAS_PROTOTYPE_PROPERTY)
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

    /// Get the [`JsString`] constant from the [`CodeBlock`].
    ///
    /// # Panics
    ///
    /// If the type of the [`Constant`] is not [`Constant::String`].
    /// Or `index` is greater or equal to length of `constants`.
    pub(crate) fn constant_string(&self, index: usize) -> JsString {
        if let Some(Constant::String(value)) = self.constants.get(index) {
            return value.clone();
        }

        panic!("expected string constant at index {index}")
    }

    /// Get the function ([`Gc<CodeBlock>`]) constant from the [`CodeBlock`].
    ///
    /// # Panics
    ///
    /// If the type of the [`Constant`] is not [`Constant::Function`].
    /// Or `index` is greater or equal to length of `constants`.
    pub(crate) fn constant_function(&self, index: usize) -> Gc<Self> {
        if let Some(Constant::Function(value)) = self.constants.get(index) {
            return value.clone();
        }

        panic!("expected function constant at index {index}")
    }

    /// Get the [`CompileTimeEnvironment`] constant from the [`CodeBlock`].
    ///
    /// # Panics
    ///
    /// If the type of the [`Constant`] is not [`Constant::CompileTimeEnvironment`].
    /// Or `index` is greater or equal to length of `constants`.
    pub(crate) fn constant_compile_time_environment(
        &self,
        index: usize,
    ) -> Rc<CompileTimeEnvironment> {
        if let Some(Constant::CompileTimeEnvironment(value)) = self.constants.get(index) {
            return value.clone();
        }

        panic!("expected compile time environment constant at index {index}")
    }
}

/// ---- `CodeBlock` private API ----
impl CodeBlock {
    /// Read type T from code.
    ///
    /// # Safety
    ///
    /// Does not check if read happens out-of-bounds.
    pub(crate) const unsafe fn read_unchecked<T>(&self, offset: usize) -> T
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
    pub(crate) fn instruction_operands(&self, instruction: &Instruction) -> String {
        match instruction {
            Instruction::SetRegisterFromAccumulator { register }
            | Instruction::SetAccumulator { register } => format!("R{}", register.value()),
            Instruction::Add { .. }
            | Instruction::Sub { .. }
            | Instruction::Div { .. }
            | Instruction::Mul { .. }
            | Instruction::Mod { .. }
            | Instruction::Pow { .. }
            | Instruction::ShiftRight { .. }
            | Instruction::ShiftLeft { .. }
            | Instruction::UnsignedShiftRight { .. }
            | Instruction::BitOr { .. }
            | Instruction::BitAnd { .. }
            | Instruction::BitXor { .. }
            | Instruction::BitNot { .. }
            | Instruction::In { .. }
            | Instruction::Eq { .. }
            | Instruction::NotEq { .. }
            | Instruction::GreaterThan { .. }
            | Instruction::GreaterThanOrEq { .. }
            | Instruction::LessThan { .. }
            | Instruction::LessThanOrEq { .. }
            | Instruction::InstanceOf { .. }
            | Instruction::StrictNotEq { .. }
            | Instruction::StrictEq { .. }
            | Instruction::InPrivate { .. }
            | Instruction::Inc { .. }
            | Instruction::Dec { .. }
            | Instruction::ToNumeric { .. } => "TODO: fix".to_string(),
            Instruction::PopIntoRegister { dst } => format!("R{}", dst.value()),
            Instruction::PushFromRegister { src } => format!("R{}", src.value()),
            Instruction::Move {
                operand_types,
                dst: r1,
                src: r2,
            } => {
                format!(
                    "dst:reg{}, src:{}",
                    r1.value(),
                    r2.to_string::<0>(*operand_types)
                )
            }
            Instruction::SetFunctionName { prefix } => match prefix {
                0 => "prefix: none",
                1 => "prefix: get",
                2 => "prefix: set",
                _ => unreachable!(),
            }
            .to_owned(),
            Instruction::RotateLeft { n } | Instruction::RotateRight { n } => n.to_string(),
            Instruction::Generator { r#async } => {
                format!("async: {async}")
            }
            Instruction::PushInt8 { value } => value.to_string(),
            Instruction::PushInt16 { value } => value.to_string(),
            Instruction::PushInt32 { value } => value.to_string(),
            Instruction::PushFloat { value } => ryu_js::Buffer::new().format(*value).to_string(),
            Instruction::PushDouble { value } => ryu_js::Buffer::new().format(*value).to_string(),
            Instruction::PushLiteral { index }
            | Instruction::ThrowNewTypeError { message: index }
            | Instruction::ThrowNewSyntaxError { message: index }
            | Instruction::HasRestrictedGlobalProperty { index }
            | Instruction::CanDeclareGlobalFunction { index }
            | Instruction::CanDeclareGlobalVar { index } => index.value().to_string(),
            Instruction::PushRegExp {
                pattern_index: source_index,
                flags_index: flag_index,
            } => {
                let pattern = self
                    .constant_string(source_index.value() as usize)
                    .to_std_string_escaped();
                let flags = self
                    .constant_string(flag_index.value() as usize)
                    .to_std_string_escaped();
                format!("/{pattern}/{flags}")
            }
            Instruction::Jump { address: value }
            | Instruction::JumpIfTrue { address: value }
            | Instruction::JumpIfFalse { address: value }
            | Instruction::JumpIfNotUndefined { address: value }
            | Instruction::JumpIfNullOrUndefined { address: value }
            | Instruction::Case { address: value }
            | Instruction::Default { address: value } => value.to_string(),
            Instruction::LogicalAnd {
                exit,
                lhs,
                operand_types,
            }
            | Instruction::LogicalOr {
                exit,
                lhs,
                operand_types,
            }
            | Instruction::Coalesce {
                exit,
                lhs,
                operand_types,
            } => {
                format!("lhs:{} exit:{exit}", lhs.to_string::<0>(*operand_types))
            }
            Instruction::CallEval {
                argument_count: value,
            }
            | Instruction::Call {
                argument_count: value,
            }
            | Instruction::New {
                argument_count: value,
            }
            | Instruction::SuperCall {
                argument_count: value,
            }
            | Instruction::ConcatToString { value_count: value }
            | Instruction::GetArgument { index: value } => value.value().to_string(),
            Instruction::PushDeclarativeEnvironment {
                compile_environments_index,
            } => compile_environments_index.value().to_string(),
            Instruction::CopyDataProperties {
                excluded_key_count: value1,
                excluded_key_count_computed: value2,
            } => format!("{}, {}", value1.value(), value2.value()),
            Instruction::GeneratorDelegateNext {
                return_method_undefined: value1,
                throw_method_undefined: value2,
            }
            | Instruction::GeneratorDelegateResume {
                exit: value1,
                r#return: value2,
            } => {
                format!("{value1}, {value2}")
            }
            Instruction::TemplateLookup { exit: value, site } => format!("{value}, {site}"),
            Instruction::TemplateCreate { count, site } => {
                format!("{}, {site}", count.value())
            }
            Instruction::GetFunction { dst, index } => {
                let index = index.value() as usize;
                format!(
                    "R{} = {index:04}: '{}' (length: {})",
                    dst.value(),
                    self.constant_function(index).name().to_std_string_escaped(),
                    self.constant_function(index).length
                )
            }
            Instruction::DefVar { index }
            | Instruction::DefInitVar { index }
            | Instruction::PutLexicalValue { index }
            | Instruction::GetName { index }
            | Instruction::GetLocator { index }
            | Instruction::GetNameAndLocator { index }
            | Instruction::GetNameOrUndefined { index }
            | Instruction::SetName { index }
            | Instruction::DeleteName { index } => {
                format!(
                    "{:04}: '{}'",
                    index.value(),
                    self.bindings[index.value() as usize]
                        .name()
                        .to_std_string_escaped()
                )
            }
            Instruction::DefineOwnPropertyByName { index }
            | Instruction::DefineClassStaticMethodByName { index }
            | Instruction::DefineClassMethodByName { index }
            | Instruction::SetPropertyGetterByName { index }
            | Instruction::DefineClassStaticGetterByName { index }
            | Instruction::DefineClassGetterByName { index }
            | Instruction::SetPropertySetterByName { index }
            | Instruction::DefineClassStaticSetterByName { index }
            | Instruction::DefineClassSetterByName { index }
            | Instruction::ThrowMutateImmutable { index }
            | Instruction::DeletePropertyByName { index }
            | Instruction::SetPrivateField { index }
            | Instruction::DefinePrivateField { index }
            | Instruction::SetPrivateMethod { index }
            | Instruction::SetPrivateSetter { index }
            | Instruction::SetPrivateGetter { index }
            | Instruction::GetPrivateField { index }
            | Instruction::PushClassFieldPrivate { index }
            | Instruction::PushClassPrivateGetter { index }
            | Instruction::PushClassPrivateSetter { index }
            | Instruction::PushClassPrivateMethod { index } => {
                format!(
                    "{:04}: '{}'",
                    index.value(),
                    self.constant_string(index.value() as usize)
                        .to_std_string_escaped(),
                )
            }
            Instruction::GetPropertyByName {
                operand_types,
                dst,
                receiver,
                value,
                index,
            } => {
                let ic = &self.ic[index.value() as usize];
                let slot = ic.slot();
                format!(
                    "dst:reg{}, receiver:{}, value:{}, {:04}: '{}', Shape: 0x{:x}, Slot: index: {}, attributes {:?}",
                    dst.value(),
                    receiver.to_string::<0>(*operand_types),
                    value.to_string::<1>(*operand_types),
                    index.value(),
                    ic.name.to_std_string_escaped(),
                    ic.shape.borrow().to_addr_usize(),
                    slot.index,
                    slot.attributes,
                )
            }
            Instruction::SetPropertyByName { index } => {
                let ic = &self.ic[index.value() as usize];
                let slot = ic.slot();
                format!(
                    "{:04}: '{}', Shape: 0x{:x}, Slot: index: {}, attributes {:?}",
                    index.value(),
                    ic.name.to_std_string_escaped(),
                    ic.shape.borrow().to_addr_usize(),
                    slot.index,
                    slot.attributes,
                )
            }
            Instruction::PushPrivateEnvironment {
                class,
                name_indices,
                operand_types,
            } => {
                format!(
                    "class:{}, names:{name_indices:?}",
                    class.to_string::<0>(*operand_types)
                )
            }
            Instruction::JumpTable { default, addresses } => {
                let mut operands = format!("#{}: Default: {default:4}", addresses.len());
                for (i, address) in addresses.iter().enumerate() {
                    operands += &format!(", {i}: {address}");
                }
                operands
            }
            Instruction::JumpIfNotResumeKind { exit, resume_kind } => {
                format!("ResumeKind: {resume_kind:?}, exit: {exit}")
            }
            Instruction::CreateIteratorResult { done } => {
                format!("done: {done}")
            }
            Instruction::CreateGlobalFunctionBinding {
                index,
                configurable,
            }
            | Instruction::CreateGlobalVarBinding {
                index,
                configurable,
            } => {
                let name = self
                    .constant_string(index.value() as usize)
                    .to_std_string_escaped();
                format!("name: {name}, configurable: {configurable}")
            }
            Instruction::PushClassPrototype {
                operand_types,
                dst,
                class,
                superclass,
            } => {
                format!(
                    "dst:reg{}, class:{}, superclass:{}",
                    dst.value(),
                    class.to_string::<0>(*operand_types),
                    superclass.to_string::<1>(*operand_types)
                )
            }
            Instruction::SetClassPrototype {
                operand_types,
                dst,
                prototype,
                class,
            } => {
                format!(
                    "dst:reg{}, prototype:{}, class:{}",
                    dst.value(),
                    prototype.to_string::<0>(*operand_types),
                    class.to_string::<1>(*operand_types)
                )
            }
            Instruction::Pop
            | Instruction::Dup
            | Instruction::Swap
            | Instruction::PushZero
            | Instruction::PushOne
            | Instruction::PushNaN
            | Instruction::PushPositiveInfinity
            | Instruction::PushNegativeInfinity
            | Instruction::PushNull
            | Instruction::PushTrue
            | Instruction::PushFalse
            | Instruction::PushUndefined
            | Instruction::PushEmptyObject
            | Instruction::SetHomeObject
            | Instruction::TypeOf
            | Instruction::Void
            | Instruction::LogicalNot
            | Instruction::Pos
            | Instruction::Neg
            | Instruction::GetPropertyByValue
            | Instruction::GetPropertyByValuePush
            | Instruction::SetPropertyByValue
            | Instruction::DefineOwnPropertyByValue
            | Instruction::DefineClassStaticMethodByValue
            | Instruction::DefineClassMethodByValue
            | Instruction::SetPropertyGetterByValue
            | Instruction::DefineClassStaticGetterByValue
            | Instruction::DefineClassGetterByValue
            | Instruction::SetPropertySetterByValue
            | Instruction::DefineClassStaticSetterByValue
            | Instruction::DefineClassSetterByValue
            | Instruction::DeletePropertyByValue
            | Instruction::DeleteSuperThrow
            | Instruction::ToPropertyKey
            | Instruction::ToBoolean
            | Instruction::Throw
            | Instruction::ReThrow
            | Instruction::Exception
            | Instruction::MaybeException
            | Instruction::This
            | Instruction::ThisForObjectEnvironmentName { .. }
            | Instruction::Super
            | Instruction::CheckReturn
            | Instruction::Return
            | Instruction::AsyncGeneratorClose
            | Instruction::CreatePromiseCapability
            | Instruction::CompletePromiseCapability
            | Instruction::PopEnvironment
            | Instruction::IncrementLoopIteration
            | Instruction::CreateForInIterator
            | Instruction::GetIterator
            | Instruction::GetAsyncIterator
            | Instruction::IteratorNext
            | Instruction::IteratorNextWithoutPop
            | Instruction::IteratorFinishAsyncNext
            | Instruction::IteratorValue
            | Instruction::IteratorValueWithoutPop
            | Instruction::IteratorResult
            | Instruction::IteratorDone
            | Instruction::IteratorToArray
            | Instruction::IteratorReturn
            | Instruction::IteratorStackEmpty
            | Instruction::RequireObjectCoercible
            | Instruction::ValueNotNullOrUndefined
            | Instruction::RestParameterInit
            | Instruction::PushValueToArray
            | Instruction::PushElisionToArray
            | Instruction::PushIteratorToArray
            | Instruction::PushNewArray
            | Instruction::GeneratorYield
            | Instruction::AsyncGeneratorYield
            | Instruction::GeneratorNext
            | Instruction::PushClassField
            | Instruction::SuperCallDerived
            | Instruction::Await
            | Instruction::NewTarget
            | Instruction::ImportMeta
            | Instruction::SuperCallPrepare
            | Instruction::CallEvalSpread
            | Instruction::CallSpread
            | Instruction::NewSpread
            | Instruction::SuperCallSpread
            | Instruction::SetPrototype
            | Instruction::PushObjectEnvironment
            | Instruction::IsObject
            | Instruction::SetNameByLocator
            | Instruction::PopPrivateEnvironment
            | Instruction::ImportCall
            | Instruction::GetAccumulator
            | Instruction::SetAccumulatorFromStack
            | Instruction::BindThisValue
            | Instruction::CreateMappedArgumentsObject
            | Instruction::CreateUnmappedArgumentsObject
            | Instruction::Nop => String::new(),

            Instruction::U16Operands
            | Instruction::U32Operands
            | Instruction::Reserved1
            | Instruction::Reserved2
            | Instruction::Reserved3
            | Instruction::Reserved4
            | Instruction::Reserved5
            | Instruction::Reserved6
            | Instruction::Reserved7
            | Instruction::Reserved8
            | Instruction::Reserved9
            | Instruction::Reserved10
            | Instruction::Reserved11
            | Instruction::Reserved12
            | Instruction::Reserved13
            | Instruction::Reserved14
            | Instruction::Reserved15
            | Instruction::Reserved16
            | Instruction::Reserved17
            | Instruction::Reserved18
            | Instruction::Reserved19
            | Instruction::Reserved20
            | Instruction::Reserved21
            | Instruction::Reserved22
            | Instruction::Reserved23
            | Instruction::Reserved24
            | Instruction::Reserved25
            | Instruction::Reserved26
            | Instruction::Reserved27
            | Instruction::Reserved28
            | Instruction::Reserved29
            | Instruction::Reserved30
            | Instruction::Reserved31
            | Instruction::Reserved32
            | Instruction::Reserved33
            | Instruction::Reserved34
            | Instruction::Reserved35
            | Instruction::Reserved36
            | Instruction::Reserved37
            | Instruction::Reserved38
            | Instruction::Reserved39
            | Instruction::Reserved40
            | Instruction::Reserved41
            | Instruction::Reserved42
            | Instruction::Reserved43
            | Instruction::Reserved44
            | Instruction::Reserved45
            | Instruction::Reserved46
            | Instruction::Reserved47
            | Instruction::Reserved48
            | Instruction::Reserved49 => unreachable!("Reserved opcodes are unrechable"),
        }
    }
}

impl Display for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();

        writeln!(
            f,
            "{:-^70}\nLocation  Count    Handler    Opcode                     Operands\n",
            format!("Compiled Output: '{}'", name.to_std_string_escaped()),
        )?;

        let mut iterator = InstructionIterator::new(&self.bytecode);

        let mut count = 0;
        while let Some((instruction_start_pc, varying_operand_kind, instruction)) = iterator.next()
        {
            let opcode = instruction.opcode().as_str();
            let operands = self.instruction_operands(&instruction);
            let pc = iterator.pc();

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

            let varying_operand_kind = match varying_operand_kind {
                super::VaryingOperandKind::U8 => "",
                super::VaryingOperandKind::U16 => ".U16",
                super::VaryingOperandKind::U32 => ".U32",
            };

            writeln!(
                f,
                    "{instruction_start_pc:06}    {count:04}   {handler}    {opcode}{varying_operand_kind:<27}{operands}",
                )?;
            count += 1;
        }

        f.write_str("\nConstants:")?;

        if self.constants.is_empty() {
            f.write_str(" <empty>\n")?;
        } else {
            f.write_str("\n")?;
            for (i, value) in self.constants.iter().enumerate() {
                write!(f, "    {i:04}: ")?;
                match value {
                    Constant::String(v) => {
                        writeln!(
                            f,
                            "[STRING] \"{}\"",
                            v.to_std_string_escaped().escape_debug()
                        )?;
                    }
                    Constant::BigInt(v) => writeln!(f, "[BIGINT] {v}n")?,
                    Constant::Function(code) => writeln!(
                        f,
                        "[FUNCTION] name: '{}' (length: {})\n",
                        code.name().to_std_string_escaped(),
                        code.length
                    )?,
                    Constant::CompileTimeEnvironment(v) => {
                        writeln!(
                            f,
                            "[ENVIRONMENT] index: {}, bindings: {}",
                            v.environment_index(),
                            v.num_bindings()
                        )?;
                    }
                }
            }
        }

        f.write_str("\nBindings:\n")?;
        if self.bindings.is_empty() {
            f.write_str("    <empty>\n")?;
        } else {
            for (i, binding_locator) in self.bindings.iter().enumerate() {
                writeln!(
                    f,
                    "    {i:04}: {}",
                    binding_locator.name().to_std_string_escaped()
                )?;
            }
        }

        f.write_str("\nHandlers:\n")?;
        if self.handlers.is_empty() {
            f.write_str("    <empty>\n")?;
        } else {
            for (i, handler) in self.handlers.iter().enumerate() {
                writeln!(f,
                    "    {i:04}: Range: [{:04}, {:04}): Handler: {:04}, Stack: {:02}, Environment: {:02}",
                    handler.start,
                    handler.end,
                    handler.handler(),
                    handler.stack_count,
                    handler.environment_count,
                )?;
            }
        }
        Ok(())
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
    prototype: JsObject,
    context: &mut Context,
) -> JsObject {
    let _timer = Profiler::global().start_event("create_function_object", "vm");

    let name: JsValue = code.name().clone().into();
    let length: JsValue = code.length.into();

    let script_or_module = context.get_active_script_or_module();

    let is_async = code.is_async();
    let is_generator = code.is_generator();
    let function = OrdinaryFunction::new(
        code,
        context.vm.environments.clone(),
        script_or_module,
        context.realm().clone(),
    );

    let templates = context.intrinsics().templates();

    let (mut template, storage, constructor_prototype) = if is_generator {
        let prototype = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            if is_async {
                context.intrinsics().objects().async_generator()
            } else {
                context.intrinsics().objects().generator()
            },
            OrdinaryObject,
        );

        (
            templates.function_with_prototype_without_proto().clone(),
            vec![length, name, prototype.into()],
            None,
        )
    } else if is_async {
        (
            templates.function_without_proto().clone(),
            vec![length, name],
            None,
        )
    } else {
        let constructor_prototype = templates
            .function_prototype()
            .create(OrdinaryObject, vec![JsValue::undefined()]);

        let template = templates.function_with_prototype_without_proto();

        (
            template.clone(),
            vec![length, name, constructor_prototype.clone().into()],
            Some(constructor_prototype),
        )
    };

    template.set_prototype(prototype);

    let constructor = template.create(function, storage);

    if let Some(constructor_prototype) = &constructor_prototype {
        constructor_prototype.borrow_mut().properties_mut().storage[0] = constructor.clone().into();
    }
    constructor
}

/// Creates a new function object.
///
/// This is prefered over [`create_function_object`] if prototype is [`None`],
/// because it constructs the function from a pre-initialized object template,
/// with all the properties and prototype set.
pub(crate) fn create_function_object_fast(code: Gc<CodeBlock>, context: &mut Context) -> JsObject {
    let _timer = Profiler::global().start_event("create_function_object_fast", "vm");

    let name: JsValue = code.name().clone().into();
    let length: JsValue = code.length.into();

    let script_or_module = context.get_active_script_or_module();

    let is_async = code.is_async();
    let is_generator = code.is_generator();
    let has_prototype_property = code.has_prototype_property();
    let function = OrdinaryFunction::new(
        code,
        context.vm.environments.clone(),
        script_or_module,
        context.realm().clone(),
    );

    if is_generator {
        let prototype = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            if is_async {
                context.intrinsics().objects().async_generator()
            } else {
                context.intrinsics().objects().generator()
            },
            OrdinaryObject,
        );
        let template = if is_async {
            context.intrinsics().templates().async_generator_function()
        } else {
            context.intrinsics().templates().generator_function()
        };

        template.create(function, vec![length, name, prototype.into()])
    } else if is_async {
        context
            .intrinsics()
            .templates()
            .async_function()
            .create(function, vec![length, name])
    } else if !has_prototype_property {
        context
            .intrinsics()
            .templates()
            .function()
            .create(function, vec![length, name])
    } else {
        let prototype = context
            .intrinsics()
            .templates()
            .function_prototype()
            .create(OrdinaryObject, vec![JsValue::undefined()]);

        let constructor = context
            .intrinsics()
            .templates()
            .function_with_prototype()
            .create(function, vec![length, name, prototype.clone().into()]);

        prototype.borrow_mut().properties_mut().storage[0] = constructor.clone().into();

        constructor
    }
}
