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

use std::{
    borrow::{Borrow, BorrowMut},
    cell::Cell,
    ops::{Deref, DerefMut},
};

use crate::{
    JsBigInt, JsStr, JsString, SourceText, SpannedSourceText,
    builtins::function::{ThisMode, arguments::MappedArguments},
    js_string,
    vm::{
        CallFrame, CodeBlock, CodeBlockFlags, Constant, GeneratorResumeKind, Handler, InlineCache,
        opcode::{BindingOpcode, ByteCodeEmitter, VaryingOperand},
        source_info::{SourceInfo, SourceMap, SourceMapBuilder, SourcePath},
    },
};
use boa_ast::{
    Declaration, Expression, LinearSpan, Position, Spanned, Statement, StatementList,
    StatementListItem,
    declaration::{Binding, LexicalDeclaration, VarDeclaration},
    expression::{
        Call, Identifier, New, Optional, OptionalOperationKind,
        access::{PropertyAccess, PropertyAccessField},
        literal::ObjectMethodDefinition,
        operator::{assign::AssignTarget, update::UpdateTarget},
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
};
use boa_gc::Gc;
use boa_interner::{Interner, Sym};
use boa_macros::js_str;
use boa_string::StaticJsStrings;
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
            name: method.name().literal(),
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
#[allow(variant_size_differences)]
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
            Expression::This(_this) => Some(Access::This),
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

#[derive(Copy, Clone, Debug)]
pub(crate) enum BindingAccessOpcode {
    PutLexicalValue,
    DefInitVar,
    SetName,
    SetNameByLocator,
    GetName,
    GetNameAndLocator,
    GetNameOrUndefined,
    DeleteName,
    GetLocator,
    DefVar,
}

/// Manages the source position scope, push on creation, pop on drop.
pub(crate) struct SourcePositionGuard<'a, 'b> {
    compiler: &'a mut ByteCompiler<'b>,
}
impl<'a, 'b> SourcePositionGuard<'a, 'b> {
    fn new(compiler: &'a mut ByteCompiler<'b>, position: Position) -> Self {
        compiler.push_source_position(position);
        Self { compiler }
    }
}
impl Drop for SourcePositionGuard<'_, '_> {
    fn drop(&mut self) {
        self.pop_source_position();
    }
}
impl<'a> Deref for SourcePositionGuard<'_, 'a> {
    type Target = ByteCompiler<'a>;
    fn deref(&self) -> &Self::Target {
        self.compiler
    }
}
impl DerefMut for SourcePositionGuard<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.compiler
    }
}
impl<'a> Borrow<ByteCompiler<'a>> for SourcePositionGuard<'_, 'a> {
    fn borrow(&self) -> &ByteCompiler<'a> {
        self.compiler
    }
}
impl<'a> BorrowMut<ByteCompiler<'a>> for SourcePositionGuard<'_, 'a> {
    fn borrow_mut(&mut self) -> &mut ByteCompiler<'a> {
        self.compiler
    }
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
    pub(crate) bytecode: ByteCodeEmitter,

    pub(crate) source_map_builder: SourceMapBuilder,
    pub(crate) source_path: SourcePath,

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
    names_map: FxHashMap<Sym, u32>,
    bindings_map: FxHashMap<BindingLocator, u32>,
    jump_info: Vec<JumpControlInfo>,

    /// Used to handle exception throws that escape the async function types.
    ///
    /// Async functions and async generator functions, need to be closed and resolved.
    pub(crate) async_handler: Option<u32>,
    json_parse: bool,

    /// Whether the function is in a `with` statement.
    pub(crate) in_with: bool,

    /// Used to determine if a we emitted a `CreateUnmappedArgumentsObject` opcode
    pub(crate) emitted_mapped_arguments_object_opcode: bool,

    pub(crate) interner: &'ctx mut Interner,
    spanned_source_text: SpannedSourceText,

    #[cfg(feature = "annex-b")]
    pub(crate) annex_b_function_names: Vec<Sym>,
}

pub(crate) enum BindingKind {
    Stack(u32),
    Local(Option<u32>),
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
        source_path: SourcePath,
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
                CallFrame::PROMISE_CAPABILITY_PROMISE_REGISTER_INDEX as u32
            );
            debug_assert_eq!(
                resolve_register.index(),
                CallFrame::PROMISE_CAPABILITY_RESOLVE_REGISTER_INDEX as u32
            );
            debug_assert_eq!(
                reject_register.index(),
                CallFrame::PROMISE_CAPABILITY_REJECT_REGISTER_INDEX as u32
            );

            if is_generator {
                let async_function_object_register = register_allocator.alloc_persistent();
                debug_assert_eq!(
                    async_function_object_register.index(),
                    CallFrame::ASYNC_GENERATOR_OBJECT_REGISTER_INDEX as u32
                );
            }
        }

        Self {
            function_name: name,
            length: 0,
            bytecode: ByteCodeEmitter::new(),
            source_map_builder: SourceMapBuilder::default(),
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
            source_path,

            #[cfg(feature = "annex-b")]
            annex_b_function_names: Vec::new(),
            in_with,
            emitted_mapped_arguments_object_opcode: false,
        }
    }

    /// Executes a closure that emits a call opcode, while persisting the accumulator.
    /// This fixes an issue where function calls that don't return a value
    /// incorrectly inherit the caller's accumulator value.
    pub(crate) fn emit_with_accumulator_stashed<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let acc_backup = self.register_allocator.alloc();
        self.bytecode
            .emit_set_register_from_accumulator(acc_backup.variable());

        let undefined = self.register_allocator.alloc();
        self.bytecode.emit_push_undefined(undefined.variable());
        self.bytecode.emit_set_accumulator(undefined.variable());
        self.register_allocator.dealloc(undefined);

        f(self);

        self.bytecode.emit_set_accumulator(acc_backup.variable());
        self.register_allocator.dealloc(acc_backup);
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

    fn get_or_insert_name(&mut self, name: Sym) -> u32 {
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
        self.get_or_insert_name(name.description())
    }

    // TODO: Make this return `Option<BindingKind>` instead of making BindingKind::Local
    //       inner field optional.
    #[inline]
    pub(crate) fn get_binding(&mut self, binding: &IdentifierReference) -> BindingKind {
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
            return BindingKind::Local(self.local_binding_registers.get(binding).copied());
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
    pub(crate) fn insert_binding(&mut self, binding: IdentifierReference) -> BindingKind {
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
            return BindingKind::Local(Some(
                *self
                    .local_binding_registers
                    .entry(binding)
                    .or_insert_with(|| self.register_allocator.alloc_persistent().index()),
            ));
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
                    let index = self.insert_binding(binding);
                    self.emit_binding_access(BindingAccessOpcode::DefVar, &index, value);
                }
            }
            BindingOpcode::InitVar => match self.lexical_scope.set_mutable_binding(name.clone()) {
                Ok(binding) => {
                    let index = self.insert_binding(binding);
                    self.emit_binding_access(BindingAccessOpcode::DefInitVar, &index, value);
                }
                Err(BindingLocatorError::MutateImmutable) => {
                    let index = self.get_or_insert_string(name);
                    self.bytecode.emit_throw_mutate_immutable(index.into());
                }
                Err(BindingLocatorError::Silent) => {}
            },
            BindingOpcode::InitLexical => {
                let binding = self.lexical_scope.get_identifier_reference(name);
                let index = self.insert_binding(binding);
                self.emit_binding_access(BindingAccessOpcode::PutLexicalValue, &index, value);
            }
            BindingOpcode::SetName => match self.lexical_scope.set_mutable_binding(name.clone()) {
                Ok(binding) => {
                    let index = self.insert_binding(binding);
                    self.emit_binding_access(BindingAccessOpcode::SetName, &index, value);
                }
                Err(BindingLocatorError::MutateImmutable) => {
                    let index = self.get_or_insert_string(name);
                    self.bytecode.emit_throw_mutate_immutable(index.into());
                }
                Err(BindingLocatorError::Silent) => {}
            },
        }
    }

    fn next_opcode_location(&mut self) -> u32 {
        self.bytecode.next_opcode_location()
    }

    pub(crate) fn position_guard(
        &mut self,
        spanned: impl Spanned,
    ) -> SourcePositionGuard<'_, 'ctx> {
        SourcePositionGuard::new(self, spanned.span().start())
    }

    pub(crate) fn push_source_position<T>(&mut self, position: T)
    where
        T: Into<Option<Position>>,
    {
        let start_pc = self.next_opcode_location();
        self.source_map_builder
            .push_source_position(start_pc, position.into());
    }

    pub(crate) fn pop_source_position(&mut self) {
        let start_pc = self.next_opcode_location();
        self.source_map_builder.pop_source_position(start_pc);
    }

    pub(crate) fn emit_get_function(&mut self, dst: &Register, index: u32) {
        self.bytecode
            .emit_get_function(dst.variable(), index.into());
    }

    fn pop_into_register(&mut self, dst: &Register) {
        self.bytecode.emit_pop_into_register(dst.variable());
    }

    pub(crate) fn push_from_register(&mut self, src: &Register) {
        self.bytecode.emit_push_from_register(src.variable());
    }

    pub(crate) fn emit_binding_access(
        &mut self,
        opcode: BindingAccessOpcode,
        binding: &BindingKind,
        value: &Register,
    ) {
        match binding {
            BindingKind::Global(index) => match opcode {
                BindingAccessOpcode::SetNameByLocator => {
                    self.bytecode.emit_set_name_by_locator(value.variable());
                }
                BindingAccessOpcode::GetName => {
                    let ic_index = self.ic.len() as u32;
                    let name = self.bindings[*index as usize].name().clone();
                    self.ic.push(InlineCache::new(name));
                    self.bytecode.emit_get_name_global(
                        value.variable(),
                        (*index).into(),
                        ic_index.into(),
                    );
                }
                BindingAccessOpcode::GetLocator => self.bytecode.emit_get_locator((*index).into()),
                BindingAccessOpcode::DefVar => self.bytecode.emit_def_var((*index).into()),
                BindingAccessOpcode::PutLexicalValue => self
                    .bytecode
                    .emit_put_lexical_value(value.variable(), (*index).into()),
                BindingAccessOpcode::DefInitVar => self
                    .bytecode
                    .emit_def_init_var(value.variable(), (*index).into()),
                BindingAccessOpcode::SetName => self
                    .bytecode
                    .emit_set_name(value.variable(), (*index).into()),
                BindingAccessOpcode::GetNameAndLocator => self
                    .bytecode
                    .emit_get_name_and_locator(value.variable(), (*index).into()),
                BindingAccessOpcode::GetNameOrUndefined => self
                    .bytecode
                    .emit_get_name_or_undefined(value.variable(), (*index).into()),
                BindingAccessOpcode::DeleteName => self
                    .bytecode
                    .emit_delete_name(value.variable(), (*index).into()),
            },
            BindingKind::Stack(index) => match opcode {
                BindingAccessOpcode::SetNameByLocator => {
                    self.bytecode.emit_set_name_by_locator(value.variable());
                }
                BindingAccessOpcode::GetLocator => self.bytecode.emit_get_locator((*index).into()),
                BindingAccessOpcode::DefVar => self.bytecode.emit_def_var((*index).into()),
                BindingAccessOpcode::PutLexicalValue => self
                    .bytecode
                    .emit_put_lexical_value(value.variable(), (*index).into()),
                BindingAccessOpcode::DefInitVar => self
                    .bytecode
                    .emit_def_init_var(value.variable(), (*index).into()),
                BindingAccessOpcode::SetName => self
                    .bytecode
                    .emit_set_name(value.variable(), (*index).into()),
                BindingAccessOpcode::GetName => self
                    .bytecode
                    .emit_get_name(value.variable(), (*index).into()),
                BindingAccessOpcode::GetNameAndLocator => self
                    .bytecode
                    .emit_get_name_and_locator(value.variable(), (*index).into()),
                BindingAccessOpcode::GetNameOrUndefined => self
                    .bytecode
                    .emit_get_name_or_undefined(value.variable(), (*index).into()),
                BindingAccessOpcode::DeleteName => self
                    .bytecode
                    .emit_delete_name(value.variable(), (*index).into()),
            },
            BindingKind::Local(None) => {
                let error_msg = self.get_or_insert_literal(Literal::String(js_string!(
                    "access of uninitialized binding"
                )));
                self.bytecode
                    .emit_throw_new_reference_error(error_msg.into());
            }
            BindingKind::Local(Some(index)) => match opcode {
                BindingAccessOpcode::GetName
                | BindingAccessOpcode::GetNameOrUndefined
                | BindingAccessOpcode::GetNameAndLocator => {
                    self.bytecode.emit_move(value.variable(), (*index).into());
                }
                BindingAccessOpcode::GetLocator | BindingAccessOpcode::DefVar => {}
                BindingAccessOpcode::SetName
                | BindingAccessOpcode::DefInitVar
                | BindingAccessOpcode::PutLexicalValue
                | BindingAccessOpcode::SetNameByLocator => {
                    self.bytecode.emit_move((*index).into(), value.variable());
                }
                BindingAccessOpcode::DeleteName => self.bytecode.emit_push_false(value.variable()),
            },
        }
    }

    fn emit_get_property_by_name(
        &mut self,
        dst: &Register,
        receiver: Option<&Register>,
        value: &Register,
        ident: Sym,
    ) {
        let ic_index = self.ic.len() as u32;

        let name_index = self.get_or_insert_name(ident);
        let Constant::String(ref name) = self.constants[name_index as usize].clone() else {
            unreachable!("there should be a string at index")
        };
        self.ic.push(InlineCache::new(name.clone()));

        if let Some(receiver) = receiver {
            self.bytecode.emit_get_property_by_name_with_this(
                dst.variable(),
                receiver.variable(),
                value.variable(),
                ic_index.into(),
            );
        } else if name == &StaticJsStrings::LENGTH {
            self.bytecode.emit_get_length_property(
                dst.variable(),
                value.variable(),
                ic_index.into(),
            );
        } else {
            self.bytecode.emit_get_property_by_name(
                dst.variable(),
                value.variable(),
                ic_index.into(),
            );
        }
    }

    fn emit_set_property_by_name(
        &mut self,
        value: &Register,
        receiver: Option<&Register>,
        object: &Register,
        ident: Sym,
    ) {
        let ic_index = self.ic.len() as u32;

        let name_index = self.get_or_insert_name(ident);
        let Constant::String(ref name) = self.constants[name_index as usize].clone() else {
            unreachable!("there should be a string at index")
        };
        self.ic.push(InlineCache::new(name.clone()));

        if let Some(receiver) = receiver {
            self.bytecode.emit_set_property_by_name_with_this(
                value.variable(),
                receiver.variable(),
                object.variable(),
                ic_index.into(),
            );
        } else {
            self.bytecode.emit_set_property_by_name(
                value.variable(),
                object.variable(),
                ic_index.into(),
            );
        }
    }

    fn emit_type_error(&mut self, message: &str) {
        let error_msg = self.get_or_insert_literal(Literal::String(js_string!(message)));
        self.bytecode.emit_throw_new_type_error(error_msg.into());
    }
    fn emit_syntax_error(&mut self, message: &str) {
        let error_msg = self.get_or_insert_literal(Literal::String(js_string!(message)));
        self.bytecode.emit_throw_new_syntax_error(error_msg.into());
    }

    fn emit_push_integer(&mut self, value: i32, dst: &Register) {
        self.emit_push_integer_with_index(value, dst.variable());
    }

    fn emit_push_integer_with_index(&mut self, value: i32, dst: VaryingOperand) {
        match value {
            0 => self.bytecode.emit_push_zero(dst),
            1 => self.bytecode.emit_push_one(dst),
            x if i32::from(x as i8) == x => self.bytecode.emit_push_int8(dst, x as i8),
            x if i32::from(x as i16) == x => {
                self.bytecode.emit_push_int16(dst, x as i16);
            }
            x => self.bytecode.emit_push_int32(dst, x),
        }
    }

    fn emit_push_literal(&mut self, literal: Literal, dst: &Register) {
        let index = self.get_or_insert_literal(literal);
        self.bytecode
            .emit_push_literal(dst.variable(), index.into());
    }

    fn emit_push_rational(&mut self, value: f64, dst: &Register) {
        if value.is_nan() {
            return self.bytecode.emit_push_nan(dst.variable());
        }

        if value.is_infinite() {
            if value.is_sign_positive() {
                return self.bytecode.emit_push_positive_infinity(dst.variable());
            }
            return self.bytecode.emit_push_negative_infinity(dst.variable());
        }

        // Check if the f64 value can fit in an i32.
        if f64::from(value as i32).to_bits() == value.to_bits() {
            self.emit_push_integer(value as i32, dst);
        } else {
            let f32_value = value as f32;

            #[allow(clippy::float_cmp)]
            if f64::from(f32_value) == value {
                self.bytecode.emit_push_float(dst.variable(), f32_value);
            } else {
                self.bytecode.emit_push_double(dst.variable(), value);
            }
        }
    }

    fn jump(&mut self) -> Label {
        let index = self.next_opcode_location();
        self.bytecode.emit_jump(Self::DUMMY_ADDRESS);
        Label { index }
    }

    pub(crate) fn jump_if_true(&mut self, value: &Register) -> Label {
        let index = self.next_opcode_location();
        self.bytecode
            .emit_jump_if_true(Self::DUMMY_ADDRESS, value.variable());
        Label { index }
    }

    pub(crate) fn jump_if_false(&mut self, value: &Register) -> Label {
        let index = self.next_opcode_location();
        self.bytecode
            .emit_jump_if_false(Self::DUMMY_ADDRESS, value.variable());
        Label { index }
    }

    pub(crate) fn jump_if_null_or_undefined(&mut self, value: &Register) -> Label {
        let index = self.next_opcode_location();
        self.bytecode
            .emit_jump_if_null_or_undefined(Self::DUMMY_ADDRESS, value.variable());
        Label { index }
    }

    pub(crate) fn emit_jump_if_not_undefined(&mut self, value: &Register) -> Label {
        let index = self.next_opcode_location();
        self.bytecode
            .emit_jump_if_not_undefined(Self::DUMMY_ADDRESS, value.variable());
        Label { index }
    }

    pub(crate) fn case(&mut self, value: &Register, condition: &Register) -> Label {
        let index = self.next_opcode_location();
        self.bytecode
            .emit_case(Self::DUMMY_ADDRESS, value.variable(), condition.variable());
        Label { index }
    }

    pub(crate) fn template_lookup(&mut self, dst: &Register, site: u64) -> Label {
        let index = self.next_opcode_location();
        self.bytecode
            .emit_template_lookup(Self::DUMMY_ADDRESS, site, dst.variable());
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
        self.bytecode.emit_jump_if_not_resume_kind(
            Self::DUMMY_ADDRESS,
            (resume_kind as u8).into(),
            value.variable(),
        );
        Label { index }
    }

    #[track_caller]
    pub(crate) fn patch_jump_with_target(&mut self, label: Label, target: u32) {
        self.bytecode.patch_jump(label.index, target);
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
                let index = self.get_binding(&binding);
                self.emit_binding_access(BindingAccessOpcode::GetName, &index, dst);
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => {
                    let mut compiler = self.position_guard(access.field());

                    let object = compiler.register_allocator.alloc();
                    compiler.compile_expr(access.target(), &object);

                    match access.field() {
                        PropertyAccessField::Const(ident) => {
                            compiler.emit_get_property_by_name(dst, None, &object, ident.sym());
                        }
                        PropertyAccessField::Expr(expr) => {
                            let key = compiler.register_allocator.alloc();
                            compiler.compile_expr(expr, &key);
                            compiler.bytecode.emit_get_property_by_value(
                                dst.variable(),
                                key.variable(),
                                object.variable(),
                                object.variable(),
                            );
                            compiler.register_allocator.dealloc(key);
                        }
                    }
                    compiler.register_allocator.dealloc(object);
                }
                PropertyAccess::Private(access) => {
                    let mut compiler = self.position_guard(access.field());

                    let index = compiler.get_or_insert_private_name(access.field());
                    let object = compiler.register_allocator.alloc();
                    compiler.compile_expr(access.target(), &object);
                    compiler.bytecode.emit_get_private_field(
                        dst.variable(),
                        object.variable(),
                        index.into(),
                    );
                    compiler.register_allocator.dealloc(object);
                }
                PropertyAccess::Super(access) => {
                    let mut compiler = self.position_guard(access.field());

                    let value = compiler.register_allocator.alloc();
                    let receiver = compiler.register_allocator.alloc();
                    compiler.bytecode.emit_super(value.variable());
                    compiler.bytecode.emit_this(receiver.variable());
                    match access.field() {
                        PropertyAccessField::Const(ident) => {
                            compiler.emit_get_property_by_name(
                                dst,
                                Some(&receiver),
                                &value,
                                ident.sym(),
                            );
                        }
                        PropertyAccessField::Expr(expr) => {
                            let key = compiler.register_allocator.alloc();
                            compiler.compile_expr(expr, &key);
                            compiler.bytecode.emit_get_property_by_value(
                                dst.variable(),
                                key.variable(),
                                receiver.variable(),
                                value.variable(),
                            );
                            compiler.register_allocator.dealloc(key);
                        }
                    }
                    compiler.register_allocator.dealloc(receiver);
                    compiler.register_allocator.dealloc(value);
                }
            },
            Access::This => {
                self.bytecode.emit_this(dst.variable());
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
                let index = self.get_binding(&binding);

                let value = self.register_allocator.alloc();
                if !is_lexical {
                    self.emit_binding_access(BindingAccessOpcode::GetLocator, &index, &value);
                }
                self.register_allocator.dealloc(value);

                let value = expr_fn(self);

                if is_lexical {
                    match self.lexical_scope.set_mutable_binding(name.clone()) {
                        Ok(binding) => {
                            let index = self.insert_binding(binding);
                            self.emit_binding_access(BindingAccessOpcode::SetName, &index, value);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_string(name);
                            self.bytecode.emit_throw_mutate_immutable(index.into());
                        }
                        Err(BindingLocatorError::Silent) => {}
                    }
                } else {
                    self.emit_binding_access(BindingAccessOpcode::SetNameByLocator, &index, value);
                }
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let object = self.register_allocator.alloc();
                        self.compile_expr(access.target(), &object);
                        let value = expr_fn(self);
                        self.emit_set_property_by_name(value, None, &object, name.sym());
                        self.register_allocator.dealloc(object);
                    }
                    PropertyAccessField::Expr(expr) => {
                        let object = self.register_allocator.alloc();
                        self.compile_expr(access.target(), &object);

                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);

                        let value = expr_fn(self);

                        self.bytecode.emit_set_property_by_value(
                            value.variable(),
                            key.variable(),
                            object.variable(),
                            object.variable(),
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

                    self.bytecode.emit_set_private_field(
                        value.variable(),
                        object.variable(),
                        index.into(),
                    );

                    self.register_allocator.dealloc(object);
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let object = self.register_allocator.alloc();
                        self.bytecode.emit_super(object.variable());

                        let receiver = self.register_allocator.alloc();
                        self.bytecode.emit_this(receiver.variable());

                        let value = expr_fn(self);

                        self.emit_set_property_by_name(value, Some(&receiver), &object, name.sym());

                        self.register_allocator.dealloc(receiver);
                        self.register_allocator.dealloc(object);
                    }
                    PropertyAccessField::Expr(expr) => {
                        let object = self.register_allocator.alloc();
                        self.bytecode.emit_super(object.variable());

                        let receiver = self.register_allocator.alloc();
                        self.bytecode.emit_this(receiver.variable());

                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);

                        let value = expr_fn(self);

                        self.bytecode.emit_set_property_by_value(
                            value.variable(),
                            key.variable(),
                            receiver.variable(),
                            object.variable(),
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
                        let index = self.get_or_insert_name(name.sym());
                        self.compile_expr(access.target(), dst);
                        self.bytecode
                            .emit_delete_property_by_name(dst.variable(), index.into());
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(access.target(), dst);
                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);
                        self.bytecode
                            .emit_delete_property_by_value(dst.variable(), key.variable());
                        self.register_allocator.dealloc(key);
                    }
                },
                PropertyAccess::Super(_) => self.bytecode.emit_delete_super_throw(),
                PropertyAccess::Private(_) => {
                    unreachable!("deleting private properties should always throw early errors.")
                }
            },
            Access::Variable { name } => {
                let name = name.to_js_string(self.interner());
                let binding = self.lexical_scope.get_identifier_reference(name);
                let index = self.get_binding(&binding);
                self.emit_binding_access(BindingAccessOpcode::DeleteName, &index, dst);
            }
            Access::This => self.bytecode.emit_push_true(dst.variable()),
        }
    }

    /// Compile a [`StatementList`].
    pub fn compile_statement_list(&mut self, list: &StatementList, use_expr: bool, block: bool) {
        if use_expr || self.jump_control_info_has_use_expr() {
            let mut use_expr_index = 0;
            for (i, statement) in list.statements().iter().enumerate() {
                match statement {
                    StatementListItem::Statement(statement) => match statement.as_ref() {
                        Statement::Break(_) | Statement::Continue(_) => break,
                        Statement::Empty | Statement::Var(_) => {}
                        Statement::Block(block) if !returns_value(block) => {}
                        _ => use_expr_index = i,
                    },
                    StatementListItem::Declaration(_) => {}
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
                        self.emit_get_property_by_name(dst, None, this, ident.sym());
                    }
                    PropertyAccessField::Expr(field) => {
                        let key = self.register_allocator.alloc();
                        self.compile_expr(field, &key);
                        self.bytecode.emit_get_property_by_value(
                            dst.variable(),
                            key.variable(),
                            this.variable(),
                            this.variable(),
                        );
                        self.register_allocator.dealloc(key);
                    }
                }
            }
            PropertyAccess::Private(access) => {
                self.compile_expr(access.target(), this);

                let index = self.get_or_insert_private_name(access.field());
                self.bytecode
                    .emit_get_private_field(dst.variable(), this.variable(), index.into());
            }
            PropertyAccess::Super(access) => {
                let object = self.register_allocator.alloc();
                self.bytecode.emit_this(this.variable());
                self.bytecode.emit_super(object.variable());

                match access.field() {
                    PropertyAccessField::Const(ident) => {
                        self.emit_get_property_by_name(dst, Some(this), &object, ident.sym());
                    }
                    PropertyAccessField::Expr(expr) => {
                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);
                        self.bytecode.emit_get_property_by_value(
                            dst.variable(),
                            key.variable(),
                            this.variable(),
                            object.variable(),
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
                self.bytecode.emit_push_undefined(this.variable());
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
            self.bytecode.emit_push_undefined(value.variable());
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
                self.bytecode.emit_move(this.variable(), value.variable());
                match field {
                    PropertyAccessField::Const(name) => {
                        self.emit_get_property_by_name(value, None, value, name.sym());
                    }
                    PropertyAccessField::Expr(expr) => {
                        let key = self.register_allocator.alloc();
                        self.compile_expr(expr, &key);
                        self.bytecode.emit_get_property_by_value(
                            value.variable(),
                            key.variable(),
                            value.variable(),
                            value.variable(),
                        );
                        self.register_allocator.dealloc(key);
                    }
                }
            }
            OptionalOperationKind::PrivatePropertyAccess { field } => {
                self.bytecode.emit_move(this.variable(), value.variable());
                let index = self.get_or_insert_private_name(*field);
                self.bytecode.emit_get_private_field(
                    value.variable(),
                    value.variable(),
                    index.into(),
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

                    self.bytecode.emit_push_new_array(array.variable());

                    for arg in args {
                        self.compile_expr(arg, &value);
                        if let Expression::Spread(_) = arg {
                            self.bytecode.emit_get_iterator(value.variable());
                            self.bytecode.emit_push_iterator_to_array(array.variable());
                        } else {
                            self.bytecode
                                .emit_push_value_to_array(value.variable(), array.variable());
                        }
                    }

                    self.push_from_register(&array);

                    self.register_allocator.dealloc(value);
                    self.register_allocator.dealloc(array);
                } else {
                    for arg in args {
                        let value = self.register_allocator.alloc();
                        self.compile_expr(arg, &value);
                        self.push_from_register(&value);
                        self.register_allocator.dealloc(value);
                    }
                }

                self.emit_with_accumulator_stashed(|compiler| {
                    if contains_spread {
                        compiler.bytecode.emit_call_spread();
                    } else {
                        compiler.bytecode.emit_call((args.len() as u32).into());
                    }
                });

                self.pop_into_register(value);
                self.bytecode.emit_push_undefined(this.variable());
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
                        let index = self.insert_binding(binding);
                        let value = self.register_allocator.alloc();
                        self.emit_binding_access(BindingAccessOpcode::GetLocator, &index, &value);
                        self.compile_expr(expr, &value);
                        self.emit_binding_access(
                            BindingAccessOpcode::SetNameByLocator,
                            &index,
                            &value,
                        );
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
                        self.bytecode.emit_push_undefined(value.variable());
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
                                self.bytecode.emit_push_undefined(value.variable());
                            }
                            self.emit_binding(BindingOpcode::InitLexical, ident, &value);
                            self.register_allocator.dealloc(value);
                        }
                        Binding::Pattern(pattern) => {
                            let value = self.register_allocator.alloc();
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, &value);
                            } else {
                                self.bytecode.emit_push_undefined(value.variable());
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
                                self.bytecode.emit_push_undefined(value.variable());
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
                if self.annex_b_function_names.contains(&name.sym()) {
                    let name = name.to_js_string(self.interner());
                    let binding = self.lexical_scope.get_identifier_reference(name.clone());
                    let index = self.get_binding(&binding);

                    let value = self.register_allocator.alloc();
                    self.emit_binding_access(BindingAccessOpcode::GetName, &index, &value);
                    match self.variable_scope.set_mutable_binding_var(name.clone()) {
                        Ok(binding) => {
                            let index = self.get_binding(&binding);
                            self.emit_binding_access(BindingAccessOpcode::SetName, &index, &value);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_string(name);
                            self.bytecode.emit_throw_mutate_immutable(index.into());
                        }
                        Err(BindingLocatorError::Silent) => {}
                    }
                    self.register_allocator.dealloc(value);
                }
            }
            Declaration::ClassDeclaration(class) => self.compile_class(class.as_ref().into(), None),
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
            .source_path(self.source_path.clone())
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
            .source_path(self.source_path.clone())
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
            .source_path(self.source_path.clone())
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
                    if ident.sym() == Sym::EVAL {
                        kind = CallKind::CallEval;
                    }

                    if self.in_with {
                        let name = self.resolve_identifier_expect(*ident);
                        let binding = self.lexical_scope.get_identifier_reference(name);
                        let index = self.get_binding(&binding);
                        let index = match index {
                            BindingKind::Global(index) | BindingKind::Stack(index) => index,
                            BindingKind::Local(_) => {
                                unreachable!("with binding cannot be local")
                            }
                        };
                        let value = self.register_allocator.alloc();
                        self.bytecode
                            .emit_this_for_object_environment_name(value.variable(), index.into());
                        self.push_from_register(&value);
                        self.register_allocator.dealloc(value);
                    } else {
                        let value = self.register_allocator.alloc();
                        self.bytecode.emit_push_undefined(value.variable());
                        self.push_from_register(&value);
                        self.register_allocator.dealloc(value);
                    }
                } else {
                    let value = self.register_allocator.alloc();
                    self.bytecode.emit_push_undefined(value.variable());
                    self.push_from_register(&value);
                    self.register_allocator.dealloc(value);
                }

                let value = self.register_allocator.alloc();
                self.compile_expr(expr, &value);
                self.push_from_register(&value);
                self.register_allocator.dealloc(value);
            }
            expr => {
                let this = self.register_allocator.alloc();
                let value = self.register_allocator.alloc();
                self.compile_expr(expr, &value);
                self.bytecode.emit_push_undefined(this.variable());
                self.push_from_register(&this);
                self.push_from_register(&value);
                self.register_allocator.dealloc(this);
                self.register_allocator.dealloc(value);
            }
        }

        let mut compiler = self.position_guard(call);

        let contains_spread = call
            .args()
            .iter()
            .any(|arg| matches!(arg, Expression::Spread(_)));

        if contains_spread {
            let array = compiler.register_allocator.alloc();
            let value = compiler.register_allocator.alloc();

            compiler.bytecode.emit_push_new_array(array.variable());

            for arg in call.args() {
                compiler.compile_expr(arg, &value);
                if let Expression::Spread(_) = arg {
                    compiler.bytecode.emit_get_iterator(value.variable());
                    compiler
                        .bytecode
                        .emit_push_iterator_to_array(array.variable());
                } else {
                    compiler
                        .bytecode
                        .emit_push_value_to_array(value.variable(), array.variable());
                }
            }

            compiler.push_from_register(&array);

            compiler.register_allocator.dealloc(array);
            compiler.register_allocator.dealloc(value);
        } else {
            for arg in call.args() {
                let value = compiler.register_allocator.alloc();
                compiler.compile_expr(arg, &value);
                compiler.push_from_register(&value);
                compiler.register_allocator.dealloc(value);
            }
        }

        compiler.emit_with_accumulator_stashed(|compiler| match kind {
            CallKind::CallEval => {
                let scope_index = compiler.constants.len() as u32;
                let lexical_scope = compiler.lexical_scope.clone();
                compiler.constants.push(Constant::Scope(lexical_scope));
                if contains_spread {
                    compiler.bytecode.emit_call_eval_spread(scope_index.into());
                } else {
                    compiler
                        .bytecode
                        .emit_call_eval((call.args().len() as u32).into(), scope_index.into());
                }
            }
            CallKind::Call if contains_spread => compiler.bytecode.emit_call_spread(),
            CallKind::Call => {
                compiler
                    .bytecode
                    .emit_call((call.args().len() as u32).into());
            }
            CallKind::New if contains_spread => compiler.bytecode.emit_new_spread(),
            CallKind::New => compiler
                .bytecode
                .emit_new((call.args().len() as u32).into()),
        });
        compiler.pop_into_register(dst);
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

        let final_bytecode_len = self.next_opcode_location();

        let mapped_arguments_binding_indices = if self.emitted_mapped_arguments_object_opcode {
            MappedArguments::binding_indices(&self.params, &self.parameter_scope, self.interner)
        } else {
            ThinVec::default()
        };

        let register_count = self.register_allocator.finish();

        let source_map_entries = self.source_map_builder.build(final_bytecode_len);

        CodeBlock {
            length: self.length,
            register_count,
            this_mode: self.this_mode,
            parameter_length: self.params.as_ref().len() as u32,
            mapped_arguments_binding_indices,
            bytecode: self.bytecode.into_bytecode(),
            constants: self.constants,
            bindings: self.bindings.into_boxed_slice(),
            handlers: self.handlers,
            flags: Cell::new(self.code_block_flags),
            ic: self.ic.into_boxed_slice(),
            source_info: SourceInfo::new(
                SourceMap::new(source_map_entries, self.source_path),
                self.function_name,
                self.spanned_source_text,
            ),
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
