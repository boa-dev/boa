//! This module contains the bytecode compiler.

mod class;
mod declaration;
mod declarations;
mod env;
mod expression;
mod function;
mod jump_control;
mod module;
mod statement;
mod utils;

use std::{cell::Cell, rc::Rc};

use crate::{
    builtins::function::ThisMode,
    environments::{BindingLocator, BindingLocatorError, CompileTimeEnvironment},
    js_string,
    vm::{
        BindingOpcode, CodeBlock, CodeBlockFlags, Constant, GeneratorResumeKind, Handler, Opcode,
        VaryingOperandKind,
    },
    Context, JsBigInt, JsString,
};
use boa_ast::{
    declaration::{Binding, LexicalDeclaration, VarDeclaration},
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        operator::{assign::AssignTarget, update::UpdateTarget},
        Call, Identifier, New, Optional, OptionalOperationKind,
    },
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunction, AsyncGenerator, Class,
        FormalParameterList, Function, FunctionBody, Generator, PrivateName,
    },
    operations::returns_value,
    pattern::Pattern,
    Declaration, Expression, Statement, StatementList, StatementListItem,
};
use boa_gc::Gc;
use boa_interner::{Interner, Sym};
use rustc_hash::FxHashMap;

pub(crate) use function::FunctionCompiler;
pub(crate) use jump_control::JumpControlInfo;
use thin_vec::ThinVec;

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
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FunctionSpec<'a> {
    pub(crate) kind: FunctionKind,
    pub(crate) name: Option<Identifier>,
    parameters: &'a FormalParameterList,
    body: &'a FunctionBody,
    has_binding_identifier: bool,
}

impl<'a> From<&'a Function> for FunctionSpec<'a> {
    fn from(function: &'a Function) -> Self {
        FunctionSpec {
            kind: FunctionKind::Ordinary,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            has_binding_identifier: function.has_binding_identifier(),
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
            has_binding_identifier: false,
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
            has_binding_identifier: false,
        }
    }
}

impl<'a> From<&'a AsyncFunction> for FunctionSpec<'a> {
    fn from(function: &'a AsyncFunction) -> Self {
        FunctionSpec {
            kind: FunctionKind::Async,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            has_binding_identifier: function.has_binding_identifier(),
        }
    }
}

impl<'a> From<&'a Generator> for FunctionSpec<'a> {
    fn from(function: &'a Generator) -> Self {
        FunctionSpec {
            kind: FunctionKind::Generator,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            has_binding_identifier: function.has_binding_identifier(),
        }
    }
}

impl<'a> From<&'a AsyncGenerator> for FunctionSpec<'a> {
    fn from(function: &'a AsyncGenerator) -> Self {
        FunctionSpec {
            kind: FunctionKind::AsyncGenerator,
            name: function.name(),
            parameters: function.parameters(),
            body: function.body(),
            has_binding_identifier: function.has_binding_identifier(),
        }
    }
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
#[allow(unused)]
pub(crate) enum Operand {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    Varying(u32),
}

/// The [`ByteCompiler`] is used to compile ECMAScript AST from [`boa_ast`] to bytecode.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct ByteCompiler<'ctx, 'host> {
    /// Name of this function.
    pub(crate) function_name: Sym,

    /// The number of arguments expected.
    pub(crate) length: u32,

    /// \[\[ThisMode\]\]
    pub(crate) this_mode: ThisMode,

    /// Parameters passed to this function.
    pub(crate) params: FormalParameterList,

    /// Bytecode
    pub(crate) bytecode: Vec<u8>,

    pub(crate) constants: ThinVec<Constant>,

    /// Locators for all bindings in the codeblock.
    pub(crate) bindings: Vec<BindingLocator>,

    /// The current variable environment.
    pub(crate) variable_environment: Rc<CompileTimeEnvironment>,

    /// The current lexical environment.
    pub(crate) lexical_environment: Rc<CompileTimeEnvironment>,

    current_open_environments_count: u32,
    current_stack_value_count: u32,
    pub(crate) code_block_flags: CodeBlockFlags,
    handlers: ThinVec<Handler>,
    literals_map: FxHashMap<Literal, u32>,
    names_map: FxHashMap<Identifier, u32>,
    bindings_map: FxHashMap<BindingLocator, u32>,
    jump_info: Vec<JumpControlInfo>,

    /// Used to handle exception throws that escape the async function types.
    ///
    /// Async functions and async generator functions, need to be closed and resolved.
    pub(crate) async_handler: Option<u32>,
    json_parse: bool,

    // TODO: remove when we separate scripts from the context
    pub(crate) context: &'ctx mut Context<'host>,

    #[cfg(feature = "annex-b")]
    annex_b_function_names: Vec<Identifier>,
}

impl<'ctx, 'host> ByteCompiler<'ctx, 'host> {
    /// Represents a placeholder address that will be patched later.
    const DUMMY_ADDRESS: u32 = u32::MAX;
    const DUMMY_LABEL: Label = Label { index: u32::MAX };

    /// Creates a new [`ByteCompiler`].
    #[inline]
    pub(crate) fn new(
        name: Sym,
        strict: bool,
        json_parse: bool,
        variable_environment: Rc<CompileTimeEnvironment>,
        lexical_environment: Rc<CompileTimeEnvironment>,
        // TODO: remove when we separate scripts from the context
        context: &'ctx mut Context<'host>,
    ) -> ByteCompiler<'ctx, 'host> {
        let mut code_block_flags = CodeBlockFlags::empty();
        code_block_flags.set(CodeBlockFlags::STRICT, strict);
        code_block_flags |= CodeBlockFlags::HAS_PROTOTYPE_PROPERTY;
        Self {
            function_name: name,
            length: 0,
            bytecode: Vec::default(),
            constants: ThinVec::default(),
            bindings: Vec::default(),
            this_mode: ThisMode::Global,
            params: FormalParameterList::default(),
            current_open_environments_count: 0,

            // This starts at two because the first value is the `this` value, then function object.
            current_stack_value_count: 2,
            code_block_flags,
            handlers: ThinVec::default(),

            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            bindings_map: FxHashMap::default(),
            jump_info: Vec::new(),
            async_handler: None,
            json_parse,
            variable_environment,
            lexical_environment,
            context,

            #[cfg(feature = "annex-b")]
            annex_b_function_names: Vec::new(),
        }
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
        self.context.interner()
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

        let string = self.interner().resolve_expect(name.sym()).utf16();
        let index = self.constants.len() as u32;
        self.constants.push(Constant::String(js_string!(string)));
        self.names_map.insert(name, index);
        index
    }

    #[inline]
    fn get_or_insert_private_name(&mut self, name: PrivateName) -> u32 {
        self.get_or_insert_name(Identifier::new(name.description()))
    }

    #[inline]
    pub(crate) fn get_or_insert_binding(&mut self, binding: BindingLocator) -> u32 {
        if let Some(index) = self.bindings_map.get(&binding) {
            return *index;
        }

        let index = self.bindings.len() as u32;
        self.bindings.push(binding);
        self.bindings_map.insert(binding, index);
        index
    }

    #[inline]
    #[must_use]
    pub(crate) fn push_function_to_constants(&mut self, function: Gc<CodeBlock>) -> u32 {
        let index = self.constants.len() as u32;
        self.constants.push(Constant::Function(function));
        index
    }

    fn emit_binding(&mut self, opcode: BindingOpcode, name: Identifier) {
        match opcode {
            BindingOpcode::Var => {
                let binding = self.variable_environment.get_identifier_reference(name);
                if !binding.locator().is_global() {
                    let index = self.get_or_insert_binding(binding.locator());
                    self.emit_with_varying_operand(Opcode::DefVar, index);
                }
            }
            BindingOpcode::InitVar => match self.lexical_environment.set_mutable_binding(name) {
                Ok(binding) => {
                    let index = self.get_or_insert_binding(binding);
                    self.emit_with_varying_operand(Opcode::DefInitVar, index);
                }
                Err(BindingLocatorError::MutateImmutable) => {
                    let index = self.get_or_insert_name(name);
                    self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                }
                Err(BindingLocatorError::Silent) => {
                    self.emit_opcode(Opcode::Pop);
                }
            },
            BindingOpcode::InitLexical => {
                let binding = self.lexical_environment.get_identifier_reference(name);
                let index = self.get_or_insert_binding(binding.locator());
                self.emit_with_varying_operand(Opcode::PutLexicalValue, index);
            }
            BindingOpcode::SetName => match self.lexical_environment.set_mutable_binding(name) {
                Ok(binding) => {
                    let index = self.get_or_insert_binding(binding);
                    self.emit_with_varying_operand(Opcode::SetName, index);
                }
                Err(BindingLocatorError::MutateImmutable) => {
                    let index = self.get_or_insert_name(name);
                    self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                }
                Err(BindingLocatorError::Silent) => {
                    self.emit_opcode(Opcode::Pop);
                }
            },
        }
    }

    fn next_opcode_location(&mut self) -> u32 {
        assert!(self.bytecode.len() < u32::MAX as usize);
        self.bytecode.len() as u32
    }

    pub(crate) fn emit(&mut self, opcode: Opcode, operands: &[Operand]) {
        let mut varying_kind = VaryingOperandKind::U8;
        for operand in operands {
            if let Operand::Varying(operand) = *operand {
                if u8::try_from(operand).is_ok() {
                } else if u16::try_from(operand).is_ok() {
                    varying_kind = VaryingOperandKind::U16;
                } else {
                    varying_kind = VaryingOperandKind::U32;
                    break;
                }
            }
        }

        match varying_kind {
            VaryingOperandKind::U8 => {}
            VaryingOperandKind::U16 => self.emit_opcode(Opcode::U16Operands),
            VaryingOperandKind::U32 => self.emit_opcode(Opcode::U32Operands),
        }
        self.emit_opcode(opcode);
        for operand in operands {
            self.emit_operand(*operand, varying_kind);
        }
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

    pub(crate) fn emit_operand(&mut self, operand: Operand, varying_kind: VaryingOperandKind) {
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
        }
    }

    fn emit_i64(&mut self, value: i64) {
        self.emit_u64(value as u64);
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

    fn emit_push_integer(&mut self, value: i32) {
        match value {
            0 => self.emit_opcode(Opcode::PushZero),
            1 => self.emit_opcode(Opcode::PushOne),
            x if i32::from(x as i8) == x => {
                self.emit(Opcode::PushInt8, &[Operand::I8(x as i8)]);
            }
            x if i32::from(x as i16) == x => {
                self.emit(Opcode::PushInt16, &[Operand::I16(x as i16)]);
            }
            x => self.emit(Opcode::PushInt32, &[Operand::I32(x)]),
        }
    }

    fn emit_push_literal(&mut self, literal: Literal) {
        let index = self.get_or_insert_literal(literal);
        self.emit_with_varying_operand(Opcode::PushLiteral, index);
    }

    fn emit_push_rational(&mut self, value: f64) {
        if value.is_nan() {
            return self.emit_opcode(Opcode::PushNaN);
        }

        if value.is_infinite() {
            if value.is_sign_positive() {
                return self.emit_opcode(Opcode::PushPositiveInfinity);
            }
            return self.emit_opcode(Opcode::PushNegativeInfinity);
        }

        // Check if the f64 value can fit in an i32.
        if f64::from(value as i32).to_bits() == value.to_bits() {
            self.emit_push_integer(value as i32);
        } else {
            let f32_value = value as f32;

            #[allow(clippy::float_cmp)]
            if f64::from(f32_value) == value {
                self.emit(Opcode::PushFloat, &[Operand::U32(f32_value.to_bits())]);
            } else {
                self.emit(Opcode::PushDouble, &[Operand::U64(value.to_bits())]);
            }
        }
    }

    fn jump(&mut self) -> Label {
        self.emit_opcode_with_operand(Opcode::Jump)
    }

    fn jump_if_true(&mut self) -> Label {
        self.emit_opcode_with_operand(Opcode::JumpIfTrue)
    }

    fn jump_if_false(&mut self) -> Label {
        self.emit_opcode_with_operand(Opcode::JumpIfFalse)
    }

    fn jump_if_null_or_undefined(&mut self) -> Label {
        self.emit_opcode_with_operand(Opcode::JumpIfNullOrUndefined)
    }

    fn emit_resume_kind(&mut self, resume_kind: GeneratorResumeKind) {
        self.emit_push_integer(resume_kind as i32);
    }

    fn jump_if_not_resume_kind(&mut self, resume_kind: GeneratorResumeKind) -> Label {
        let label = self.emit_opcode_with_operand(Opcode::JumpIfNotResumeKind);
        self.emit_u8(resume_kind as u8);
        label
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

    /// Emit an opcode with a dummy operand.
    /// Return the `Label` of the operand.
    pub(crate) fn emit_opcode_with_operand(&mut self, opcode: Opcode) -> Label {
        let index = self.next_opcode_location();
        self.emit(opcode, &[Operand::U32(Self::DUMMY_ADDRESS)]);
        Label { index }
    }

    /// Emit an opcode with two dummy operands.
    /// Return the `Label`s of the two operands.
    pub(crate) fn emit_opcode_with_two_operands(&mut self, opcode: Opcode) -> (Label, Label) {
        let index = self.next_opcode_location();
        self.emit(
            opcode,
            &[
                Operand::U32(Self::DUMMY_ADDRESS),
                Operand::U32(Self::DUMMY_ADDRESS),
            ],
        );
        (Label { index }, Label { index: index + 4 })
    }

    pub(crate) fn patch_jump_with_target(&mut self, label: Label, target: u32) {
        const U32_SIZE: usize = std::mem::size_of::<u32>();

        let Label { index } = label;

        let index = index as usize;
        let bytes = target.to_ne_bytes();

        // This is done to avoid unneeded bounds checks.
        assert!(self.bytecode.len() > index + U32_SIZE && usize::MAX - U32_SIZE >= index);
        self.bytecode[index + 1..=index + U32_SIZE].copy_from_slice(bytes.as_slice());
    }

    fn patch_jump(&mut self, label: Label) {
        let target = self.next_opcode_location();
        self.patch_jump_with_target(label, target);
    }

    fn access_get(&mut self, access: Access<'_>, use_expr: bool) {
        match access {
            Access::Variable { name } => {
                let binding = self.lexical_environment.get_identifier_reference(name);
                let index = self.get_or_insert_binding(binding.locator());
                self.emit_with_varying_operand(Opcode::GetName, index);
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name((*name).into());
                        self.compile_expr(access.target(), true);
                        self.emit_opcode(Opcode::Dup);
                        self.emit_with_varying_operand(Opcode::GetPropertyByName, index);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(access.target(), true);
                        self.emit_opcode(Opcode::Dup);
                        self.compile_expr(expr, true);
                        self.emit_opcode(Opcode::GetPropertyByValue);
                    }
                },
                PropertyAccess::Private(access) => {
                    let index = self.get_or_insert_private_name(access.field());
                    self.compile_expr(access.target(), true);
                    self.emit_with_varying_operand(Opcode::GetPrivateField, index);
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(field) => {
                        let index = self.get_or_insert_name((*field).into());
                        self.emit_opcode(Opcode::Super);
                        self.emit_opcode(Opcode::This);
                        self.emit_with_varying_operand(Opcode::GetPropertyByName, index);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.emit_opcode(Opcode::Super);
                        self.emit_opcode(Opcode::This);
                        self.compile_expr(expr, true);
                        self.emit_opcode(Opcode::GetPropertyByValue);
                    }
                },
            },
            Access::This => {
                self.emit_opcode(Opcode::This);
            }
        }

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }

    fn access_set_top_of_stack_expr_fn(compiler: &mut ByteCompiler<'_, '_>, level: u8) {
        match level {
            0 => {}
            1 => compiler.emit_opcode(Opcode::Swap),
            _ => {
                compiler.emit(Opcode::RotateLeft, &[Operand::U8(level + 1)]);
            }
        }
    }

    fn access_set<F, R>(&mut self, access: Access<'_>, use_expr: bool, expr_fn: F)
    where
        F: FnOnce(&mut ByteCompiler<'_, '_>, u8) -> R,
    {
        match access {
            Access::Variable { name } => {
                let binding = self.lexical_environment.get_identifier_reference(name);
                let index = self.get_or_insert_binding(binding.locator());

                if !binding.is_lexical() {
                    self.emit_with_varying_operand(Opcode::GetLocator, index);
                }

                expr_fn(self, 0);
                if use_expr {
                    self.emit(Opcode::Dup, &[]);
                }

                if binding.is_lexical() {
                    match self.lexical_environment.set_mutable_binding(name) {
                        Ok(binding) => {
                            let index = self.get_or_insert_binding(binding);
                            self.emit_with_varying_operand(Opcode::SetName, index);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_name(name);
                            self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                        }
                        Err(BindingLocatorError::Silent) => {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                } else {
                    self.emit_opcode(Opcode::SetNameByLocator);
                }
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        self.compile_expr(access.target(), true);
                        self.emit_opcode(Opcode::Dup);
                        expr_fn(self, 2);
                        let index = self.get_or_insert_name((*name).into());

                        self.emit_with_varying_operand(Opcode::SetPropertyByName, index);
                        if !use_expr {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(access.target(), true);
                        self.emit_opcode(Opcode::Dup);
                        self.compile_expr(expr, true);
                        expr_fn(self, 3);
                        self.emit_opcode(Opcode::SetPropertyByValue);
                        if !use_expr {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                },
                PropertyAccess::Private(access) => {
                    self.compile_expr(access.target(), true);
                    expr_fn(self, 1);
                    let index = self.get_or_insert_private_name(access.field());
                    self.emit_with_varying_operand(Opcode::SetPrivateField, index);
                    if !use_expr {
                        self.emit_opcode(Opcode::Pop);
                    }
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        self.emit_opcode(Opcode::Super);
                        self.emit_opcode(Opcode::This);
                        expr_fn(self, 1);
                        let index = self.get_or_insert_name((*name).into());
                        self.emit_with_varying_operand(Opcode::SetPropertyByName, index);
                        if !use_expr {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.emit_opcode(Opcode::Super);
                        self.emit_opcode(Opcode::This);
                        self.compile_expr(expr, true);
                        expr_fn(self, 1);
                        self.emit_opcode(Opcode::SetPropertyByValue);
                        if !use_expr {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                },
            },
            Access::This => todo!("access_set `this`"),
        }
    }

    fn access_delete(&mut self, access: Access<'_>) {
        match access {
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name((*name).into());
                        self.compile_expr(access.target(), true);
                        self.emit_with_varying_operand(Opcode::DeletePropertyByName, index);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(access.target(), true);
                        self.compile_expr(expr, true);
                        self.emit_opcode(Opcode::DeletePropertyByValue);
                    }
                },
                // TODO: throw ReferenceError on super deletion.
                PropertyAccess::Super(_) => self.emit_opcode(Opcode::DeleteSuperThrow),
                PropertyAccess::Private(_) => {
                    unreachable!("deleting private properties should always throw early errors.")
                }
            },
            Access::Variable { name } => {
                let binding = self.lexical_environment.get_identifier_reference(name);
                let index = self.get_or_insert_binding(binding.locator());
                self.emit_with_varying_operand(Opcode::DeleteName, index);
            }
            Access::This => {
                self.emit_opcode(Opcode::PushTrue);
            }
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
    pub fn compile_expr(&mut self, expr: &Expression, use_expr: bool) {
        self.compile_expr_impl(expr, use_expr);
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
    fn compile_access_preserve_this(&mut self, access: &PropertyAccess) {
        match access {
            PropertyAccess::Simple(access) => {
                self.compile_expr(access.target(), true);
                self.emit_opcode(Opcode::Dup);
                self.emit_opcode(Opcode::Dup);
                match access.field() {
                    PropertyAccessField::Const(field) => {
                        let index = self.get_or_insert_name((*field).into());
                        self.emit_with_varying_operand(Opcode::GetPropertyByName, index);
                    }
                    PropertyAccessField::Expr(field) => {
                        self.compile_expr(field, true);
                        self.emit_opcode(Opcode::GetPropertyByValue);
                    }
                }
            }
            PropertyAccess::Private(access) => {
                self.compile_expr(access.target(), true);
                self.emit_opcode(Opcode::Dup);
                let index = self.get_or_insert_private_name(access.field());
                self.emit_with_varying_operand(Opcode::GetPrivateField, index);
            }
            PropertyAccess::Super(access) => {
                self.emit_opcode(Opcode::This);
                self.emit_opcode(Opcode::Super);
                self.emit_opcode(Opcode::This);
                match access.field() {
                    PropertyAccessField::Const(field) => {
                        let index = self.get_or_insert_name((*field).into());
                        self.emit_with_varying_operand(Opcode::GetPropertyByName, index);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(expr, true);
                        self.emit_opcode(Opcode::GetPropertyByValue);
                    }
                }
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
    fn compile_optional_preserve_this(&mut self, optional: &Optional) {
        let mut jumps = Vec::with_capacity(optional.chain().len());

        match optional.target().flatten() {
            Expression::PropertyAccess(access) => {
                self.compile_access_preserve_this(access);
            }
            Expression::Optional(opt) => self.compile_optional_preserve_this(opt),
            expr => {
                self.emit(Opcode::PushUndefined, &[]);
                self.compile_expr(expr, true);
            }
        }
        jumps.push(self.jump_if_null_or_undefined());

        let (first, rest) = optional
            .chain()
            .split_first()
            .expect("chain must have at least one element");
        assert!(first.shorted());

        self.compile_optional_item_kind(first.kind());

        for item in rest {
            if item.shorted() {
                jumps.push(self.jump_if_null_or_undefined());
            }
            self.compile_optional_item_kind(item.kind());
        }
        let skip_undef = self.jump();

        for label in jumps {
            self.patch_jump(label);
        }

        self.emit_opcode(Opcode::PushUndefined);

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
    /// is not null or undefined (if the operator `?.` was used).
    /// - This assumes that the state of the stack before compiling is `...rest, this, value`,
    /// since the operation compiled by this function could be a call.
    fn compile_optional_item_kind(&mut self, kind: &OptionalOperationKind) {
        match kind {
            OptionalOperationKind::SimplePropertyAccess { field } => {
                self.emit_opcode(Opcode::Dup);
                self.emit_opcode(Opcode::Dup);
                match field {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name((*name).into());
                        self.emit_with_varying_operand(Opcode::GetPropertyByName, index);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(expr, true);
                        self.emit_opcode(Opcode::GetPropertyByValue);
                    }
                }
                self.emit(Opcode::RotateLeft, &[Operand::U8(3)]);
                self.emit_opcode(Opcode::Pop);
            }
            OptionalOperationKind::PrivatePropertyAccess { field } => {
                self.emit_opcode(Opcode::Dup);
                let index = self.get_or_insert_private_name(*field);
                self.emit_with_varying_operand(Opcode::GetPrivateField, index);
                self.emit(Opcode::RotateLeft, &[Operand::U8(3)]);
                self.emit_opcode(Opcode::Pop);
            }
            OptionalOperationKind::Call { args } => {
                let args = &**args;
                let contains_spread = args.iter().any(|arg| matches!(arg, Expression::Spread(_)));

                if contains_spread {
                    self.emit_opcode(Opcode::PushNewArray);
                    for arg in args {
                        self.compile_expr(arg, true);
                        if let Expression::Spread(_) = arg {
                            self.emit_opcode(Opcode::GetIterator);
                            self.emit_opcode(Opcode::PushIteratorToArray);
                        } else {
                            self.emit_opcode(Opcode::PushValueToArray);
                        }
                    }
                    self.emit_opcode(Opcode::CallSpread);
                } else {
                    for arg in args {
                        self.compile_expr(arg, true);
                    }
                    self.emit_with_varying_operand(Opcode::Call, args.len() as u32);
                }

                self.emit_opcode(Opcode::PushUndefined);
                self.emit_opcode(Opcode::Swap);
            }
        }
    }

    /// Compile a [`VarDeclaration`].
    fn compile_var_decl(&mut self, decl: &VarDeclaration) {
        for variable in decl.0.as_ref() {
            match variable.binding() {
                Binding::Identifier(ident) => {
                    if let Some(expr) = variable.init() {
                        let binding = self.lexical_environment.get_identifier_reference(*ident);
                        let index = self.get_or_insert_binding(binding.locator());
                        self.emit_with_varying_operand(Opcode::GetLocator, index);
                        self.compile_expr(expr, true);
                        self.emit_opcode(Opcode::SetNameByLocator);
                    } else {
                        self.emit_binding(BindingOpcode::Var, *ident);
                    }
                }
                Binding::Pattern(pattern) => {
                    if let Some(init) = variable.init() {
                        self.compile_expr(init, true);
                    } else {
                        self.emit_opcode(Opcode::PushUndefined);
                    };

                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar);
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
                            if let Some(expr) = variable.init() {
                                self.compile_expr(expr, true);
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            }
                            self.emit_binding(BindingOpcode::InitLexical, *ident);
                        }
                        Binding::Pattern(pattern) => {
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, true);
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, BindingOpcode::InitLexical);
                        }
                    }
                }
            }
            LexicalDeclaration::Const(decls) => {
                for variable in decls.as_ref() {
                    match variable.binding() {
                        Binding::Identifier(ident) => {
                            let init = variable
                                .init()
                                .expect("const declaration must have initializer");
                            self.compile_expr(init, true);
                            self.emit_binding(BindingOpcode::InitLexical, *ident);
                        }
                        Binding::Pattern(pattern) => {
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, true);
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, BindingOpcode::InitLexical);
                        }
                    }
                }
            }
        };
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
            Declaration::Function(function) if block => {
                let name = function
                    .name()
                    .expect("function declaration must have name");
                if self.annex_b_function_names.contains(&name) {
                    let binding = self.lexical_environment.get_identifier_reference(name);
                    let index = self.get_or_insert_binding(binding.locator());
                    self.emit_with_varying_operand(Opcode::GetName, index);

                    match self.variable_environment.set_mutable_binding_var(name) {
                        Ok(binding) => {
                            let index = self.get_or_insert_binding(binding);
                            self.emit_with_varying_operand(Opcode::SetName, index);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_name(name);
                            self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                        }
                        Err(BindingLocatorError::Silent) => {
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                }
            }
            Declaration::Class(class) => self.class(class, false),
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
            has_binding_identifier,
            ..
        } = function;

        let binding_identifier = if has_binding_identifier {
            if let Some(name) = name {
                Some(name.sym())
            } else {
                Some(Sym::EMPTY_STRING)
            }
        } else {
            None
        };

        let code = FunctionCompiler::new()
            .name(name.map(Identifier::sym))
            .generator(generator)
            .r#async(r#async)
            .strict(self.strict())
            .arrow(arrow)
            .binding_identifier(binding_identifier)
            .compile(
                parameters,
                body,
                self.variable_environment.clone(),
                self.lexical_environment.clone(),
                self.context,
            );

        self.push_function_to_constants(code)
    }

    /// Compiles a function AST Node into bytecode, setting its corresponding binding or
    /// pushing it to the stack if necessary.
    pub(crate) fn function_with_binding(
        &mut self,
        function: FunctionSpec<'_>,
        node_kind: NodeKind,
        use_expr: bool,
    ) {
        let name = function.name;

        let index = self.function(function);
        self.emit_with_varying_operand(Opcode::GetFunction, index);

        match node_kind {
            NodeKind::Declaration => {
                self.emit_binding(
                    BindingOpcode::InitVar,
                    name.expect("function declaration must have a name"),
                );
            }
            NodeKind::Expression => {
                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
        }
    }

    /// Compile an object method AST Node into bytecode.
    pub(crate) fn object_method(&mut self, function: FunctionSpec<'_>) {
        let (generator, r#async, arrow) = (
            function.kind.is_generator(),
            function.kind.is_async(),
            function.kind.is_arrow(),
        );
        let FunctionSpec {
            name,
            parameters,
            body,
            has_binding_identifier,
            ..
        } = function;

        let binding_identifier = if has_binding_identifier {
            if let Some(name) = name {
                Some(name.sym())
            } else {
                Some(Sym::EMPTY_STRING)
            }
        } else {
            None
        };

        let code = FunctionCompiler::new()
            .name(name.map(Identifier::sym))
            .generator(generator)
            .r#async(r#async)
            .strict(self.strict())
            .arrow(arrow)
            .method(true)
            .binding_identifier(binding_identifier)
            .compile(
                parameters,
                body,
                self.variable_environment.clone(),
                self.lexical_environment.clone(),
                self.context,
            );

        let index = self.push_function_to_constants(code);
        self.emit_with_varying_operand(Opcode::GetFunction, index);
    }

    /// Compile a class method AST Node into bytecode.
    fn method(&mut self, function: FunctionSpec<'_>) {
        let (generator, r#async, arrow) = (
            function.kind.is_generator(),
            function.kind.is_async(),
            function.kind.is_arrow(),
        );
        let FunctionSpec {
            name,
            parameters,
            body,
            has_binding_identifier,
            ..
        } = function;

        let binding_identifier = if has_binding_identifier {
            if let Some(name) = name {
                Some(name.sym())
            } else {
                Some(Sym::EMPTY_STRING)
            }
        } else {
            None
        };

        let code = FunctionCompiler::new()
            .name(name.map(Identifier::sym))
            .generator(generator)
            .r#async(r#async)
            .strict(true)
            .arrow(arrow)
            .method(true)
            .binding_identifier(binding_identifier)
            .compile(
                parameters,
                body,
                self.variable_environment.clone(),
                self.lexical_environment.clone(),
                self.context,
            );

        let index = self.push_function_to_constants(code);
        self.emit_with_varying_operand(Opcode::GetFunction, index);
    }

    fn call(&mut self, callable: Callable<'_>, use_expr: bool) {
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
                self.compile_access_preserve_this(access);
            }

            Expression::Optional(opt) if kind == CallKind::Call => {
                self.compile_optional_preserve_this(opt);
            }
            expr if kind == CallKind::Call => {
                if let Expression::Identifier(ident) = expr {
                    if *ident == Sym::EVAL {
                        kind = CallKind::CallEval;
                    }
                }
                self.emit_opcode(Opcode::PushUndefined);
                self.compile_expr(expr, true);
            }
            expr => {
                self.compile_expr(expr, true);
            }
        }

        let contains_spread = call
            .args()
            .iter()
            .any(|arg| matches!(arg, Expression::Spread(_)));

        if contains_spread {
            self.emit_opcode(Opcode::PushNewArray);
            for arg in call.args() {
                self.compile_expr(arg, true);
                if let Expression::Spread(_) = arg {
                    self.emit_opcode(Opcode::GetIterator);
                    self.emit_opcode(Opcode::PushIteratorToArray);
                } else {
                    self.emit_opcode(Opcode::PushValueToArray);
                }
            }
        } else {
            for arg in call.args() {
                self.compile_expr(arg, true);
            }
        }

        match kind {
            CallKind::CallEval if contains_spread => self.emit_opcode(Opcode::CallEvalSpread),
            CallKind::CallEval => {
                self.emit_with_varying_operand(Opcode::CallEval, call.args().len() as u32);
            }
            CallKind::Call if contains_spread => self.emit_opcode(Opcode::CallSpread),
            CallKind::Call => {
                self.emit_with_varying_operand(Opcode::Call, call.args().len() as u32);
            }
            CallKind::New if contains_spread => self.emit_opcode(Opcode::NewSpread),
            CallKind::New => self.emit_with_varying_operand(Opcode::New, call.args().len() as u32),
        }

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
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

        let name = self
            .context
            .interner()
            .resolve_expect(self.function_name)
            .utf16()
            .into();

        CodeBlock {
            name,
            length: self.length,
            this_mode: self.this_mode,
            params: self.params,
            bytecode: self.bytecode.into_boxed_slice(),
            constants: self.constants,
            bindings: self.bindings.into_boxed_slice(),
            handlers: self.handlers,
            flags: Cell::new(self.code_block_flags),
        }
    }

    fn compile_declaration_pattern(&mut self, pattern: &Pattern, def: BindingOpcode) {
        self.compile_declaration_pattern_impl(pattern, def);
    }

    fn class(&mut self, class: &Class, expression: bool) {
        self.compile_class(class, expression);
    }
}
