//! `CodeBlock`
//!
//! This module is for the `CodeBlock` which implements a function representation in the VM

use crate::{
    Context, JsBigInt, JsString, JsValue, SpannedSourceText,
    builtins::{
        OrdinaryObject,
        function::{OrdinaryFunction, ThisMode},
    },
    object::JsObject,
};
use bitflags::bitflags;
use boa_ast::scope::{BindingLocator, Scope};
use boa_gc::{Finalize, Gc, Trace, empty_trace};
use std::{cell::Cell, fmt::Display, fmt::Write as _};
use thin_vec::ThinVec;

use super::{
    InlineCache,
    opcode::{Bytecode, InstructionIterator},
    operands::{Address, OperandsShape},
    source_info::{SourceInfo, SourceMap, SourcePath},
};

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

impl CodeBlockFlags {
    /// Check if the [`CodeBlock`] has a function scope.
    #[must_use]
    pub(crate) fn has_function_scope(self) -> bool {
        self.contains(Self::HAS_FUNCTION_SCOPE)
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
/// If any exception happens and gets caught by this handler, the `pc` will be set to `end` of the
/// [`Handler`] and remove any environments or stack values that where pushed after the handler.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Handler {
    pub(crate) start: Address,
    pub(crate) end: Address,
    pub(crate) environment_count: u32,
}

impl Handler {
    /// Get the handler address.
    pub(crate) const fn handler(&self) -> Address {
        self.end
    }

    /// Check if the provided `pc` is contained in the handler range.
    pub(crate) const fn contains(&self, pc: u32) -> bool {
        pc < self.end.as_u32() && pc >= self.start.as_u32()
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

/// Binding information about a global function.
#[derive(Copy, Clone, Debug, Trace, Finalize)]
#[boa_gc(empty_trace)]
pub(crate) struct GlobalFunctionBinding {
    /// The index of the global function's name in the constants array.
    pub(crate) name_index: u32,
    /// The index of the global function in the constants array
    pub(crate) function_index: u32,
}

/// The internal representation of a JavaScript function.
///
/// A `CodeBlock` is generated for each function compiled by the
/// [`ByteCompiler`](crate::bytecompiler::ByteCompiler). It stores the bytecode and the other
/// attributes of the function.
#[derive(Clone, Debug, Trace, Finalize)]
pub struct CodeBlock {
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
    pub(crate) bytecode: Bytecode,

    pub(crate) constants: ThinVec<Constant>,

    /// Locators for all bindings in the codeblock.
    #[unsafe_ignore_trace]
    pub(crate) bindings: Box<[BindingLocator]>,

    /// Exception [`Handler`]s.
    #[unsafe_ignore_trace]
    pub(crate) handlers: ThinVec<Handler>,

    /// inline caching
    pub(crate) ic: Box<[InlineCache]>,

    /// Bytecode to source code mapping.
    pub(crate) source_info: SourceInfo,

    pub(crate) global_lexs: Box<[u32]>,
    pub(crate) global_fns: Box<[GlobalFunctionBinding]>,
    pub(crate) global_vars: Box<[u32]>,

    // Used for identifying anonymous functions in compiled output and call frames.
    pub(crate) debug_id: u64,

    #[cfg(feature = "trace")]
    #[unsafe_ignore_trace]
    pub(crate) traced: Cell<bool>,
}

/// ---- `CodeBlock` public API ----
impl CodeBlock {
    /// Creates a new `CodeBlock`.
    #[must_use]
    pub fn new(name: JsString, length: u32, strict: bool) -> Self {
        let mut flags = CodeBlockFlags::empty();
        flags.set(CodeBlockFlags::STRICT, strict);
        Self {
            bytecode: Bytecode::default(),
            constants: ThinVec::default(),
            bindings: Box::default(),
            flags: Cell::new(flags),
            length,
            register_count: 0,
            this_mode: ThisMode::Global,
            mapped_arguments_binding_indices: ThinVec::new(),
            parameter_length: 0,
            handlers: ThinVec::default(),
            ic: Box::default(),
            source_info: SourceInfo::new(
                SourceMap::new(Box::default(), SourcePath::None),
                name,
                SpannedSourceText::new_empty(),
            ),
            global_lexs: Box::default(),
            global_fns: Box::default(),
            global_vars: Box::default(),
            debug_id: CodeBlock::get_next_codeblock_id(),
            #[cfg(feature = "trace")]
            traced: Cell::new(false),
        }
    }

    /// Retrieves the name associated with this code block.
    #[must_use]
    pub fn name(&self) -> &JsString {
        self.source_info.function_name()
    }

    /// Retrieves the path of this code block.
    #[must_use]
    pub fn path(&self) -> &SourcePath {
        self.source_info.map().path()
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
        self.flags.get().has_function_scope()
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

    pub(crate) fn source_info(&self) -> &SourceInfo {
        &self.source_info
    }

    pub(crate) fn get_next_codeblock_id() -> u64 {
        thread_local! {
            static CODEBLOCK_ID_COUNTER: Cell<u64> = const { Cell::new(0) };
        }

        CODEBLOCK_ID_COUNTER.with(|c| {
            let id = c.get();
            c.set(id + 1);
            id
        })
    }
}

impl Display for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();
        writeln!(
            f,
            "{:-^80}",
            format!(
                " Compiled Output: {} ",
                if name.is_empty() {
                    format!("[anon#{}]", self.debug_id)
                } else {
                    format!("'{}'", name.to_std_string_escaped())
                }
            ),
        )?;
        writeln!(
            f,
            "Location     Handler      Opcode                            Operands"
        )?;
        let mut iterator = InstructionIterator::new(&self.bytecode);
        while let Some((instruction_start_pc, opcode, instruction)) = iterator.next() {
            let opcode = opcode.as_str();
            let operands = OperandsShape::from_instruction(&instruction);
            let pc = iterator.pc();
            let handler = if let Some((i, handler)) = self.find_handler(instruction_start_pc as u32)
            {
                let border_char = if instruction_start_pc as u32 == u32::from(handler.start) {
                    '>'
                } else if pc as u32 == u32::from(handler.end) {
                    '<'
                } else {
                    ' '
                };
                format!("{border_char}{i:2}: {}", handler.handler())
            } else {
                "           ".to_string()
            };
            writeln!(
                f,
                "  {instruction_start_pc:>06x}    {handler}     {opcode:<32}  {operands}",
            )?;
        }
        writeln!(
            f,
            "\nRegister Count: {}, Flags: {:?}",
            self.register_count,
            self.flags.get()
        )?;
        f.write_str("Constants:")?;
        if self.constants.is_empty() {
            f.write_str(" <empty>\n")?;
        } else {
            f.write_char('\n')?;
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
                        "[FUNCTION] name: '{}' (length: {})",
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
        f.write_str("Bindings:")?;
        if self.bindings.is_empty() {
            f.write_str(" <empty>\n")?;
        } else {
            f.write_char('\n')?;
            for (i, binding_locator) in self.bindings.iter().enumerate() {
                writeln!(
                    f,
                    "    {i:04}: {}, scope: {:?}",
                    binding_locator.name().to_std_string_escaped(),
                    binding_locator.scope()
                )?;
            }
        }
        f.write_str("Handlers:")?;
        if self.handlers.is_empty() {
            f.write_str(" <empty>\n")?;
        } else {
            f.write_char('\n')?;
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
        f.write_str("Source Map:")?;
        if self.source_info().map().entries().is_empty() {
            f.write_str(" <empty>\n")?;
        } else {
            f.write_char('\n')?;

            let bytecode_len = self.bytecode.bytes.len() as u32;
            for (i, handler) in self.source_info().map().entries().windows(2).enumerate() {
                let current = handler[0];
                let next = handler.get(1);

                write!(
                    f,
                    "    {i:04}: {:?}: ",
                    current.pc..next.map_or(bytecode_len, |entry| entry.pc),
                )?;

                if let Some(position) = current.position {
                    writeln!(
                        f,
                        "({}, {})",
                        position.line_number(),
                        position.column_number()
                    )?;
                } else {
                    f.write_str("unknown")?;
                }
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
    let name: JsValue = code.name().clone().into();
    let length: JsValue = code.length.into();

    let script_or_module = context.get_active_script_or_module();

    let is_async = code.is_async();
    let is_generator = code.is_generator();
    let function = OrdinaryFunction::new(
        code,
        context.vm.frame().environments.snapshot_for_closure(),
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
/// This is preferred over [`create_function_object`] if prototype is [`None`],
/// because it constructs the function from a pre-initialized object template,
/// with all the properties and prototype set.
pub(crate) fn create_function_object_fast(code: Gc<CodeBlock>, context: &mut Context) -> JsObject {
    let name: JsValue = code.name().clone().into();
    let length: JsValue = code.length.into();

    let script_or_module = context.get_active_script_or_module();

    let is_async = code.is_async();
    let is_generator = code.is_generator();
    let has_prototype_property = code.has_prototype_property();
    let function = OrdinaryFunction::new(
        code,
        context.vm.frame().environments.snapshot_for_closure(),
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
