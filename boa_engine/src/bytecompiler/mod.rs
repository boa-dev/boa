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
    vm::{BindingOpcode, CodeBlock, CodeBlockFlags, Opcode},
    Context, JsBigInt, JsString, JsValue,
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

    /// Literals
    pub(crate) literals: Vec<JsValue>,

    /// Property field names and private name `[[Description]]`s.
    pub(crate) names: Vec<JsString>,

    /// Locators for all bindings in the codeblock.
    pub(crate) bindings: Vec<BindingLocator>,

    /// Functions inside this function
    pub(crate) functions: Vec<Gc<CodeBlock>>,

    /// Compile time environments in this function.
    pub(crate) compile_environments: Vec<Rc<CompileTimeEnvironment>>,

    /// The environment that is currently active.
    pub(crate) current_environment: Rc<CompileTimeEnvironment>,

    pub(crate) code_block_flags: CodeBlockFlags,

    literals_map: FxHashMap<Literal, u32>,
    names_map: FxHashMap<Identifier, u32>,
    bindings_map: FxHashMap<BindingLocator, u32>,
    jump_info: Vec<JumpControlInfo>,
    in_async_generator: bool,
    json_parse: bool,

    // TODO: remove when we separate scripts from the context
    context: &'ctx mut Context<'host>,

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
        current_environment: Rc<CompileTimeEnvironment>,
        // TODO: remove when we separate scripts from the context
        context: &'ctx mut Context<'host>,
    ) -> ByteCompiler<'ctx, 'host> {
        let mut code_block_flags = CodeBlockFlags::empty();
        code_block_flags.set(CodeBlockFlags::STRICT, strict);
        Self {
            function_name: name,
            length: 0,
            bytecode: Vec::default(),
            literals: Vec::default(),
            names: Vec::default(),
            bindings: Vec::default(),
            functions: Vec::default(),
            this_mode: ThisMode::Global,
            params: FormalParameterList::default(),
            compile_environments: Vec::default(),
            code_block_flags,

            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            bindings_map: FxHashMap::default(),
            jump_info: Vec::new(),
            in_async_generator: false,
            json_parse,
            current_environment,
            context,

            #[cfg(feature = "annex-b")]
            annex_b_function_names: Vec::new(),
        }
    }

    pub(crate) const fn strict(&self) -> bool {
        self.code_block_flags.contains(CodeBlockFlags::STRICT)
    }

    pub(crate) fn interner(&self) -> &Interner {
        self.context.interner()
    }

    fn get_or_insert_literal(&mut self, literal: Literal) -> u32 {
        if let Some(index) = self.literals_map.get(&literal) {
            return *index;
        }

        let value = match literal.clone() {
            Literal::String(value) => JsValue::new(value),
            Literal::BigInt(value) => JsValue::new(value),
        };

        let index = self.literals.len() as u32;
        self.literals.push(value);
        self.literals_map.insert(literal, index);
        index
    }

    fn get_or_insert_name(&mut self, name: Identifier) -> u32 {
        if let Some(index) = self.names_map.get(&name) {
            return *index;
        }

        let string = self.interner().resolve_expect(name.sym()).utf16();
        let index = self.names.len() as u32;
        self.names.push(js_string!(string));
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

    fn emit_binding(&mut self, opcode: BindingOpcode, name: Identifier) {
        match opcode {
            BindingOpcode::Var => {
                let binding = self.initialize_mutable_binding(name, true);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefVar, &[index]);
            }
            BindingOpcode::InitVar => {
                if self.has_binding(name) {
                    match self.set_mutable_binding(name) {
                        Ok(binding) => {
                            let index = self.get_or_insert_binding(binding);
                            self.emit(Opcode::DefInitVar, &[index]);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_name(name);
                            self.emit(Opcode::ThrowMutateImmutable, &[index]);
                        }
                        Err(BindingLocatorError::Silent) => {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                } else {
                    let binding = self.initialize_mutable_binding(name, true);
                    let index = self.get_or_insert_binding(binding);
                    self.emit(Opcode::DefInitVar, &[index]);
                };
            }
            BindingOpcode::InitLet => {
                let binding = self.initialize_mutable_binding(name, false);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::PutLexicalValue, &[index]);
            }
            BindingOpcode::InitConst => {
                let binding = self.initialize_immutable_binding(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::PutLexicalValue, &[index]);
            }
            BindingOpcode::SetName => match self.set_mutable_binding(name) {
                Ok(binding) => {
                    let index = self.get_or_insert_binding(binding);
                    self.emit(Opcode::SetName, &[index]);
                }
                Err(BindingLocatorError::MutateImmutable) => {
                    let index = self.get_or_insert_name(name);
                    self.emit(Opcode::ThrowMutateImmutable, &[index]);
                }
                Err(BindingLocatorError::Silent) => {
                    self.emit(Opcode::Pop, &[]);
                }
            },
        }
    }

    fn next_opcode_location(&mut self) -> u32 {
        assert!(self.bytecode.len() < u32::MAX as usize);
        self.bytecode.len() as u32
    }

    pub(crate) fn emit(&mut self, opcode: Opcode, operands: &[u32]) {
        self.emit_opcode(opcode);
        for operand in operands {
            self.emit_u32(*operand);
        }
    }

    fn emit_u64(&mut self, value: u64) {
        self.bytecode.extend(value.to_ne_bytes());
    }

    fn emit_u32(&mut self, value: u32) {
        self.bytecode.extend(value.to_ne_bytes());
    }

    fn emit_u16(&mut self, value: u16) {
        self.bytecode.extend(value.to_ne_bytes());
    }

    pub(crate) fn emit_opcode(&mut self, opcode: Opcode) {
        self.emit_u8(opcode as u8);
    }

    fn emit_u8(&mut self, value: u8) {
        self.bytecode.push(value);
    }

    fn emit_push_integer(&mut self, value: i32) {
        match value {
            0 => self.emit_opcode(Opcode::PushZero),
            1 => self.emit_opcode(Opcode::PushOne),
            x if i32::from(x as i8) == x => {
                self.emit_opcode(Opcode::PushInt8);
                self.emit_u8(x as i8 as u8);
            }
            x if i32::from(x as i16) == x => {
                self.emit_opcode(Opcode::PushInt16);
                self.emit_u16(x as i16 as u16);
            }
            x => self.emit(Opcode::PushInt32, &[x as _]),
        }
    }

    fn emit_push_literal(&mut self, literal: Literal) {
        let index = self.get_or_insert_literal(literal);
        self.emit(Opcode::PushLiteral, &[index]);
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
            self.emit_opcode(Opcode::PushRational);
            self.emit_u64(value.to_bits());
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

    /// Emit an opcode with a dummy operand.
    /// Return the `Label` of the operand.
    pub(crate) fn emit_opcode_with_operand(&mut self, opcode: Opcode) -> Label {
        let index = self.next_opcode_location();
        self.emit(opcode, &[Self::DUMMY_ADDRESS]);
        Label { index }
    }

    /// Emit an opcode with two dummy operands.
    /// Return the `Label`s of the two operands.
    pub(crate) fn emit_opcode_with_two_operands(&mut self, opcode: Opcode) -> (Label, Label) {
        let index = self.next_opcode_location();
        self.emit(opcode, &[Self::DUMMY_ADDRESS, Self::DUMMY_ADDRESS]);
        (Label { index }, Label { index: index + 4 })
    }

    /// Emit an opcode with three dummy operands.
    /// Return the `Label`s of the three operands.
    pub(crate) fn emit_opcode_with_three_operands(
        &mut self,
        opcode: Opcode,
    ) -> (Label, Label, Label) {
        let index = self.next_opcode_location();
        self.emit(
            opcode,
            &[
                Self::DUMMY_ADDRESS,
                Self::DUMMY_ADDRESS,
                Self::DUMMY_ADDRESS,
            ],
        );
        (
            Label { index },
            Label { index: index + 4 },
            Label { index: index + 8 },
        )
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
                let binding = self.get_binding_value(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::GetName, &[index]);
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name((*name).into());
                        self.compile_expr(access.target(), true);
                        self.emit_opcode(Opcode::Dup);
                        self.emit(Opcode::GetPropertyByName, &[index]);
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
                    self.emit(Opcode::GetPrivateField, &[index]);
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(field) => {
                        let index = self.get_or_insert_name((*field).into());
                        self.emit_opcode(Opcode::Super);
                        self.emit_opcode(Opcode::This);
                        self.emit(Opcode::GetPropertyByName, &[index]);
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
                self.emit(Opcode::This, &[]);
            }
        }

        if !use_expr {
            self.emit(Opcode::Pop, &[]);
        }
    }

    fn access_set_top_of_stack_expr_fn(compiler: &mut ByteCompiler<'_, '_>, level: u8) {
        match level {
            0 => {}
            1 => compiler.emit_opcode(Opcode::Swap),
            _ => {
                compiler.emit_opcode(Opcode::RotateLeft);
                compiler.emit_u8(level + 1);
            }
        }
    }

    fn access_set<F, R>(&mut self, access: Access<'_>, use_expr: bool, expr_fn: F)
    where
        F: FnOnce(&mut ByteCompiler<'_, '_>, u8) -> R,
    {
        match access {
            Access::Variable { name } => {
                let binding = self.get_binding_value(name);
                let index = self.get_or_insert_binding(binding);
                let lex = self.current_environment.is_lex_binding(name);

                if !lex {
                    self.emit(Opcode::GetLocator, &[index]);
                }

                expr_fn(self, 0);
                if use_expr {
                    self.emit(Opcode::Dup, &[]);
                }

                if lex {
                    match self.set_mutable_binding(name) {
                        Ok(binding) => {
                            let index = self.get_or_insert_binding(binding);
                            self.emit(Opcode::SetName, &[index]);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_name(name);
                            self.emit(Opcode::ThrowMutateImmutable, &[index]);
                        }
                        Err(BindingLocatorError::Silent) => {
                            self.emit(Opcode::Pop, &[]);
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

                        self.emit(Opcode::SetPropertyByName, &[index]);
                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
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
                    self.emit(Opcode::SetPrivateField, &[index]);
                    if !use_expr {
                        self.emit(Opcode::Pop, &[]);
                    }
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        self.emit_opcode(Opcode::Super);
                        self.emit_opcode(Opcode::This);
                        expr_fn(self, 1);
                        let index = self.get_or_insert_name((*name).into());
                        self.emit(Opcode::SetPropertyByName, &[index]);
                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
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
                        self.emit(Opcode::DeletePropertyByName, &[index]);
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
                let binding = self.get_binding_value(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DeleteName, &[index]);
            }
            Access::This => {
                self.emit_opcode(Opcode::PushTrue);
            }
        }
    }

    /// Compile a [`StatementList`].
    pub fn compile_statement_list(&mut self, list: &StatementList, use_expr: bool, block: bool) {
        if use_expr || self.jump_control_info_has_use_expr() {
            let mut has_returns_value = false;
            let mut use_expr_index = 0;
            let mut first_return_is_abrupt = false;
            for (i, statement) in list.statements().iter().enumerate() {
                match statement {
                    StatementListItem::Statement(Statement::Break(_) | Statement::Continue(_)) => {
                        if !has_returns_value {
                            first_return_is_abrupt = true;
                        }
                        break;
                    }
                    StatementListItem::Statement(Statement::Empty | Statement::Var(_))
                    | StatementListItem::Declaration(_) => {}
                    StatementListItem::Statement(Statement::Block(block))
                        if !returns_value(block) => {}
                    StatementListItem::Statement(_) => {
                        has_returns_value = true;
                        use_expr_index = i;
                    }
                }
            }

            if first_return_is_abrupt {
                self.emit_opcode(Opcode::PushUndefined);
                self.emit_opcode(Opcode::SetReturnValue);
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
                        self.emit(Opcode::GetPropertyByName, &[index]);
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
                self.emit(Opcode::GetPrivateField, &[index]);
            }
            PropertyAccess::Super(access) => {
                self.emit_opcode(Opcode::This);
                self.emit_opcode(Opcode::Super);
                self.emit_opcode(Opcode::This);
                match access.field() {
                    PropertyAccessField::Const(field) => {
                        let index = self.get_or_insert_name((*field).into());
                        self.emit(Opcode::GetPropertyByName, &[index]);
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
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(expr, true);
                        self.emit_opcode(Opcode::GetPropertyByValue);
                    }
                }
                self.emit_opcode(Opcode::RotateLeft);
                self.emit_u8(3);
                self.emit_opcode(Opcode::Pop);
            }
            OptionalOperationKind::PrivatePropertyAccess { field } => {
                self.emit_opcode(Opcode::Dup);
                let index = self.get_or_insert_private_name(*field);
                self.emit(Opcode::GetPrivateField, &[index]);
                self.emit_opcode(Opcode::RotateLeft);
                self.emit_u8(3);
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
                    self.emit(Opcode::Call, &[args.len() as u32]);
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
                    let ident = ident;
                    if let Some(expr) = variable.init() {
                        self.compile_expr(expr, true);
                        self.emit_binding(BindingOpcode::InitVar, *ident);
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
                            let ident = ident;
                            if let Some(expr) = variable.init() {
                                self.compile_expr(expr, true);
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            }
                            self.emit_binding(BindingOpcode::InitLet, *ident);
                        }
                        Binding::Pattern(pattern) => {
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, true);
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, BindingOpcode::InitLet);
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
                            self.emit_binding(BindingOpcode::InitConst, *ident);
                        }
                        Binding::Pattern(pattern) => {
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, true);
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, BindingOpcode::InitConst);
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
                    let binding = self.get_binding_value(name);
                    let index = self.get_or_insert_binding(binding);
                    self.emit(Opcode::GetName, &[index]);

                    match self.set_mutable_binding_var(name) {
                        Ok(binding) => {
                            let index = self.get_or_insert_binding(binding);
                            self.emit(Opcode::SetName, &[index]);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_name(name);
                            self.emit(Opcode::ThrowMutateImmutable, &[index]);
                        }
                        Err(BindingLocatorError::Silent) => {
                            self.emit(Opcode::Pop, &[]);
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
                self.current_environment.clone(),
                self.context,
            );

        let index = self.functions.len() as u32;
        self.functions.push(code);

        index
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
        let (generator, r#async, arrow) = (
            function.kind.is_generator(),
            function.kind.is_async(),
            function.kind.is_arrow(),
        );

        let index = self.function(function);

        if r#async && generator {
            self.emit(Opcode::GetGeneratorAsync, &[index]);
        } else if generator {
            self.emit(Opcode::GetGenerator, &[index]);
        } else if r#async && arrow {
            self.emit(Opcode::GetAsyncArrowFunction, &[index]);
        } else if r#async {
            self.emit(Opcode::GetFunctionAsync, &[index]);
        } else if arrow {
            self.emit(Opcode::GetArrowFunction, &[index]);
        } else {
            self.emit(Opcode::GetFunction, &[index]);
        }
        if !generator {
            self.emit_u8(0);
        }

        match node_kind {
            NodeKind::Declaration => {
                self.emit_binding(
                    BindingOpcode::InitVar,
                    name.expect("function declaration must have a name"),
                );
            }
            NodeKind::Expression => {
                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
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
            .binding_identifier(binding_identifier)
            .compile(
                parameters,
                body,
                self.current_environment.clone(),
                self.context,
            );

        let index = self.functions.len() as u32;
        self.functions.push(code);

        if r#async && generator {
            self.emit(Opcode::GetGeneratorAsync, &[index]);
        } else if generator {
            self.emit(Opcode::GetGenerator, &[index]);
        } else if r#async && arrow {
            self.emit(Opcode::GetAsyncArrowFunction, &[index]);
        } else if r#async {
            self.emit(Opcode::GetFunctionAsync, &[index]);
        } else if arrow {
            self.emit(Opcode::GetArrowFunction, &[index]);
        } else {
            self.emit(Opcode::GetFunction, &[index]);
        }
        if !generator {
            self.emit_u8(1);
        }
    }

    /// Compile a class method AST Node into bytecode.
    fn method(&mut self, function: FunctionSpec<'_>, class_name: Sym) {
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
            .binding_identifier(binding_identifier)
            .class_name(class_name)
            .compile(
                parameters,
                body,
                self.current_environment.clone(),
                self.context,
            );

        let index = self.functions.len() as u32;
        self.functions.push(code);

        if r#async && generator {
            self.emit(Opcode::GetGeneratorAsync, &[index]);
        } else if generator {
            self.emit(Opcode::GetGenerator, &[index]);
        } else if r#async && arrow {
            self.emit(Opcode::GetAsyncArrowFunction, &[index]);
        } else if r#async {
            self.emit(Opcode::GetFunctionAsync, &[index]);
        } else if arrow {
            self.emit(Opcode::GetArrowFunction, &[index]);
        } else {
            self.emit(Opcode::GetFunction, &[index]);
        }
        if !generator {
            self.emit_u8(1);
        }
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
                self.compile_expr(expr, true);
                self.emit_opcode(Opcode::PushUndefined);
                self.emit_opcode(Opcode::Swap);
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
            CallKind::CallEval => self.emit(Opcode::CallEval, &[call.args().len() as u32]),
            CallKind::Call if contains_spread => self.emit_opcode(Opcode::CallSpread),
            CallKind::Call => self.emit(Opcode::Call, &[call.args().len() as u32]),
            CallKind::New if contains_spread => self.emit_opcode(Opcode::NewSpread),
            CallKind::New => self.emit(Opcode::New, &[call.args().len() as u32]),
        }

        if !use_expr {
            self.emit(Opcode::Pop, &[]);
        }
    }

    /// Finish compiling code with the [`ByteCompiler`] and return the generated [`CodeBlock`].
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn finish(self) -> CodeBlock {
        CodeBlock {
            name: self.function_name,
            length: self.length,
            this_mode: self.this_mode,
            params: self.params,
            bytecode: self.bytecode.into_boxed_slice(),
            literals: self.literals.into_boxed_slice(),
            names: self.names.into_boxed_slice(),
            bindings: self.bindings.into_boxed_slice(),
            functions: self.functions.into_boxed_slice(),
            compile_environments: self.compile_environments.into_boxed_slice(),
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
