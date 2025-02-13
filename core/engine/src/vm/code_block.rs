//! `CodeBlock`
//!
//! This module is for the `CodeBlock` which implements a function representation in the VM

use crate::{
    builtins::{
        function::{OrdinaryFunction, ThisMode},
        OrdinaryObject,
    },
    object::JsObject,
    Context, JsBigInt, JsString, JsValue, SpannedSourceText,
};
use bitflags::bitflags;
use boa_ast::scope::{BindingLocator, Scope};
use boa_gc::{empty_trace, Finalize, Gc, Trace};
use boa_profiler::Profiler;
use std::{cell::Cell, fmt::Display, mem::size_of};
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

        /// If the function requires a function scope.
        const HAS_FUNCTION_SCOPE = 0b1_0000_0000;

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

    /// Declarative or function scope.
    // Safety: Nothing in `Scope` needs tracing, so this is safe.
    Scope(#[unsafe_ignore_trace] Scope),
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

    pub(crate) local_bindings_initialized: Box<[bool]>,

    /// Exception [`Handler`]s.
    #[unsafe_ignore_trace]
    pub(crate) handlers: ThinVec<Handler>,

    /// inline caching
    pub(crate) ic: Box<[InlineCache]>,

    /// source text of the code block
    pub(crate) source_text_spanned: SpannedSourceText,
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
            local_bindings_initialized: Box::default(),
            name,
            flags: Cell::new(flags),
            length,
            register_count: 0,
            this_mode: ThisMode::Global,
            mapped_arguments_binding_indices: ThinVec::new(),
            parameter_length: 0,
            handlers: ThinVec::default(),
            ic: Box::default(),
            source_text_spanned: SpannedSourceText::new_empty(),
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

    /// Returns true if this function requires a function scope.
    pub(crate) fn has_function_scope(&self) -> bool {
        self.flags
            .get()
            .contains(CodeBlockFlags::HAS_FUNCTION_SCOPE)
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

    /// Get the [`Scope`] constant from the [`CodeBlock`].
    ///
    /// # Panics
    ///
    /// If the type of the [`Constant`] is not [`Constant::Scope`].
    /// Or `index` is greater or equal to length of `constants`.
    pub(crate) fn constant_scope(&self, index: usize) -> Scope {
        if let Some(Constant::Scope(value)) = self.constants.get(index) {
            return value.clone();
        }

        panic!("expected scope constant at index {index}")
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
            Instruction::SetRegisterFromAccumulator { dst }
            | Instruction::PopIntoRegister { dst }
            | Instruction::PushZero { dst }
            | Instruction::PushOne { dst }
            | Instruction::PushNaN { dst }
            | Instruction::PushPositiveInfinity { dst }
            | Instruction::PushNegativeInfinity { dst }
            | Instruction::PushNull { dst }
            | Instruction::PushTrue { dst }
            | Instruction::PushFalse { dst }
            | Instruction::PushUndefined { dst }
            | Instruction::Exception { dst }
            | Instruction::This { dst }
            | Instruction::Super { dst }
            | Instruction::SuperCallPrepare { dst }
            | Instruction::NewTarget { dst }
            | Instruction::ImportMeta { dst }
            | Instruction::CreateMappedArgumentsObject { dst }
            | Instruction::CreateUnmappedArgumentsObject { dst }
            | Instruction::RestParameterInit { dst }
            | Instruction::PushNewArray { dst } => format!("dst:{}", dst.value()),
            Instruction::Add { lhs, rhs, dst }
            | Instruction::Sub { lhs, rhs, dst }
            | Instruction::Div { lhs, rhs, dst }
            | Instruction::Mul { lhs, rhs, dst }
            | Instruction::Mod { lhs, rhs, dst }
            | Instruction::Pow { lhs, rhs, dst }
            | Instruction::ShiftRight { lhs, rhs, dst }
            | Instruction::ShiftLeft { lhs, rhs, dst }
            | Instruction::UnsignedShiftRight { lhs, rhs, dst }
            | Instruction::BitOr { lhs, rhs, dst }
            | Instruction::BitAnd { lhs, rhs, dst }
            | Instruction::BitXor { lhs, rhs, dst }
            | Instruction::BitNot { lhs, rhs, dst }
            | Instruction::In { lhs, rhs, dst }
            | Instruction::Eq { lhs, rhs, dst }
            | Instruction::StrictEq { lhs, rhs, dst }
            | Instruction::NotEq { lhs, rhs, dst }
            | Instruction::StrictNotEq { lhs, rhs, dst }
            | Instruction::GreaterThan { lhs, rhs, dst }
            | Instruction::GreaterThanOrEq { lhs, rhs, dst }
            | Instruction::LessThan { lhs, rhs, dst }
            | Instruction::LessThanOrEq { lhs, rhs, dst }
            | Instruction::InstanceOf { lhs, rhs, dst } => {
                format!(
                    "lhs:{}, rhs:{}, dst:{}",
                    lhs.value(),
                    rhs.value(),
                    dst.value()
                )
            }
            Instruction::InPrivate { dst, index, rhs } => {
                format!(
                    "rhs:{}, index:{}, dst:{}",
                    rhs.value(),
                    index.value(),
                    dst.value()
                )
            }
            Instruction::Inc { src, dst }
            | Instruction::Dec { src, dst }
            | Instruction::Move { src, dst }
            | Instruction::ToPropertyKey { src, dst }
            | Instruction::PopIntoLocal { src, dst }
            | Instruction::PushFromLocal { src, dst } => {
                format!("src:{}, dst:{}", src.value(), dst.value())
            }
            Instruction::SetFunctionName {
                function,
                name,
                prefix,
            } => {
                format!(
                    "function:{}, name:{}, prefix:{}",
                    function.value(),
                    name.value(),
                    match prefix {
                        0 => "prefix:",
                        1 => "prefix: get",
                        2 => "prefix: set",
                        _ => unreachable!(),
                    }
                )
            }
            Instruction::Generator { r#async } => {
                format!("async: {async}")
            }
            Instruction::PushInt8 { value, dst } => {
                format!("value:{}, dst:{}", value, dst.value())
            }
            Instruction::PushInt16 { value, dst } => {
                format!("value:{}, dst:{}", value, dst.value())
            }
            Instruction::PushInt32 { value, dst } => {
                format!("value:{}, dst:{}", value, dst.value())
            }
            Instruction::PushFloat { value, dst } => {
                format!("value:{}, dst:{}", value, dst.value())
            }
            Instruction::PushDouble { value, dst } => {
                format!("value:{}, dst:{}", value, dst.value())
            }
            Instruction::PushLiteral { index, dst }
            | Instruction::ThisForObjectEnvironmentName { index, dst }
            | Instruction::GetFunction { index, dst }
            | Instruction::HasRestrictedGlobalProperty { index, dst }
            | Instruction::CanDeclareGlobalFunction { index, dst }
            | Instruction::CanDeclareGlobalVar { index, dst }
            | Instruction::GetArgument { index, dst } => {
                format!("index:{}, dst:{}", index.value(), dst.value())
            }
            Instruction::ThrowNewTypeError { message: index }
            | Instruction::ThrowNewSyntaxError { message: index } => index.value().to_string(),
            Instruction::PushRegExp {
                pattern_index,
                flags_index,
                dst,
            } => {
                format!(
                    "pattern:{}, flags:{}, dst:{}",
                    pattern_index.value(),
                    flags_index.value(),
                    dst.value()
                )
            }
            Instruction::Jump { address } => address.to_string(),
            Instruction::JumpIfTrue { address, value }
            | Instruction::JumpIfFalse { address, value }
            | Instruction::JumpIfNotUndefined { address, value }
            | Instruction::JumpIfNullOrUndefined { address, value }
            | Instruction::LogicalAnd { address, value }
            | Instruction::LogicalOr { address, value }
            | Instruction::Coalesce { address, value } => {
                format!("value:{}, address:{}", value.value(), address)
            }
            Instruction::Case {
                address,
                value,
                condition,
            } => {
                format!(
                    "value:{}, condition:{}, address:{}",
                    value.value(),
                    condition.value(),
                    address
                )
            }
            Instruction::CallEval {
                argument_count,
                scope_index,
            } => {
                format!(
                    "argument_count:{}, scope_index:{}",
                    argument_count.value(),
                    scope_index.value()
                )
            }
            Instruction::CallEvalSpread { scope_index }
            | Instruction::PushScope { scope_index } => {
                format!("scope_index:{}", scope_index.value())
            }
            Instruction::Call { argument_count }
            | Instruction::New { argument_count }
            | Instruction::SuperCall { argument_count } => {
                format!("argument_count:{}", argument_count.value())
            }
            Instruction::DefVar { binding_index } | Instruction::GetLocator { binding_index } => {
                format!("binding_index:{}", binding_index.value())
            }
            Instruction::DefInitVar { src, binding_index }
            | Instruction::PutLexicalValue { src, binding_index }
            | Instruction::SetName { src, binding_index } => {
                format!(
                    "src:{}, binding_index:{}",
                    src.value(),
                    binding_index.value()
                )
            }
            Instruction::GetName { dst, binding_index }
            | Instruction::GetNameAndLocator { dst, binding_index }
            | Instruction::GetNameOrUndefined { dst, binding_index }
            | Instruction::DeleteName { dst, binding_index } => {
                format!(
                    "dst:{}, binding_index:{}",
                    dst.value(),
                    binding_index.value()
                )
            }
            Instruction::GetNameGlobal {
                dst,
                binding_index,
                ic_index,
            } => {
                format!(
                    "dst:{}, binding_index:{}, ic_index:{}",
                    dst.value(),
                    binding_index.value(),
                    ic_index.value()
                )
            }
            Instruction::GeneratorDelegateNext {
                return_method_undefined,
                throw_method_undefined,
                value,
                resume_kind,
                is_return,
            } => {
                format!(
                    "return_method_undefined:{}, throw_method_undefined:{}, value:{}, resume_kind:{}, is_return:{}",
                    return_method_undefined, throw_method_undefined, value.value(), resume_kind.value(), is_return.value()
                )
            }
            Instruction::GeneratorDelegateResume {
                r#return,
                exit,
                value,
                resume_kind,
                is_return,
            } => {
                format!(
                    "return:{}, exit:{}, value:{}, resume_kind:{}, is_return:{}",
                    r#return,
                    exit,
                    value.value(),
                    resume_kind.value(),
                    is_return.value()
                )
            }
            Instruction::DefineOwnPropertyByName {
                object,
                value,
                name_index,
            }
            | Instruction::SetPropertyGetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::SetPropertySetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefinePrivateField {
                object,
                value,
                name_index,
            }
            | Instruction::SetPrivateMethod {
                object,
                value,
                name_index,
            }
            | Instruction::SetPrivateSetter {
                object,
                value,
                name_index,
            }
            | Instruction::SetPrivateGetter {
                object,
                value,
                name_index,
            }
            | Instruction::PushClassPrivateGetter {
                object,
                value,
                name_index,
            }
            | Instruction::PushClassPrivateSetter {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassStaticMethodByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassMethodByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassStaticGetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassGetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassStaticSetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::DefineClassSetterByName {
                object,
                value,
                name_index,
            }
            | Instruction::SetPrivateField {
                object,
                value,
                name_index,
            }
            | Instruction::PushClassFieldPrivate {
                object,
                value,
                name_index,
            } => {
                format!(
                    "object:{}, value:{}, name_index:{}",
                    object.value(),
                    value.value(),
                    name_index.value()
                )
            }
            Instruction::GetPrivateField {
                dst,
                object,
                name_index,
            } => {
                format!(
                    "dst:{}, object:{}, name_index:{}",
                    dst.value(),
                    object.value(),
                    name_index.value()
                )
            }
            Instruction::PushClassPrivateMethod {
                object,
                proto,
                value,
                name_index,
            } => {
                format!(
                    "object:{}, proto:{}, value:{}, name_index:{}",
                    object.value(),
                    proto.value(),
                    value.value(),
                    name_index.value()
                )
            }
            Instruction::ThrowMutateImmutable { index } => {
                format!("index:{}", index.value())
            }

            Instruction::DeletePropertyByName { object, name_index } => {
                format!(
                    "object:{}, name_index:{}",
                    object.value(),
                    name_index.value()
                )
            }
            Instruction::GetPropertyByName {
                dst,
                receiver,
                value,
                ic_index,
            } => {
                let ic = &self.ic[ic_index.value() as usize];
                format!(
                    "dst:{}, receiver:{}, value:{}, ic:shape:0x{:x}",
                    dst.value(),
                    receiver.value(),
                    value.value(),
                    ic.shape.borrow().to_addr_usize(),
                )
            }
            Instruction::SetPropertyByName {
                value,
                receiver,
                object,
                ic_index,
            } => {
                let ic = &self.ic[ic_index.value() as usize];
                format!(
                    "object:{}, receiver:{}, value:{}, ic:shape:0x{:x}",
                    object.value(),
                    receiver.value(),
                    value.value(),
                    ic.shape.borrow().to_addr_usize(),
                )
            }
            Instruction::GetPropertyByValue {
                dst,
                key,
                receiver,
                object,
            }
            | Instruction::GetPropertyByValuePush {
                dst,
                key,
                receiver,
                object,
            } => {
                format!(
                    "dst:{}, object:{}, receiver:{}, key:{}",
                    dst.value(),
                    object.value(),
                    receiver.value(),
                    key.value(),
                )
            }
            Instruction::SetPropertyByValue {
                value,
                key,
                receiver,
                object,
            } => {
                format!(
                    "object:{}, receiver:{}, key:{}, value:{}",
                    object.value(),
                    receiver.value(),
                    key.value(),
                    value.value(),
                )
            }
            Instruction::DefineOwnPropertyByValue { value, key, object }
            | Instruction::DefineClassStaticMethodByValue { value, key, object }
            | Instruction::DefineClassMethodByValue { value, key, object }
            | Instruction::SetPropertyGetterByValue { value, key, object }
            | Instruction::DefineClassStaticGetterByValue { value, key, object }
            | Instruction::DefineClassGetterByValue { value, key, object }
            | Instruction::SetPropertySetterByValue { value, key, object }
            | Instruction::DefineClassStaticSetterByValue { value, key, object }
            | Instruction::DefineClassSetterByValue { value, key, object } => {
                format!(
                    "object:{}, key:{}, value:{}",
                    object.value(),
                    key.value(),
                    value.value()
                )
            }
            Instruction::DeletePropertyByValue { key, object } => {
                format!("object:{}, key:{}", object.value(), key.value())
            }
            Instruction::CreateIteratorResult { value, done } => {
                format!("value:{}, done:{}", value.value(), done)
            }
            Instruction::PushClassPrototype {
                dst,
                class,
                superclass,
            } => {
                format!(
                    "dst:{}, class:{}, superclass:{}",
                    dst.value(),
                    class.value(),
                    superclass.value(),
                )
            }
            Instruction::SetClassPrototype {
                dst,
                prototype,
                class,
            } => {
                format!(
                    "dst:{}, prototype:{}, class:{}",
                    dst.value(),
                    prototype.value(),
                    class.value()
                )
            }
            Instruction::SetHomeObject { function, home } => {
                format!("function:{}, home:{}", function.value(), home.value())
            }
            Instruction::SetPrototype { object, prototype } => {
                format!("object:{}, prototype:{}", object.value(), prototype.value())
            }
            Instruction::PushValueToArray { value, array } => {
                format!("value:{}, array:{}", value.value(), array.value())
            }
            Instruction::PushElisionToArray { array }
            | Instruction::PushIteratorToArray { array } => {
                format!("array:{}", array.value())
            }
            Instruction::TypeOf { value }
            | Instruction::LogicalNot { value }
            | Instruction::Pos { value }
            | Instruction::Neg { value }
            | Instruction::IsObject { value }
            | Instruction::ImportCall { value }
            | Instruction::BindThisValue { value } => {
                format!("value:{}", value.value())
            }
            Instruction::PushClassField {
                object,
                name_index,
                value,
                is_anonymous_function,
            } => {
                format!(
                    "object:{}, value:{}, name_index:{}, is_anonymous_function:{}",
                    object.value(),
                    value.value(),
                    name_index.value(),
                    is_anonymous_function
                )
            }
            Instruction::MaybeException {
                has_exception,
                exception,
            } => {
                format!(
                    "has_exception:{}, exception:{}",
                    has_exception.value(),
                    exception.value()
                )
            }
            Instruction::SetAccumulator { src }
            | Instruction::PushFromRegister { src }
            | Instruction::Throw { src }
            | Instruction::SetNameByLocator { src }
            | Instruction::PushObjectEnvironment { src }
            | Instruction::CreateForInIterator { src }
            | Instruction::GetIterator { src }
            | Instruction::GetAsyncIterator { src }
            | Instruction::ValueNotNullOrUndefined { src }
            | Instruction::GeneratorYield { src }
            | Instruction::AsyncGeneratorYield { src }
            | Instruction::Await { src } => {
                format!("src:{}", src.value())
            }
            Instruction::IteratorDone { dst }
            | Instruction::IteratorValue { dst }
            | Instruction::IteratorResult { dst }
            | Instruction::IteratorToArray { dst }
            | Instruction::IteratorStackEmpty { dst }
            | Instruction::PushEmptyObject { dst } => {
                format!("dst:{}", dst.value())
            }
            Instruction::IteratorFinishAsyncNext { resume_kind, value }
            | Instruction::GeneratorNext { resume_kind, value } => {
                format!(
                    "resume_kind:{}, value:{}",
                    resume_kind.value(),
                    value.value()
                )
            }
            Instruction::IteratorReturn { value, called } => {
                format!("value:{}, called:{}", value.value(), called.value())
            }
            Instruction::JumpIfNotResumeKind {
                address,
                resume_kind,
                src,
            } => {
                format!(
                    "address:{}, resume_kind:{}, src:{}",
                    address,
                    *resume_kind as u8,
                    src.value()
                )
            }
            Instruction::CreateGlobalFunctionBinding {
                src,
                configurable,
                name_index,
            } => {
                format!(
                    "src:{}, configurable:{}, name_index:{}",
                    src.value(),
                    configurable,
                    name_index.value()
                )
            }
            Instruction::CreateGlobalVarBinding {
                configurable,
                name_index,
            } => {
                format!(
                    "configurable:{}, name_index:{}",
                    configurable,
                    name_index.value()
                )
            }
            Instruction::PushPrivateEnvironment {
                class,
                name_indices,
            } => {
                format!("class:{}, names:{name_indices:?}", class.value())
            }
            Instruction::TemplateLookup { address, site, dst } => {
                format!("address:{}, site:{}, dst:{}", address, site, dst.value())
            }
            Instruction::JumpTable { default, addresses } => {
                let mut operands = format!("#{}: Default: {default:4}", addresses.len());
                for (i, address) in addresses.iter().enumerate() {
                    operands += &format!(", {i}: {address}");
                }
                operands
            }
            Instruction::ConcatToString { dst, values } => {
                format!("dst:{}, values:{values:?}", dst.value())
            }
            Instruction::CopyDataProperties {
                object,
                source,
                excluded_keys,
            } => {
                format!(
                    "object:{}, source:{}, excluded_keys:{excluded_keys:?}",
                    object.value(),
                    source.value()
                )
            }
            Instruction::TemplateCreate { site, dst, values } => {
                format!("site:{}, dst:{}, values:{values:?}", site, dst.value())
            }
            Instruction::Pop
            | Instruction::DeleteSuperThrow
            | Instruction::ReThrow
            | Instruction::CheckReturn
            | Instruction::Return
            | Instruction::AsyncGeneratorClose
            | Instruction::CreatePromiseCapability
            | Instruction::CompletePromiseCapability
            | Instruction::PopEnvironment
            | Instruction::IncrementLoopIteration
            | Instruction::IteratorNext
            | Instruction::SuperCallDerived
            | Instruction::CallSpread
            | Instruction::NewSpread
            | Instruction::SuperCallSpread
            | Instruction::PopPrivateEnvironment => String::new(),
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
            | Instruction::Reserved49
            | Instruction::Reserved50
            | Instruction::Reserved51
            | Instruction::Reserved52
            | Instruction::Reserved53
            | Instruction::Reserved54
            | Instruction::Reserved55
            | Instruction::Reserved56
            | Instruction::Reserved57
            | Instruction::Reserved58
            | Instruction::Reserved59
            | Instruction::Reserved60 => unreachable!("Reserved opcodes are unreachable"),
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
                    Constant::Scope(v) => {
                        writeln!(
                            f,
                            "[SCOPE] index: {}, bindings: {}",
                            v.scope_index(),
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
                writeln!(
                    f,
                    "    {i:04}: Range: [{:04}, {:04}): Handler: {:04}, Environment: {:02}",
                    handler.start,
                    handler.end,
                    handler.handler(),
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
