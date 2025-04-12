//! This module contains the bytecode compiler.

mod class;
mod declaration;
mod declarations;
mod env;
mod expression;
mod function;
mod jump_control;
mod module;
mod register;
mod statement;
mod utils;

use std::cell::Cell;

use crate::{
    builtins::function::{arguments::MappedArguments, ThisMode},
    js_string,
    vm::{
        BindingOpcode, CallFrame, CodeBlock, CodeBlockFlags, Constant, GeneratorResumeKind,
        Handler, InlineCache, Opcode, VaryingOperandKind,
    },
    JsBigInt, JsStr, JsString, SourceText, SpannedSourceText,
};
use boa_ast::{
    declaration::{Binding, LexicalDeclaration, VarDeclaration},
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        literal::ObjectMethodDefinition,
        operator::{assign::AssignTarget, update::UpdateTarget},
        Call, Identifier, New, Optional, OptionalOperationKind,
    },
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunctionDeclaration, AsyncFunctionExpression,
        AsyncGeneratorDeclaration, AsyncGeneratorExpression, ClassMethodDefinition,
        FormalParameterList, FunctionBody, FunctionDeclaration, FunctionExpression,
        GeneratorDeclaration, GeneratorExpression, PrivateName,
    },
    operations::returns_value,
    pattern::Pattern,
    property::MethodDefinitionKind,
    scope::{BindingLocator, BindingLocatorError, FunctionScopes, IdentifierReference, Scope},
    Declaration, Expression, LinearSpan, Statement, StatementList, StatementListItem,
};
use boa_gc::Gc;
use boa_interner::{Interner, Sym};
use boa_macros::js_str;
use rustc_hash::FxHashMap;
use thin_vec::ThinVec;

pub(crate) use declarations::{
    eval_declaration_instantiation_context, global_declaration_instantiation_context,
};
pub(crate) use function::FunctionCompiler;
pub(crate) use jump_control::JumpControlInfo;
pub(crate) use register::*;

pub(crate) trait ToJsString {
    fn to_js_string(&self, interner: &Interner) -> JsString;
}

impl ToJsString for Sym {
    fn to_js_string(&self, interner: &Interner) -> JsString {
        // TODO: Identify latin1 encodeable strings during parsing to avoid this check.
        let string = interner.resolve_expect(*self).utf16();
        for c in string {
            if u8::try_from(*c).is_err() {
                return js_string!(string);
            }
        }
        let string = string.iter().map(|c| *c as u8).collect::<Vec<_>>();
        js_string!(JsStr::latin1(&string))
    }
}

impl ToJsString for Identifier {
    fn to_js_string(&self, interner: &Interner) -> JsString {
        self.sym().to_js_string(interner)
    }
}

/// Describes how a node has been defined in the source code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NodeKind {
    Declaration,
    Expression,
}

/// Describes the type of a function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FunctionKind {
    Ordinary,
    Arrow,
    AsyncArrow,
    Async,
    Generator,
    AsyncGenerator,
}

impl FunctionKind {
    pub(crate) const fn is_arrow(self) -> bool {
        matches!(self, Self::Arrow | Self::AsyncArrow)
    }

    pub(crate) const fn is_async(self) -> bool {
        matches!(self, Self::Async | Self::AsyncGenerator | Self::AsyncArrow)
    }

    pub(crate) const fn is_generator(self) -> bool {
        matches!(self, Self::Generator | Self::AsyncGenerator)
    }
}

/// Describes the complete specification of a function node.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FunctionSpec<'a> {
    pub(crate) kind: FunctionKind,
    pub(crate) name: Option<Identifier>,
    parameters: &'a FormalParameterList,
    body: &'a FunctionBody,
    pub(crate) scopes: &'a FunctionScopes,
    pub(crate) name_scope: Option<&'a Scope>,
    linear_span: Option<LinearSpan>,
    pub(crate) contains_direct_eval: bool,
}

impl PartialEq for FunctionSpec<'_> {
    fn eq(&self, other: &Self) -> bool {
        // all fields except `linear_span`
        self.kind == other.kind
            && self.name == other.name
            && self.parameters == other.parameters
            && self.body == other.body
            && self.scopes == other.scopes
            && self.name_scope == other.name_scope
    }
}

impl<'a> From<&'a FunctionDeclaration> for FunctionSpec<'a> {
    fn from(function: &'a FunctionDeclaration) -> Self {
        FunctionSpec {
            kind: FunctionKind::Ordinary,
            name: Some(function.name()),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a GeneratorDeclaration> for FunctionSpec<'a> {
    fn from(function: &'a GeneratorDeclaration) -> Self {
        FunctionSpec {
            kind: FunctionKind::Generator,
            name: Some(function.name()),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a AsyncFunctionDeclaration> for FunctionSpec<'a> {
    fn from(function: &'a AsyncFunctionDeclaration) -> Self {
        FunctionSpec {
            kind: FunctionKind::Async,
            name: Some(function.name()),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a AsyncGeneratorDeclaration> for FunctionSpec<'a> {
    fn from(function: &'a AsyncGeneratorDeclaration) -> Self {
        FunctionSpec {
            kind: FunctionKind::AsyncGenerator,
            name: Some(function.name()),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a FunctionExpression> for FunctionSpec<'a> {
    fn from(function: &'a FunctionExpression) -> Self {
        FunctionSpec {
            kind: FunctionKind::Ordinary,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: function.name_scope(),
            linear_span: function.linear_span(),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a ArrowFunction> for FunctionSpec<'a> {
    fn from(function: &'a ArrowFunction) -> Self {
        FunctionSpec {
            kind: FunctionKind::Arrow,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a AsyncArrowFunction> for FunctionSpec<'a> {
    fn from(function: &'a AsyncArrowFunction) -> Self {
        FunctionSpec {
            kind: FunctionKind::AsyncArrow,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: None,
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a AsyncFunctionExpression> for FunctionSpec<'a> {
    fn from(function: &'a AsyncFunctionExpression) -> Self {
        FunctionSpec {
            kind: FunctionKind::Async,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: function.name_scope(),
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a GeneratorExpression> for FunctionSpec<'a> {
    fn from(function: &'a GeneratorExpression) -> Self {
        FunctionSpec {
            kind: FunctionKind::Generator,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: function.name_scope(),
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a AsyncGeneratorExpression> for FunctionSpec<'a> {
    fn from(function: &'a AsyncGeneratorExpression) -> Self {
        FunctionSpec {
            kind: FunctionKind::AsyncGenerator,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            scopes: function.scopes(),
            name_scope: function.name_scope(),
            linear_span: Some(function.linear_span()),
            contains_direct_eval: function.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a ClassMethodDefinition> for FunctionSpec<'a> {
    fn from(method: &'a ClassMethodDefinition) -> Self {
        let kind = match method.kind() {
            MethodDefinitionKind::Generator => FunctionKind::Generator,
            MethodDefinitionKind::AsyncGenerator => FunctionKind::AsyncGenerator,
            MethodDefinitionKind::Async => FunctionKind::Async,
            _ => FunctionKind::Ordinary,
        };

        FunctionSpec {
            kind,
            name: None,
            parameters: method.parameters(),
            body: method.body(),
            scopes: method.scopes(),
            name_scope: None,
            linear_span: Some(method.linear_span()),
            contains_direct_eval: method.contains_direct_eval(),
        }
    }
}

impl<'a> From<&'a ObjectMethodDefinition> for FunctionSpec<'a> {
    fn from(method: &'a ObjectMethodDefinition) -> Self {
        let kind = match method.kind() {
            MethodDefinitionKind::Generator => FunctionKind::Generator,
            MethodDefinitionKind::AsyncGenerator => FunctionKind::AsyncGenerator,
            MethodDefinitionKind::Async => FunctionKind::Async,
            _ => FunctionKind::Ordinary,
        };

        FunctionSpec {
            kind,
            name: method.name().literal().map(Into::into),
            parameters: method.parameters(),
            body: method.body(),
            scopes: method.scopes(),
            name_scope: None,
            linear_span: Some(method.linear_span()),
            contains_direct_eval: method.contains_direct_eval(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MethodKind {
    Get,
    Set,
    Ordinary,
}

/// Represents a callable expression, like `f()` or `new Cl()`
#[derive(Debug, Clone, Copy)]
enum Callable<'a> {
    Call(&'a Call),
    New(&'a New),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Literal {
    String(JsString),
    BigInt(JsBigInt),
}

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Label {
    index: u32,
}

#[derive(Debug, Clone, Copy)]
enum Access<'a> {
    Variable { name: Identifier },
    Property { access: &'a PropertyAccess },
    This,
}

impl Access<'_> {
    const fn from_assign_target(target: &AssignTarget) -> Result<Access<'_>, &Pattern> {
        match target {
            AssignTarget::Identifier(ident) => Ok(Access::Variable { name: *ident }),
            AssignTarget::Access(access) => Ok(Access::Property { access }),
            AssignTarget::Pattern(pat) => Err(pat),
        }
    }

    const fn from_expression(expr: &Expression) -> Option<Access<'_>> {
        match expr {
            Expression::Identifier(name) => Some(Access::Variable { name: *name }),
            Expression::PropertyAccess(access) => Some(Access::Property { access }),
            Expression::This => Some(Access::This),
            Expression::Parenthesized(expr) => Self::from_expression(expr.expression()),
            _ => None,
        }
    }

    const fn from_update_target(target: &UpdateTarget) -> Access<'_> {
        match target {
            UpdateTarget::Identifier(name) => Access::Variable { name: *name },
            UpdateTarget::PropertyAccess(access) => Access::Property { access },
        }
    }
}

/// An opcode operand.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Operand<'a> {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    #[allow(unused)]
    U16(u16),
    I32(i32),
    U32(u32),
    #[allow(unused)]
    I64(i64),
    U64(u64),

    Varying(u32),
    Register(&'a Register),
}

/// The [`ByteCompiler`] is used to compile ECMAScript AST from [`boa_ast`] to bytecode.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct ByteCompiler<'ctx> {
    /// Name of this function.
    pub(crate) function_name: JsString,

    /// The number of arguments expected.
    pub(crate) length: u32,

    pub(crate) register_allocator: RegisterAllocator,

    /// `[[ThisMode]]`
    pub(crate) this_mode: ThisMode,

    /// Parameters passed to this function.
    pub(crate) params: FormalParameterList,

    /// Scope of the function parameters.
    pub(crate) parameter_scope: Scope,

    /// Bytecode
    pub(crate) bytecode: Vec<u8>,

    pub(crate) constants: ThinVec<Constant>,

    /// Locators for all bindings in the codeblock.
    pub(crate) bindings: Vec<BindingLocator>,

    pub(crate) local_binding_registers: FxHashMap<IdentifierReference, u32>,

    /// The current variable scope.
    pub(crate) variable_scope: Scope,

    /// The current lexical scope.
    pub(crate) lexical_scope: Scope,

    pub(crate) current_open_environments_count: u32,
    code_block_flags: CodeBlockFlags,
    handlers: ThinVec<Handler>,
    pub(crate) ic: Vec<InlineCache>,
    literals_map: FxHashMap<Literal, u32>,
    names_map: FxHashMap<Identifier, u32>,
    bindings_map: FxHashMap<BindingLocator, u32>,
    jump_info: Vec<JumpControlInfo>,

    /// Used to handle exception throws that escape the async function types.
    ///
    /// Async functions and async generator functions, need to be closed and resolved.
    pub(crate) async_handler: Option<u32>,
    json_parse: bool,

    /// Whether the function is in a `with` statement.
    pub(crate) in_with: bool,

    /// Used to determine if a we emited a `CreateUnmappedArgumentsObject` opcode
    pub(crate) emitted_mapped_arguments_object_opcode: bool,

    pub(crate) interner: &'ctx mut Interner,
    spanned_source_text: SpannedSourceText,

    #[cfg(feature = "annex-b")]
    pub(crate) annex_b_function_names: Vec<Identifier>,
}

pub(crate) enum BindingKind {
    Stack(u32),
    Local(u32),
    Global(u32),
}

impl<'ctx> ByteCompiler<'ctx> {
    /// Represents a placeholder address that will be patched later.
    const DUMMY_ADDRESS: u32 = u32::MAX;
    const DUMMY_LABEL: Label = Label { index: u32::MAX };

    /// Creates a new [`ByteCompiler`].
    #[inline]
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::fn_params_excessive_bools)]
    pub(crate) fn new(
        name: JsString,
        strict: bool,
        json_parse: bool,
        variable_scope: Scope,
        lexical_scope: Scope,
        is_async: bool,
        is_generator: bool,
        interner: &'ctx mut Interner,
        in_with: bool,
        spanned_source_text: SpannedSourceText,
    ) -> ByteCompiler<'ctx> {
        let mut code_block_flags = CodeBlockFlags::empty();
        code_block_flags.set(CodeBlockFlags::STRICT, strict);
        code_block_flags.set(CodeBlockFlags::IS_ASYNC, is_async);
        code_block_flags.set(CodeBlockFlags::IS_GENERATOR, is_generator);
        code_block_flags |= CodeBlockFlags::HAS_PROTOTYPE_PROPERTY;

        let mut register_allocator = RegisterAllocator::default();
        if is_async {
            let promise_register = register_allocator.alloc_persistent();
            let resolve_register = register_allocator.alloc_persistent();
            let reject_register = register_allocator.alloc_persistent();

            debug_assert_eq!(
                promise_register.index(),
                CallFrame::PROMISE_CAPABILITY_PROMISE_REGISTER_INDEX
            );
            debug_assert_eq!(
                resolve_register.index(),
                CallFrame::PROMISE_CAPABILITY_RESOLVE_REGISTER_INDEX
            );
            debug_assert_eq!(
                reject_register.index(),
                CallFrame::PROMISE_CAPABILITY_REJECT_REGISTER_INDEX
            );

            if is_generator {
                let async_function_object_register = register_allocator.alloc_persistent();
                debug_assert_eq!(
                    async_function_object_register.index(),
                    CallFrame::ASYNC_GENERATOR_OBJECT_REGISTER_INDEX
                );
            }
        }

        Self {
            function_name: name,
            length: 0,
            bytecode: Vec::default(),
            constants: ThinVec::default(),
            bindings: Vec::default(),
            local_binding_registers: FxHashMap::default(),
            this_mode: ThisMode::Global,
            params: FormalParameterList::default(),
            parameter_scope: Scope::default(),
            current_open_environments_count: 0,

            register_allocator,
            code_block_flags,
            handlers: ThinVec::default(),
            ic: Vec::default(),

            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            bindings_map: FxHashMap::default(),
            jump_info: Vec::new(),
            async_handler: None,
            json_parse,
            variable_scope,
            lexical_scope,
            interner,
            spanned_source_text,

            #[cfg(feature = "annex-b")]
            annex_b_function_names: Vec::new(),
            in_with,
            emitted_mapped_arguments_object_opcode: false,
        }
    }

    pub(crate) fn source_text(&self) -> SourceText {
        self.spanned_source_text.source_text()
    }

    pub(crate) const fn strict(&self) -> bool {
        self.code_block_flags.contains(CodeBlockFlags::STRICT)
    }

    pub(crate) const fn is_async(&self) -> bool {
        self.code_block_flags.contains(CodeBlockFlags::IS_ASYNC)
    }

    pub(crate) const fn is_generator(&self) -> bool {
        self.code_block_flags.contains(CodeBlockFlags::IS_GENERATOR)
    }

    pub(crate) const fn is_async_generator(&self) -> bool {
        self.is_async() && self.is_generator()
    }

    pub(crate) fn interner(&self) -> &Interner {
        self.interner
    }

    fn get_or_insert_literal(&mut self, literal: Literal) -> u32 {
        if let Some(index) = self.literals_map.get(&literal) {
            return *index;
        }

        let value = match literal.clone() {
            Literal::String(value) => Constant::String(value),
            Literal::BigInt(value) => Constant::BigInt(value),
        };

        let index = self.constants.len() as u32;
        self.constants.push(value);
        self.literals_map.insert(literal, index);
        index
    }

    fn get_or_insert_name(&mut self, name: Identifier) -> u32 {
        if let Some(index) = self.names_map.get(&name) {
            return *index;
        }

        let index = self.constants.len() as u32;
        let string = name.to_js_string(self.interner());
        self.constants.push(Constant::String(string));
        self.names_map.insert(name, index);
        index
    }

    fn get_or_insert_string(&mut self, value: JsString) -> u32 {
        self.get_or_insert_literal(Literal::String(value))
    }

    #[inline]
    fn get_or_insert_private_name(&mut self, name: PrivateName) -> u32 {
        self.get_or_insert_name(Identifier::new(name.description()))
    }

    #[inline]
    pub(crate) fn get_or_insert_binding(&mut self, binding: IdentifierReference) -> BindingKind {
        if binding.is_global_object() {
            if let Some(index) = self.bindings_map.get(&binding.locator()) {
                return BindingKind::Global(*index);
            }

            let index = self.bindings.len() as u32;
            self.bindings.push(binding.locator().clone());
            self.bindings_map.insert(binding.locator(), index);
            return BindingKind::Global(index);
        }

        if binding.local() {
            return BindingKind::Local(
                *self
                    .local_binding_registers
                    .entry(binding)
                    .or_insert_with(|| self.register_allocator.alloc_persistent().index()),
            );
        }

        if let Some(index) = self.bindings_map.get(&binding.locator()) {
            return BindingKind::Stack(*index);
        }

        let index = self.bindings.len() as u32;
        self.bindings.push(binding.locator().clone());
        self.bindings_map.insert(binding.locator(), index);
        BindingKind::Stack(index)
    }

    #[inline]
    #[must_use]
    pub(crate) fn push_function_to_constants(&mut self, function: Gc<CodeBlock>) -> u32 {
        let index = self.constants.len() as u32;
        self.constants.push(Constant::Function(function));
        index
    }

    fn emit_binding(&mut self, opcode: BindingOpcode, name: JsString, value: &Register) {
        match opcode {
            BindingOpcode::Var => {
                let binding = self.variable_scope.get_identifier_reference(name);
                if !binding.locator().is_global() {
                    let index = self.get_or_insert_binding(binding);
                    self.emit_binding_access(Opcode::DefVar, &index, value);
                }
            }
            BindingOpcode::InitVar => match self.lexical_scope.set_mutable_binding(name.clone()) {
                Ok(binding) => {
                    let index = self.get_or_insert_binding(binding);
                    self.emit_binding_access(Opcode::DefInitVar, &index, value);
                }
                Err(BindingLocatorError::MutateImmutable) => {
                    let index = self.get_or_insert_string(name);
                    self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                }
                Err(BindingLocatorError::Silent) => {}
            },
            BindingOpcode::InitLexical => {
                let binding = self.lexical_scope.get_identifier_reference(name);
                let index = self.get_or_insert_binding(binding);
                self.emit_binding_access(Opcode::PutLexicalValue, &index, value);
            }
            BindingOpcode::SetName => match self.lexical_scope.set_mutable_binding(name.clone()) {
                Ok(binding) => {
                    let index = self.get_or_insert_binding(binding);
                    self.emit_binding_access(Opcode::SetName, &index, value);
                }
                Err(BindingLocatorError::MutateImmutable) => {
                    let index = self.get_or_insert_string(name);
                    self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                }
                Err(BindingLocatorError::Silent) => {}
            },
        }
    }

    fn next_opcode_location(&mut self) -> u32 {
        assert!(self.bytecode.len() < u32::MAX as usize);
        self.bytecode.len() as u32
    }

    pub(crate) fn emit(&mut self, opcode: Opcode, operands: &[Operand<'_>]) {
        let mut varying_kind = VaryingOperandKind::U8;
        for operand in operands {
            match *operand {
                Operand::Register(operand) => {
                    if u8::try_from(operand.index()).is_ok() {
                    } else if u16::try_from(operand.index()).is_ok() {
                        varying_kind = std::cmp::max(varying_kind, VaryingOperandKind::U16);
                    } else {
                        varying_kind = std::cmp::max(varying_kind, VaryingOperandKind::U32);
                    }
                }
                Operand::Varying(operand) => {
                    if u8::try_from(operand).is_ok() {
                    } else if u16::try_from(operand).is_ok() {
                        varying_kind = std::cmp::max(varying_kind, VaryingOperandKind::U16);
                    } else {
                        varying_kind = std::cmp::max(varying_kind, VaryingOperandKind::U32);
                    }
                }
                _ => {}
            }
        }

        match varying_kind {
            VaryingOperandKind::U8 => {}
            VaryingOperandKind::U16 => self.emit_opcode(Opcode::U16Operands),
            VaryingOperandKind::U32 => self.emit_opcode(Opcode::U32Operands),
        }
        self.emit_opcode(opcode);
        for operand in operands {
            self.emit_operand2(*operand, varying_kind);
        }
    }

    /// Emit an opcode with a dummy operand.
    /// Return the `Label` of the operand.
    pub(crate) fn emit_with_label(&mut self, opcode: Opcode, operands: &[Operand<'_>]) -> Label {
        let index = self.next_opcode_location();
        let mut ops = Vec::with_capacity(operands.len() + 1);
        ops.push(Operand::U32(Self::DUMMY_ADDRESS));
        ops.extend_from_slice(operands);
        self.emit(opcode, &ops);
        Label { index }
    }

    pub(crate) fn emit_operand2(&mut self, operand: Operand<'_>, varying_kind: VaryingOperandKind) {
        match operand {
            Operand::Bool(v) => self.emit_u8(v.into()),
            Operand::I8(v) => self.emit_i8(v),
            Operand::U8(v) => self.emit_u8(v),
            Operand::I16(v) => self.emit_i16(v),
            Operand::U16(v) => self.emit_u16(v),
            Operand::I32(v) => self.emit_i32(v),
            Operand::U32(v) => self.emit_u32(v),
            Operand::I64(v) => self.emit_i64(v),
            Operand::U64(v) => self.emit_u64(v),
            Operand::Varying(v) => match varying_kind {
                VaryingOperandKind::U8 => self.emit_u8(v as u8),
                VaryingOperandKind::U16 => self.emit_u16(v as u16),
                VaryingOperandKind::U32 => self.emit_u32(v),
            },
            Operand::Register(reg) => {
                let v = reg.index();
                match varying_kind {
                    VaryingOperandKind::U8 => self.emit_u8(v as u8),
                    VaryingOperandKind::U16 => self.emit_u16(v as u16),
                    VaryingOperandKind::U32 => self.emit_u32(v),
                }
            }
        }
    }

    pub(crate) fn emit_get_function(&mut self, dst: &Register, index: u32) {
        self.emit(
            Opcode::GetFunction,
            &[Operand::Register(dst), Operand::Varying(index)],
        );
    }

    /// TODO: Temporary function, remove once transition is complete.
    fn pop_into_register(&mut self, dst: &Register) {
        self.emit(Opcode::PopIntoRegister, &[Operand::Register(dst)]);
    }
    /// TODO: Temporary function, remove once transition is complete.
    pub(crate) fn push_from_register(&mut self, src: &Register) {
        self.emit(Opcode::PushFromRegister, &[Operand::Register(src)]);
    }

    /// Emits an opcode with one varying operand.
    ///
    /// Simpler version of [`ByteCompiler::emit()`].
    pub(crate) fn emit_with_varying_operand(&mut self, opcode: Opcode, operand: u32) {
        if let Ok(operand) = u8::try_from(operand) {
            self.emit_opcode(opcode);
            self.emit_u8(operand);
        } else if let Ok(operand) = u16::try_from(operand) {
            self.emit_opcode(Opcode::U16Operands);
            self.emit_opcode(opcode);
            self.emit_u16(operand);
        } else {
            self.emit_opcode(Opcode::U32Operands);
            self.emit_opcode(opcode);
            self.emit_u32(operand);
        }
    }

    pub(crate) fn emit_binding_access(
        &mut self,
        opcode: Opcode,
        binding: &BindingKind,
        value: &Register,
    ) {
        match binding {
            BindingKind::Global(index) => match opcode {
                Opcode::SetNameByLocator => self.emit(opcode, &[Operand::Register(value)]),
                Opcode::GetName => {
                    let ic_index = self.ic.len() as u32;
                    let name = self.bindings[*index as usize].name().clone();
                    self.ic.push(InlineCache::new(name));
                    self.emit(
                        Opcode::GetNameGlobal,
                        &[
                            Operand::Register(value),
                            Operand::Varying(*index),
                            Operand::Varying(ic_index),
                        ],
                    );
                }
                Opcode::GetLocator | Opcode::DefVar => {
                    self.emit(opcode, &[Operand::Varying(*index)]);
                }
                _ => self.emit(
                    opcode,
                    &[Operand::Register(value), Operand::Varying(*index)],
                ),
            },
            BindingKind::Stack(index) => match opcode {
                Opcode::SetNameByLocator => self.emit(opcode, &[Operand::Register(value)]),
                Opcode::GetLocator | Opcode::DefVar => {
                    self.emit(opcode, &[Operand::Varying(*index)]);
                }
                _ => self.emit(
                    opcode,
                    &[Operand::Register(value), Operand::Varying(*index)],
                ),
            },
            BindingKind::Local(index) => match opcode {
                Opcode::GetName | Opcode::GetNameOrUndefined | Opcode::GetNameAndLocator => self
                    .emit(
                        Opcode::PushFromLocal,
                        &[Operand::Varying(*index), Operand::Register(value)],
                    ),
                Opcode::GetLocator | Opcode::DefVar => {}
                Opcode::SetName
                | Opcode::DefInitVar
                | Opcode::PutLexicalValue
                | Opcode::SetNameByLocator => self.emit(
                    Opcode::PopIntoLocal,
                    &[Operand::Register(value), Operand::Varying(*index)],
                ),
                Opcode::DeleteName => self.push_false(value),
                _ => unreachable!("invalid opcode for binding access"),
            },
        }
    }

    fn emit_i64(&mut self, value: i64) {
        self.emit_u64(value as u64);
    }

    fn emit_get_property_by_name(
        &mut self,
        dst: &Register,
        receiver: &Register,
        value: &Register,
        ident: Sym,
    ) {
        let ic_index = self.ic.len() as u32;

        let name_index = self.get_or_insert_name(Identifier::new(ident));
        let Constant::String(ref name) = self.constants[name_index as usize].clone() else {
            unreachable!("there should be a string at index")
        };
        self.ic.push(InlineCache::new(name.clone()));

        self.emit(
            Opcode::GetPropertyByName,
            &[
                Operand::Register(dst),
                Operand::Register(receiver),
                Operand::Register(value),
                Operand::Varying(ic_index),
            ],
        );
    }

    fn emit_set_property_by_name(
        &mut self,
        value: &Register,
        receiver: &Register,
        object: &Register,
        ident: Sym,
    ) {
        let ic_index = self.ic.len() as u32;

        let name_index = self.get_or_insert_name(Identifier::new(ident));
        let Constant::String(ref name) = self.constants[name_index as usize].clone() else {
            unreachable!("there should be a string at index")
        };
        self.ic.push(InlineCache::new(name.clone()));

        self.emit(
            Opcode::SetPropertyByName,
            &[
                Operand::Register(value),
                Operand::Register(receiver),
                Operand::Register(object),
                Operand::Varying(ic_index),
            ],
        );
    }

    fn emit_type_error(&mut self, message: &str) {
        let error_msg = self.get_or_insert_literal(Literal::String(js_string!(message)));
        self.emit_with_varying_operand(Opcode::ThrowNewTypeError, error_msg);
    }
    fn emit_syntax_error(&mut self, message: &str) {
        let error_msg = self.get_or_insert_literal(Literal::String(js_string!(message)));
        self.emit_with_varying_operand(Opcode::ThrowNewSyntaxError, error_msg);
    }

    fn emit_u64(&mut self, value: u64) {
        self.bytecode.extend(value.to_ne_bytes());
    }

    fn emit_i32(&mut self, value: i32) {
        self.emit_u32(value as u32);
    }

    fn emit_u32(&mut self, value: u32) {
        self.bytecode.extend(value.to_ne_bytes());
    }

    fn emit_i16(&mut self, value: i16) {
        self.emit_u16(value as u16);
    }

    fn emit_u16(&mut self, value: u16) {
        self.bytecode.extend(value.to_ne_bytes());
    }

    fn emit_i8(&mut self, value: i8) {
        self.emit_u8(value as u8);
    }

    fn emit_u8(&mut self, value: u8) {
        self.bytecode.push(value);
    }

    pub(crate) fn emit_opcode(&mut self, opcode: Opcode) {
        self.emit_u8(opcode as u8);
    }

    fn emit_push_integer(&mut self, value: i32, dst: &Register) {
        match value {
            0 => self.push_zero(dst),
            1 => self.push_one(dst),
            x if i32::from(x as i8) == x => self.push_int8(x as i8, dst),
            x if i32::from(x as i16) == x => self.push_int16(x as i16, dst),
            x => self.push_int32(x, dst),
        }
    }

    fn emit_push_literal(&mut self, literal: Literal, dst: &Register) {
        let index = self.get_or_insert_literal(literal);
        self.emit(
            Opcode::PushLiteral,
            &[Operand::Register(dst), Operand::Varying(index)],
        );
    }

    fn emit_push_rational(&mut self, value: f64, dst: &Register) {
        if value.is_nan() {
            return self.push_nan(dst);
        }

        if value.is_infinite() {
            if value.is_sign_positive() {
                return self.push_positive_infinity(dst);
            }
            return self.push_negative_infinity(dst);
        }

        // Check if the f64 value can fit in an i32.
        if f64::from(value as i32).to_bits() == value.to_bits() {
            self.emit_push_integer(value as i32, dst);
        } else {
            let f32_value = value as f32;

            #[allow(clippy::float_cmp)]
            if f64::from(f32_value) == value {
                self.push_float(f32_value, dst);
            } else {
                self.push_double(value, dst);
            }
        }
    }

    pub(crate) fn push_zero(&mut self, dst: &Register) {
        self.emit(Opcode::PushZero, &[Operand::Register(dst)]);
    }

    pub(crate) fn push_one(&mut self, dst: &Register) {
        self.emit(Opcode::PushOne, &[Operand::Register(dst)]);
    }

    pub(crate) fn push_int8(&mut self, value: i8, dst: &Register) {
        self.emit(
            Opcode::PushInt8,
            &[Operand::Register(dst), Operand::I8(value)],
        );
    }

    pub(crate) fn push_int16(&mut self, value: i16, dst: &Register) {
        self.emit(
            Opcode::PushInt16,
            &[Operand::Register(dst), Operand::I16(value)],
        );
    }

    pub(crate) fn push_int32(&mut self, value: i32, dst: &Register) {
        self.emit(
            Opcode::PushInt32,
            &[Operand::Register(dst), Operand::I32(value)],
        );
    }

    pub(crate) fn push_float(&mut self, value: f32, dst: &Register) {
        self.emit(
            Opcode::PushFloat,
            &[Operand::Register(dst), Operand::U32(value.to_bits())],
        );
    }

    pub(crate) fn push_double(&mut self, value: f64, dst: &Register) {
        self.emit(
            Opcode::PushDouble,
            &[Operand::Register(dst), Operand::U64(value.to_bits())],
        );
    }

    pub(crate) fn push_nan(&mut self, dst: &Register) {
        self.emit(Opcode::PushNaN, &[Operand::Register(dst)]);
    }

    pub(crate) fn push_positive_infinity(&mut self, dst: &Register) {
        self.emit(Opcode::PushPositiveInfinity, &[Operand::Register(dst)]);
    }

    pub(crate) fn push_negative_infinity(&mut self, dst: &Register) {
        self.emit(Opcode::PushNegativeInfinity, &[Operand::Register(dst)]);
    }

    pub(crate) fn push_null(&mut self, dst: &Register) {
        self.emit(Opcode::PushNull, &[Operand::Register(dst)]);
    }

    pub(crate) fn push_true(&mut self, dst: &Register) {
        self.emit(Opcode::PushTrue, &[Operand::Register(dst)]);
    }

    pub(crate) fn push_false(&mut self, dst: &Register) {
        self.emit(Opcode::PushFalse, &[Operand::Register(dst)]);
    }

    pub(crate) fn push_undefined(&mut self, dst: &Register) {
        self.emit(Opcode::PushUndefined, &[Operand::Register(dst)]);
    }

    fn emit_move(&mut self, dst: &Register, src: &Register) {
        self.emit(
            Opcode::Move,
            &[Operand::Register(dst), Operand::Register(src)],
        );
    }

    fn jump(&mut self) -> Label {
        self.emit_with_label(Opcode::Jump, &[])
    }

    pub(crate) fn jump_if_true(&mut self, value: &Register) -> Label {
        let index = self.next_opcode_location();
        self.emit(
            Opcode::JumpIfTrue,
            &[Operand::U32(Self::DUMMY_ADDRESS), Operand::Register(value)],
        );
        Label { index }
    }

    pub(crate) fn jump_if_false(&mut self, value: &Register) -> Label {
        let index = self.next_opcode_location();
        self.emit(
            Opcode::JumpIfFalse,
            &[Operand::U32(Self::DUMMY_ADDRESS), Operand::Register(value)],
        );
        Label { index }
    }

    pub(crate) fn jump_if_null_or_undefined(&mut self, value: &Register) -> Label {
        let index = self.next_opcode_location();
        self.emit(
            Opcode::JumpIfNullOrUndefined,
            &[Operand::U32(Self::DUMMY_ADDRESS), Operand::Register(value)],
        );
        Label { index }
    }

    pub(crate) fn emit_jump_if_not_undefined(&mut self, value: &Register) -> Label {
        let index = self.next_opcode_location();
        self.emit(
            Opcode::JumpIfNotUndefined,
            &[Operand::U32(Self::DUMMY_ADDRESS), Operand::Register(value)],
        );
        Label { index }
    }

    pub(crate) fn case(&mut self, value: &Register, condition: &Register) -> Label {
        let index = self.next_opcode_location();
        self.emit(
            Opcode::Case,
            &[
                Operand::U32(Self::DUMMY_ADDRESS),
                Operand::Register(value),
                Operand::Register(condition),
            ],
        );
        Label { index }
    }

    pub(crate) fn generator_delegate_next(
        &mut self,
        value: &Register,
        resume_kind: &Register,
        is_return: &Register,
    ) -> (Label, Label) {
        let index = self.next_opcode_location();
        self.emit(
            Opcode::GeneratorDelegateNext,
            &[
                Operand::U32(Self::DUMMY_ADDRESS),
                Operand::U32(Self::DUMMY_ADDRESS),
                Operand::Register(value),
                Operand::Register(resume_kind),
                Operand::Register(is_return),
            ],
        );
        (Label { index }, Label { index: index + 4 })
    }

    pub(crate) fn generator_delegate_resume(
        &mut self,
        value: &Register,
        resume_kind: &Register,
        is_return: &Register,
    ) -> (Label, Label) {
        let index = self.next_opcode_location();
        self.emit(
            Opcode::GeneratorDelegateResume,
            &[
                Operand::U32(Self::DUMMY_ADDRESS),
                Operand::U32(Self::DUMMY_ADDRESS),
                Operand::Register(value),
                Operand::Register(resume_kind),
                Operand::Register(is_return),
            ],
        );
        (Label { index }, Label { index: index + 4 })
    }

    pub(crate) fn template_lookup(&mut self, dst: &Register, site: u64) -> Label {
        let index = self.next_opcode_location();
        self.emit(
            Opcode::TemplateLookup,
            &[
                Operand::U32(Self::DUMMY_ADDRESS),
                Operand::U64(site),
                Operand::Register(dst),
            ],
        );
        Label { index }
    }

    fn emit_resume_kind(&mut self, resume_kind: GeneratorResumeKind, dst: &Register) {
        self.emit_push_integer(resume_kind as i32, dst);
    }

    fn jump_if_not_resume_kind(
        &mut self,
        resume_kind: GeneratorResumeKind,
        value: &Register,
    ) -> Label {
        let index = self.next_opcode_location();
        self.emit(
            Opcode::JumpIfNotResumeKind,
            &[
                Operand::U32(Self::DUMMY_ADDRESS),
                Operand::U8(resume_kind as u8),
                Operand::Register(value),
            ],
        );
        Label { index }
    }

    /// Push a jump table with `count` of entries.
    ///
    /// Returns the jump label entries and the default label.
    fn jump_table(&mut self, count: u32) -> (Vec<Label>, Label) {
        let index = self.next_opcode_location();
        self.emit(
            Opcode::JumpTable,
            &[Operand::U32(Self::DUMMY_ADDRESS), Operand::U32(count)],
        );
        let default = Label { index };
        let mut labels = Vec::with_capacity(count as usize);
        for i in 0..count {
            labels.push(Label {
                index: index + 8 + 4 * i,
            });
            self.emit_u32(Self::DUMMY_ADDRESS);
        }

        (labels, default)
    }

    pub(crate) fn emit_push_private_environment(&mut self, class: &Register) -> Label {
        self.emit(Opcode::PushPrivateEnvironment, &[Operand::Register(class)]);
        let index = self.next_opcode_location();
        self.emit_u32(Self::DUMMY_ADDRESS);
        Label { index: index - 1 }
    }

    #[track_caller]
    pub(crate) fn patch_jump_with_target(&mut self, label: Label, target: u32) {
        const U32_SIZE: usize = size_of::<u32>();

        let Label { index } = label;

        let index = index as usize;
        let bytes = target.to_ne_bytes();

        // This is done to avoid unneeded bounds checks.
        assert!(self.bytecode.len() > index + U32_SIZE && usize::MAX - U32_SIZE >= index);
        self.bytecode[index + 1..=index + U32_SIZE].clone_from_slice(bytes.as_slice());
    }

    fn patch_jump(&mut self, label: Label) {
        let target = self.next_opcode_location();
        self.patch_jump_with_target(label, target);
    }

    fn resolve_identifier_expect(&self, identifier: Identifier) -> JsString {
        identifier.to_js_string(self.interner())
    }

    fn access_get(&mut self, access: Access<'_>, dst: &Register) {
        match access {
            Access::Variable { name } => {
                let name = self.resolve_identifier_expect(name);
                let binding = self.lexical_scope.get_identifier_reference(name);
                let index = self.get_or_insert_binding(binding);
                self.emit_binding_access(Opcode::GetName, &index, dst);
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => {
                    let object = self.register_allocator.alloc();
                    self.compile_expr(access.target(), &object);

                    match access.field() {
                        PropertyAccessField::Const(ident) => {
                            self.emit_get_property_by_name(dst, &object, &object, *ident);
                        }
                        PropertyAccessField::Expr(expr) => {
                            let key = self.register_allocator.alloc();
                            self.compile_expr(expr, &key);

                            self.emit(
                                Opcode::GetPropertyByValue,
                                &[
                                    Operand::Register(dst),
                                    Operand::Register(&key),
                                    Operand::Register(&object),
                                    Operand::Register(&object),
                                ],
                            );
                            self.register_allocator.dealloc(key);
                        }
                    }
                    self.register_allocator.dealloc(object);
                }
                PropertyAccess::Private(access) => {
                    let index = self.get_or_insert_private_name(access.field());
                    let object = self.register_allocator.alloc();
                    self.compile_expr(access.target(), &object);
                    self.emit(
                        Opcode::GetPrivateField,
                        &[
                            Operand::Register(dst),
                            Operand::Register(&object),
                            Operand::Varying(index),
                        ],
                    );
                    self.register_allocator.dealloc(object);
                }
                PropertyAccess::Super(access) => {
                    let value = self.register_allocator.alloc();
                    let receiver = self.register_allocator.alloc();
                    self.emit(Opcode::Super, &[Operand::Register(&value)]);
                    self.emit(Opcode::This, &[Operand::Register(&receiver)]);
                    match access.field() {
                        PropertyAccessField::Const(ident) => {
                            self.emit_get_property_by_name(dst, &receiver, &value, *ident);
                        }
                        PropertyAccessField::Expr(expr) => {
                            let key = self.register_allocator.alloc();
                            self.compile_expr(expr, &key);

                            self.emit(
                                Opcode::GetPropertyByValue,
                                &[
                                    Operand::Register(dst),
                                    Operand::Register(&key),
                                    Operand::Register(&receiver),
                                    Operand::Register(&value),
                                ],
                            );
                            self.register_allocator.dealloc(key);
                        }
                    }
                    self.register_allocator.dealloc(receiver);
                    self.register_allocator.dealloc(value);
                }
            },
            Access::This => {
                self.emit(Opcode::This, &[Operand::Register(dst)]);
            }
        }
    }

    fn access_set<'a, F>(&mut self, access: Access<'_>, expr_fn: F)
    where
        F: FnOnce(&mut ByteCompiler<'_>) -> &'a Register,
    {
        match access {
            Access::Variable { name } => {
                let name = self.resolve_identifier_expect(name);
                let binding = self.lexical_scope.get_identifier_reference(name.clone());
                let is_lexical = binding.is_lexical();
                let index = self.get_or_insert_binding(binding);

                let value = self.register_allocator.alloc();
                if !is_lexical {
                    self.emit_binding_access(Opcode::GetLocator, &index, &value);
                }
                self.register_allocator.dealloc(value);

                let value = expr_fn(self);

                if is_lexical {
                    match self.lexical_scope.set_mutable_binding(name.clone()) {
                        Ok(binding) => {
                            let index = self.get_or_insert_binding(binding);
                            self.emit_binding_access(Opcode::SetName, &index, value);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_string(name);
                            self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                        }
                        Err(BindingLocatorError::Silent) => {}
                    }
                } else {
                    self.emit_binding_access(Opcode::SetNameByLocator, &index, value);
                }
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let object = self.register_allocator.alloc();
                        self.compile_expr(access.target(), &object);
                        let value = expr_fn(self);
                        self.emit_set_property_by_name(value, &object, &object, *name);
                        self.register_allocator.dealloc(object);
                    }
                    PropertyAccessField::Expr(expr) => {
                        let object = self.register_allocator.alloc();
                        self.compile_expr(access.target(), &object);

                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);

                        let value = expr_fn(self);

                        self.emit(
                            Opcode::SetPropertyByValue,
                            &[
                                Operand::Register(value),
                                Operand::Register(&key),
                                Operand::Register(&object),
                                Operand::Register(&object),
                            ],
                        );

                        self.register_allocator.dealloc(object);
                        self.register_allocator.dealloc(key);
                    }
                },
                PropertyAccess::Private(access) => {
                    let index = self.get_or_insert_private_name(access.field());

                    let object = self.register_allocator.alloc();
                    self.compile_expr(access.target(), &object);

                    let value = expr_fn(self);

                    self.emit(
                        Opcode::SetPrivateField,
                        &[
                            Operand::Register(value),
                            Operand::Register(&object),
                            Operand::Varying(index),
                        ],
                    );

                    self.register_allocator.dealloc(object);
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let object = self.register_allocator.alloc();
                        self.emit(Opcode::Super, &[Operand::Register(&object)]);

                        let receiver = self.register_allocator.alloc();
                        self.emit(Opcode::This, &[Operand::Register(&receiver)]);

                        let value = expr_fn(self);

                        self.emit_set_property_by_name(value, &receiver, &object, *name);

                        self.register_allocator.dealloc(receiver);
                        self.register_allocator.dealloc(object);
                    }
                    PropertyAccessField::Expr(expr) => {
                        let object = self.register_allocator.alloc();
                        self.emit(Opcode::Super, &[Operand::Register(&object)]);

                        let receiver = self.register_allocator.alloc();
                        self.emit(Opcode::This, &[Operand::Register(&receiver)]);

                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);

                        let value = expr_fn(self);

                        self.emit(
                            Opcode::SetPropertyByValue,
                            &[
                                Operand::Register(value),
                                Operand::Register(&key),
                                Operand::Register(&receiver),
                                Operand::Register(&object),
                            ],
                        );

                        self.register_allocator.dealloc(key);
                        self.register_allocator.dealloc(receiver);
                        self.register_allocator.dealloc(object);
                    }
                },
            },
            Access::This => todo!("access_set `this`"),
        }
    }

    fn access_delete(&mut self, access: Access<'_>, dst: &Register) {
        match access {
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name((*name).into());
                        self.compile_expr(access.target(), dst);
                        self.emit(
                            Opcode::DeletePropertyByName,
                            &[Operand::Register(dst), Operand::Varying(index)],
                        );
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(access.target(), dst);
                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);
                        self.emit(
                            Opcode::DeletePropertyByValue,
                            &[Operand::Register(dst), Operand::Register(&key)],
                        );
                        self.register_allocator.dealloc(key);
                    }
                },
                PropertyAccess::Super(_) => self.emit_opcode(Opcode::DeleteSuperThrow),
                PropertyAccess::Private(_) => {
                    unreachable!("deleting private properties should always throw early errors.")
                }
            },
            Access::Variable { name } => {
                let name = name.to_js_string(self.interner());
                let binding = self.lexical_scope.get_identifier_reference(name);
                let index = self.get_or_insert_binding(binding);
                self.emit_binding_access(Opcode::DeleteName, &index, dst);
            }
            Access::This => self.push_true(dst),
        }
    }

    /// Compile a [`StatementList`].
    pub fn compile_statement_list(&mut self, list: &StatementList, use_expr: bool, block: bool) {
        if use_expr || self.jump_control_info_has_use_expr() {
            let mut use_expr_index = 0;
            for (i, statement) in list.statements().iter().enumerate() {
                match statement {
                    StatementListItem::Statement(Statement::Break(_) | Statement::Continue(_)) => {
                        break;
                    }
                    StatementListItem::Statement(Statement::Empty | Statement::Var(_))
                    | StatementListItem::Declaration(_) => {}
                    StatementListItem::Statement(Statement::Block(block))
                        if !returns_value(block) => {}
                    StatementListItem::Statement(_) => {
                        use_expr_index = i;
                    }
                }
            }

            for (i, item) in list.statements().iter().enumerate() {
                self.compile_stmt_list_item(item, i == use_expr_index, block);
            }
        } else {
            for item in list.statements() {
                self.compile_stmt_list_item(item, false, block);
            }
        }
    }

    /// Compile an [`Expression`].
    #[inline]
    pub(crate) fn compile_expr(&mut self, expr: &Expression, dst: &'_ Register) {
        self.compile_expr_impl(expr, dst);
    }

    /// Compile a property access expression, prepending `this` to the property value in the stack.
    ///
    /// This compiles the access in a way that the state of the stack after executing the property
    /// access becomes `...rest, this, value`. where `...rest` is the rest of the stack, `this` is the
    /// `this` value of the access, and `value` is the final result of the access.
    ///
    /// This is mostly useful for optional chains with calls (`a.b?.()`) and for regular chains
    /// with calls (`a.b()`), since both of them must have `a` be the value of `this` for the function
    /// call `b()`, but a regular compilation of the access would lose the `this` value after accessing
    /// `b`.
    fn compile_access_preserve_this(
        &mut self,
        access: &PropertyAccess,
        this: &Register,
        dst: &Register,
    ) {
        match access {
            PropertyAccess::Simple(access) => {
                self.compile_expr(access.target(), this);

                match access.field() {
                    PropertyAccessField::Const(ident) => {
                        self.emit_get_property_by_name(dst, this, this, *ident);
                    }
                    PropertyAccessField::Expr(field) => {
                        let key = self.register_allocator.alloc();
                        self.compile_expr(field, &key);
                        self.emit(
                            Opcode::GetPropertyByValue,
                            &[
                                Operand::Register(dst),
                                Operand::Register(&key),
                                Operand::Register(this),
                                Operand::Register(this),
                            ],
                        );
                        self.register_allocator.dealloc(key);
                    }
                }
            }
            PropertyAccess::Private(access) => {
                self.compile_expr(access.target(), this);

                let index = self.get_or_insert_private_name(access.field());
                self.emit(
                    Opcode::GetPrivateField,
                    &[
                        Operand::Register(dst),
                        Operand::Register(this),
                        Operand::Varying(index),
                    ],
                );
            }
            PropertyAccess::Super(access) => {
                let object = self.register_allocator.alloc();
                self.emit(Opcode::This, &[Operand::Register(this)]);
                self.emit(Opcode::Super, &[Operand::Register(&object)]);

                match access.field() {
                    PropertyAccessField::Const(ident) => {
                        self.emit_get_property_by_name(dst, this, &object, *ident);
                    }
                    PropertyAccessField::Expr(expr) => {
                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);
                        self.emit(
                            Opcode::GetPropertyByValue,
                            &[
                                Operand::Register(dst),
                                Operand::Register(&key),
                                Operand::Register(this),
                                Operand::Register(&object),
                            ],
                        );
                        self.register_allocator.dealloc(key);
                    }
                }
                self.register_allocator.dealloc(object);
            }
        }
    }

    /// Compile an optional chain expression, prepending `this` to the property value in the stack.
    ///
    /// This compiles the access in a way that the state of the stack after executing the optional
    /// chain becomes `...rest, this, value`. where `...rest` is the rest of the stack, `this` is the
    /// `this` value of the chain, and `value` is the result of the chain.
    ///
    /// This is mostly useful for inner optional chains with external calls (`(a?.b)()`), because the
    /// external call is not in the optional chain, and compiling an optional chain in the usual way
    /// would only return the result of the chain without preserving the `this` value. In other words,
    /// `this` would be set to `undefined` for that call, which is incorrect since `a` should be the
    /// `this` value of the call.
    fn compile_optional_preserve_this(
        &mut self,
        optional: &Optional,
        this: &Register,
        value: &Register,
    ) {
        let mut jumps = Vec::with_capacity(optional.chain().len());

        match optional.target().flatten() {
            Expression::PropertyAccess(access) => {
                self.compile_access_preserve_this(access, this, value);
            }
            Expression::Optional(opt) => self.compile_optional_preserve_this(opt, this, value),
            expr => {
                self.push_undefined(this);
                self.compile_expr(expr, value);
            }
        }

        jumps.push(self.jump_if_null_or_undefined(value));

        let (first, rest) = optional
            .chain()
            .split_first()
            .expect("chain must have at least one element");
        assert!(first.shorted());

        self.compile_optional_item_kind(first.kind(), this, value);

        for item in rest {
            if item.shorted() {
                jumps.push(self.jump_if_null_or_undefined(value));
            }
            self.compile_optional_item_kind(item.kind(), this, value);
        }

        let skip_undef = self.jump();

        for label in jumps {
            self.patch_jump(label);
            self.push_undefined(value);
        }

        self.patch_jump(skip_undef);
    }

    /// Compile a single operation in an optional chain.
    ///
    /// On successful compilation, the state of the stack on execution will become `...rest, this, value`,
    /// where `this` is the target of the property access (`undefined` on calls), and `value` is the
    /// result of executing the action.
    /// For example, in the expression `a?.b.c()`, after compiling and executing:
    ///
    /// - `a?.b`, the state of the stack will become `...rest, a, b`.
    /// - `b.c`, the state of the stack will become `...rest, b, c`.
    /// - `c()`, the state of the stack will become `...rest, undefined, c()`.
    ///
    /// # Requirements
    /// - This should only be called after verifying that the previous value of the chain
    ///   is not null or undefined (if the operator `?.` was used).
    /// - This assumes that the state of the stack before compiling is `...rest, this, value`,
    ///   since the operation compiled by this function could be a call.
    fn compile_optional_item_kind(
        &mut self,
        kind: &OptionalOperationKind,
        this: &Register,
        value: &Register,
    ) {
        match kind {
            OptionalOperationKind::SimplePropertyAccess { field } => {
                self.emit_move(this, value);
                match field {
                    PropertyAccessField::Const(name) => {
                        self.emit_get_property_by_name(value, value, value, *name);
                    }
                    PropertyAccessField::Expr(expr) => {
                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);
                        self.emit(
                            Opcode::GetPropertyByValue,
                            &[
                                Operand::Register(value),
                                Operand::Register(&key),
                                Operand::Register(value),
                                Operand::Register(value),
                            ],
                        );
                        self.register_allocator.dealloc(key);
                    }
                }
            }
            OptionalOperationKind::PrivatePropertyAccess { field } => {
                self.emit_move(this, value);
                let index = self.get_or_insert_private_name(*field);
                self.emit(
                    Opcode::GetPrivateField,
                    &[
                        Operand::Register(value),
                        Operand::Register(value),
                        Operand::Varying(index),
                    ],
                );
            }
            OptionalOperationKind::Call { args } => {
                self.push_from_register(this);
                self.push_from_register(value);

                let args = &**args;
                let contains_spread = args.iter().any(|arg| matches!(arg, Expression::Spread(_)));

                if contains_spread {
                    let array = self.register_allocator.alloc();
                    let value = self.register_allocator.alloc();

                    self.emit(Opcode::PushNewArray, &[Operand::Register(&array)]);

                    for arg in args {
                        self.compile_expr(arg, &value);
                        if let Expression::Spread(_) = arg {
                            self.emit(Opcode::GetIterator, &[Operand::Register(&value)]);
                            self.emit(Opcode::PushIteratorToArray, &[Operand::Register(&array)]);
                        } else {
                            self.emit(
                                Opcode::PushValueToArray,
                                &[Operand::Register(&value), Operand::Register(&array)],
                            );
                        }
                    }

                    self.push_from_register(&array);

                    self.register_allocator.dealloc(value);
                    self.register_allocator.dealloc(array);

                    self.emit_opcode(Opcode::CallSpread);
                } else {
                    for arg in args {
                        let value = self.register_allocator.alloc();
                        self.compile_expr(arg, &value);
                        self.push_from_register(&value);
                        self.register_allocator.dealloc(value);
                    }
                    self.emit_with_varying_operand(Opcode::Call, args.len() as u32);
                }

                self.pop_into_register(value);
                self.push_undefined(this);
            }
        }
    }

    /// Compile a [`VarDeclaration`].
    fn compile_var_decl(&mut self, decl: &VarDeclaration) {
        for variable in decl.0.as_ref() {
            match variable.binding() {
                Binding::Identifier(ident) => {
                    let ident = ident.to_js_string(self.interner());
                    if let Some(expr) = variable.init() {
                        let binding = self.lexical_scope.get_identifier_reference(ident.clone());
                        let index = self.get_or_insert_binding(binding);
                        let value = self.register_allocator.alloc();
                        self.emit_binding_access(Opcode::GetLocator, &index, &value);
                        self.compile_expr(expr, &value);
                        self.emit_binding_access(Opcode::SetNameByLocator, &index, &value);
                        self.register_allocator.dealloc(value);
                    } else {
                        let value = self.register_allocator.alloc();
                        self.emit_binding(BindingOpcode::Var, ident, &value);
                        self.register_allocator.dealloc(value);
                    }
                }
                Binding::Pattern(pattern) => {
                    let value = self.register_allocator.alloc();
                    if let Some(init) = variable.init() {
                        self.compile_expr(init, &value);
                    } else {
                        self.push_undefined(&value);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar, &value);
                    self.register_allocator.dealloc(value);
                }
            }
        }
    }

    /// Compile a [`LexicalDeclaration`].
    fn compile_lexical_decl(&mut self, decl: &LexicalDeclaration) {
        match decl {
            LexicalDeclaration::Let(decls) => {
                for variable in decls.as_ref() {
                    match variable.binding() {
                        Binding::Identifier(ident) => {
                            let ident = ident.to_js_string(self.interner());
                            let value = self.register_allocator.alloc();
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, &value);
                            } else {
                                self.push_undefined(&value);
                            }
                            self.emit_binding(BindingOpcode::InitLexical, ident, &value);
                            self.register_allocator.dealloc(value);
                        }
                        Binding::Pattern(pattern) => {
                            let value = self.register_allocator.alloc();
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, &value);
                            } else {
                                self.push_undefined(&value);
                            }
                            self.compile_declaration_pattern(
                                pattern,
                                BindingOpcode::InitLexical,
                                &value,
                            );
                            self.register_allocator.dealloc(value);
                        }
                    }
                }
            }
            LexicalDeclaration::Const(decls) => {
                for variable in decls.as_ref() {
                    match variable.binding() {
                        Binding::Identifier(ident) => {
                            let ident = ident.to_js_string(self.interner());
                            let init = variable
                                .init()
                                .expect("const declaration must have initializer");
                            let value = self.register_allocator.alloc();
                            self.compile_expr(init, &value);
                            self.emit_binding(BindingOpcode::InitLexical, ident, &value);
                            self.register_allocator.dealloc(value);
                        }
                        Binding::Pattern(pattern) => {
                            let value = self.register_allocator.alloc();
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, &value);
                            } else {
                                self.push_undefined(&value);
                            }
                            self.compile_declaration_pattern(
                                pattern,
                                BindingOpcode::InitLexical,
                                &value,
                            );
                            self.register_allocator.dealloc(value);
                        }
                    }
                }
            }
        }
    }

    /// Compile a [`StatementListItem`].
    fn compile_stmt_list_item(&mut self, item: &StatementListItem, use_expr: bool, block: bool) {
        match item {
            StatementListItem::Statement(stmt) => {
                self.compile_stmt(stmt, use_expr, false);
            }
            StatementListItem::Declaration(decl) => self.compile_decl(decl, block),
        }
    }

    /// Compile a [`Declaration`].
    #[allow(unused_variables)]
    pub fn compile_decl(&mut self, decl: &Declaration, block: bool) {
        match decl {
            #[cfg(feature = "annex-b")]
            Declaration::FunctionDeclaration(function) if block => {
                let name = function.name();
                if self.annex_b_function_names.contains(&name) {
                    let name = name.to_js_string(self.interner());
                    let binding = self.lexical_scope.get_identifier_reference(name.clone());
                    let index = self.get_or_insert_binding(binding);

                    let value = self.register_allocator.alloc();
                    self.emit_binding_access(Opcode::GetName, &index, &value);
                    match self.variable_scope.set_mutable_binding_var(name.clone()) {
                        Ok(binding) => {
                            let index = self.get_or_insert_binding(binding);
                            self.emit_binding_access(Opcode::SetName, &index, &value);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_string(name);
                            self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                        }
                        Err(BindingLocatorError::Silent) => {}
                    }
                    self.register_allocator.dealloc(value);
                }
            }
            Declaration::ClassDeclaration(class) => self.compile_class(class.into(), None),
            Declaration::Lexical(lexical) => self.compile_lexical_decl(lexical),
            _ => {}
        }
    }

    /// Compiles a function AST Node into bytecode, and returns its index into
    /// the `functions` array.
    pub(crate) fn function(&mut self, function: FunctionSpec<'_>) -> u32 {
        let (generator, r#async, arrow) = (
            function.kind.is_generator(),
            function.kind.is_async(),
            function.kind.is_arrow(),
        );

        let FunctionSpec {
            name,
            parameters,
            body,
            scopes,
            name_scope,
            linear_span,
            ..
        } = function;

        let name = if let Some(name) = name {
            Some(name.sym().to_js_string(self.interner()))
        } else {
            Some(js_string!())
        };

        let spanned_source_text = SpannedSourceText::new(self.source_text(), linear_span);

        let code = FunctionCompiler::new(spanned_source_text)
            .name(name)
            .generator(generator)
            .r#async(r#async)
            .strict(self.strict())
            .arrow(arrow)
            .in_with(self.in_with)
            .name_scope(name_scope.cloned())
            .compile(
                parameters,
                body,
                self.variable_scope.clone(),
                self.lexical_scope.clone(),
                scopes,
                function.contains_direct_eval,
                self.interner,
            );

        self.push_function_to_constants(code)
    }

    /// Compiles a function AST Node into bytecode, setting its corresponding binding or
    /// pushing it to the stack if necessary.
    pub(crate) fn function_with_binding(
        &mut self,
        function: FunctionSpec<'_>,
        node_kind: NodeKind,
        dst: &Register,
    ) {
        let name = function.name;
        let index = self.function(function);
        self.emit_get_function(dst, index);
        match node_kind {
            NodeKind::Declaration => {
                self.emit_binding(
                    BindingOpcode::InitVar,
                    name.expect("function declaration must have a name")
                        .to_js_string(self.interner()),
                    dst,
                );
            }
            NodeKind::Expression => {}
        }
    }

    /// Compile an object method AST Node into bytecode.
    pub(crate) fn object_method(
        &mut self,
        function: FunctionSpec<'_>,
        kind: MethodKind,
    ) -> Register {
        let (generator, r#async, arrow) = (
            function.kind.is_generator(),
            function.kind.is_async(),
            function.kind.is_arrow(),
        );
        let FunctionSpec {
            name,
            parameters,
            body,
            scopes,
            name_scope,
            linear_span,
            ..
        } = function;

        let name = if let Some(name) = name {
            let name = name.sym().to_js_string(self.interner());
            match kind {
                MethodKind::Ordinary => Some(name),
                MethodKind::Get => Some(js_string!(js_str!("get "), &name)),
                MethodKind::Set => Some(js_string!(js_str!("set "), &name)),
            }
        } else {
            Some(js_string!())
        };

        let spanned_source_text = SpannedSourceText::new(self.source_text(), linear_span);

        let code = FunctionCompiler::new(spanned_source_text)
            .name(name)
            .generator(generator)
            .r#async(r#async)
            .strict(self.strict())
            .arrow(arrow)
            .method(true)
            .in_with(self.in_with)
            .name_scope(name_scope.cloned())
            .compile(
                parameters,
                body,
                self.variable_scope.clone(),
                self.lexical_scope.clone(),
                scopes,
                function.contains_direct_eval,
                self.interner,
            );

        let index = self.push_function_to_constants(code);
        let dst = self.register_allocator.alloc();
        self.emit_get_function(&dst, index);
        dst
    }

    /// Compile a class method AST Node into bytecode.
    fn method(&mut self, function: FunctionSpec<'_>) -> Register {
        let (generator, r#async, arrow) = (
            function.kind.is_generator(),
            function.kind.is_async(),
            function.kind.is_arrow(),
        );
        let FunctionSpec {
            name,
            parameters,
            body,
            scopes,
            linear_span,
            ..
        } = function;

        let name = if let Some(name) = name {
            Some(name.sym().to_js_string(self.interner()))
        } else {
            Some(js_string!())
        };

        let spanned_source_text = SpannedSourceText::new(self.source_text(), linear_span);

        let code = FunctionCompiler::new(spanned_source_text)
            .name(name)
            .generator(generator)
            .r#async(r#async)
            .strict(true)
            .arrow(arrow)
            .method(true)
            .in_with(self.in_with)
            .name_scope(function.name_scope.cloned())
            .compile(
                parameters,
                body,
                self.variable_scope.clone(),
                self.lexical_scope.clone(),
                scopes,
                function.contains_direct_eval,
                self.interner,
            );

        let index = self.push_function_to_constants(code);
        let dst = self.register_allocator.alloc();
        self.emit_get_function(&dst, index);
        dst
    }

    fn call(&mut self, callable: Callable<'_>, dst: &Register) {
        #[derive(PartialEq)]
        enum CallKind {
            CallEval,
            Call,
            New,
        }

        let (call, mut kind) = match callable {
            Callable::Call(call) => (call, CallKind::Call),
            Callable::New(new) => (new.call(), CallKind::New),
        };

        match call.function().flatten() {
            Expression::PropertyAccess(access) if kind == CallKind::Call => {
                let this = self.register_allocator.alloc();
                let dst = self.register_allocator.alloc();
                self.compile_access_preserve_this(access, &this, &dst);
                self.push_from_register(&this);
                self.push_from_register(&dst);
                self.register_allocator.dealloc(this);
                self.register_allocator.dealloc(dst);
            }

            Expression::Optional(opt) if kind == CallKind::Call => {
                let this = self.register_allocator.alloc();
                let dst = self.register_allocator.alloc();
                self.compile_optional_preserve_this(opt, &this, &dst);
                self.push_from_register(&this);
                self.push_from_register(&dst);
                self.register_allocator.dealloc(this);
                self.register_allocator.dealloc(dst);
            }
            expr if kind == CallKind::Call => {
                if let Expression::Identifier(ident) = expr {
                    if *ident == Sym::EVAL {
                        kind = CallKind::CallEval;
                    }

                    if self.in_with {
                        let name = self.resolve_identifier_expect(*ident);
                        let binding = self.lexical_scope.get_identifier_reference(name);
                        let index = self.get_or_insert_binding(binding);
                        let index = match index {
                            BindingKind::Global(index) | BindingKind::Stack(index) => index,
                            BindingKind::Local(_) => {
                                unreachable!("with binding cannot be local")
                            }
                        };
                        let value = self.register_allocator.alloc();
                        self.emit(
                            Opcode::ThisForObjectEnvironmentName,
                            &[Operand::Register(&value), Operand::Varying(index)],
                        );
                        self.push_from_register(&value);
                        self.register_allocator.dealloc(value);
                    } else {
                        let value = self.register_allocator.alloc();
                        self.push_undefined(&value);
                        self.push_from_register(&value);
                        self.register_allocator.dealloc(value);
                    }
                } else {
                    let value = self.register_allocator.alloc();
                    self.push_undefined(&value);
                    self.push_from_register(&value);
                    self.register_allocator.dealloc(value);
                }

                let value = self.register_allocator.alloc();
                self.compile_expr(expr, &value);
                self.push_from_register(&value);
                self.register_allocator.dealloc(value);
            }
            expr => {
                let value = self.register_allocator.alloc();
                self.compile_expr(expr, &value);
                self.push_from_register(&value);
                self.register_allocator.dealloc(value);
            }
        }

        let contains_spread = call
            .args()
            .iter()
            .any(|arg| matches!(arg, Expression::Spread(_)));

        if contains_spread {
            let array = self.register_allocator.alloc();
            let value = self.register_allocator.alloc();

            self.emit(Opcode::PushNewArray, &[Operand::Register(&array)]);

            for arg in call.args() {
                self.compile_expr(arg, &value);
                if let Expression::Spread(_) = arg {
                    self.emit(Opcode::GetIterator, &[Operand::Register(&value)]);
                    self.emit(Opcode::PushIteratorToArray, &[Operand::Register(&array)]);
                } else {
                    self.emit(
                        Opcode::PushValueToArray,
                        &[Operand::Register(&value), Operand::Register(&array)],
                    );
                }
            }

            self.push_from_register(&array);

            self.register_allocator.dealloc(array);
            self.register_allocator.dealloc(value);
        } else {
            for arg in call.args() {
                let value = self.register_allocator.alloc();
                self.compile_expr(arg, &value);
                self.push_from_register(&value);
                self.register_allocator.dealloc(value);
            }
        }

        match kind {
            CallKind::CallEval => {
                let scope_index = self.constants.len() as u32;
                self.constants
                    .push(Constant::Scope(self.lexical_scope.clone()));
                if contains_spread {
                    self.emit_with_varying_operand(Opcode::CallEvalSpread, scope_index);
                } else {
                    self.emit(
                        Opcode::CallEval,
                        &[
                            Operand::Varying(call.args().len() as u32),
                            Operand::Varying(scope_index),
                        ],
                    );
                }
            }
            CallKind::Call if contains_spread => self.emit_opcode(Opcode::CallSpread),
            CallKind::Call => {
                self.emit_with_varying_operand(Opcode::Call, call.args().len() as u32);
            }
            CallKind::New if contains_spread => self.emit_opcode(Opcode::NewSpread),
            CallKind::New => self.emit_with_varying_operand(Opcode::New, call.args().len() as u32),
        }
        self.pop_into_register(dst);
    }

    /// Finish compiling code with the [`ByteCompiler`] and return the generated [`CodeBlock`].
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn finish(mut self) -> CodeBlock {
        // Push return at the end of the function compilation.
        if let Some(async_handler) = self.async_handler {
            self.patch_handler(async_handler);
        }
        self.r#return(false);

        let mapped_arguments_binding_indices = self
            .emitted_mapped_arguments_object_opcode
            .then(|| {
                MappedArguments::binding_indices(&self.params, &self.parameter_scope, self.interner)
            })
            .unwrap_or_default();

        let max_local_binding_register_index =
            self.local_binding_registers.values().max().unwrap_or(&0);
        let local_bindings_initialized =
            vec![false; (max_local_binding_register_index + 1) as usize].into_boxed_slice();

        let register_count = self.register_allocator.finish();

        CodeBlock {
            name: self.function_name,
            length: self.length,
            register_count,
            this_mode: self.this_mode,
            parameter_length: self.params.as_ref().len() as u32,
            mapped_arguments_binding_indices,
            bytecode: self.bytecode.into_boxed_slice(),
            constants: self.constants,
            bindings: self.bindings.into_boxed_slice(),
            local_bindings_initialized,
            handlers: self.handlers,
            flags: Cell::new(self.code_block_flags),
            ic: self.ic.into_boxed_slice(),
            source_text_spanned: self.spanned_source_text,
        }
    }

    fn compile_declaration_pattern(
        &mut self,
        pattern: &Pattern,
        def: BindingOpcode,
        object: &Register,
    ) {
        self.compile_declaration_pattern_impl(pattern, def, object);
    }
}
