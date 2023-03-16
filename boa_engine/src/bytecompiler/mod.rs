//! This module contains the bytecode compiler.

mod class;
mod declaration;
mod expression;
mod function;
mod jump_control;
mod module;
mod statement;

use crate::{
    builtins::function::ThisMode,
    environments::{BindingLocator, CompileTimeEnvironment},
    vm::{BindingOpcode, CodeBlock, Opcode},
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
        FormalParameterList, Function, Generator, PrivateName,
    },
    operations::bound_names,
    pattern::Pattern,
    Declaration, Expression, Statement, StatementList, StatementListItem,
};
use boa_gc::{Gc, GcRefCell};
use boa_interner::{Interner, Sym};
use rustc_hash::FxHashMap;

pub(crate) use function::FunctionCompiler;
pub(crate) use jump_control::JumpControlInfo;

/// Describes how a node has been defined in the source code.
#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeKind {
    Declaration,
    Expression,
}

/// Describes the type of a function.
#[derive(Debug, Clone, Copy, PartialEq)]
enum FunctionKind {
    Ordinary,
    Arrow,
    AsyncArrow,
    Async,
    Generator,
    AsyncGenerator,
}

/// Describes the complete specification of a function node.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(single_use_lifetimes)]
struct FunctionSpec<'a> {
    kind: FunctionKind,
    name: Option<Identifier>,
    parameters: &'a FormalParameterList,
    body: &'a StatementList,
    has_binding_identifier: bool,
}

impl FunctionSpec<'_> {
    const fn is_arrow(&self) -> bool {
        matches!(self.kind, FunctionKind::Arrow | FunctionKind::AsyncArrow)
    }

    const fn is_async(&self) -> bool {
        matches!(
            self.kind,
            FunctionKind::Async | FunctionKind::AsyncGenerator | FunctionKind::AsyncArrow
        )
    }

    const fn is_generator(&self) -> bool {
        matches!(
            self.kind,
            FunctionKind::Generator | FunctionKind::AsyncGenerator
        )
    }
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
#[derive(Debug, Clone, Copy)]
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
pub struct ByteCompiler<'b, 'host> {
    /// Name of this function.
    pub(crate) function_name: Sym,

    /// Indicates if the function is an expression and has a binding identifier.
    pub(crate) has_binding_identifier: bool,

    /// The number of arguments expected.
    pub(crate) length: u32,

    /// Is this function in strict mode.
    pub(crate) strict: bool,

    /// \[\[ThisMode\]\]
    pub(crate) this_mode: ThisMode,

    /// Parameters passed to this function.
    pub(crate) params: FormalParameterList,

    /// Bytecode
    pub(crate) bytecode: Vec<u8>,

    /// Literals
    pub(crate) literals: Vec<JsValue>,

    /// Property field names.
    pub(crate) names: Vec<Identifier>,

    /// Private names.
    pub(crate) private_names: Vec<PrivateName>,

    /// Locators for all bindings in the codeblock.
    pub(crate) bindings: Vec<BindingLocator>,

    /// Number of binding for the function environment.
    pub(crate) num_bindings: usize,

    /// Functions inside this function
    pub(crate) functions: Vec<Gc<CodeBlock>>,

    /// The `arguments` binding location of the function, if set.
    pub(crate) arguments_binding: Option<BindingLocator>,

    /// Compile time environments in this function.
    pub(crate) compile_environments: Vec<Gc<GcRefCell<CompileTimeEnvironment>>>,

    /// The `[[IsClassConstructor]]` internal slot.
    pub(crate) is_class_constructor: bool,

    /// The `[[ClassFieldInitializerName]]` internal slot.
    pub(crate) class_field_initializer_name: Option<Sym>,

    /// Marks the location in the code where the function environment in pushed.
    /// This is only relevant for functions with expressions in the parameters.
    /// We execute the parameter expressions in the function code and push the function environment afterward.
    /// When the execution of the parameter expressions throws an error, we do not need to pop the function environment.
    pub(crate) function_environment_push_location: u32,

    literals_map: FxHashMap<Literal, u32>,
    names_map: FxHashMap<Identifier, u32>,
    private_names_map: FxHashMap<PrivateName, u32>,
    bindings_map: FxHashMap<BindingLocator, u32>,
    jump_info: Vec<JumpControlInfo>,
    in_async_generator: bool,
    json_parse: bool,
    context: &'b mut Context<'host>,
}

impl<'b, 'host> ByteCompiler<'b, 'host> {
    /// Represents a placeholder address that will be patched later.
    const DUMMY_ADDRESS: u32 = u32::MAX;

    /// Creates a new [`ByteCompiler`].
    #[inline]
    pub fn new(
        name: Sym,
        strict: bool,
        json_parse: bool,
        context: &'b mut Context<'host>,
    ) -> ByteCompiler<'b, 'host> {
        Self {
            function_name: name,
            strict,
            length: 0,
            bytecode: Vec::default(),
            literals: Vec::default(),
            names: Vec::default(),
            private_names: Vec::default(),
            bindings: Vec::default(),
            num_bindings: 0,
            functions: Vec::default(),
            has_binding_identifier: false,
            this_mode: ThisMode::Global,
            params: FormalParameterList::default(),
            arguments_binding: None,
            compile_environments: Vec::default(),
            is_class_constructor: false,
            class_field_initializer_name: None,
            function_environment_push_location: 0,

            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            private_names_map: FxHashMap::default(),
            bindings_map: FxHashMap::default(),
            jump_info: Vec::new(),
            in_async_generator: false,
            json_parse,
            context,
        }
    }

    fn interner(&self) -> &Interner {
        self.context.interner()
    }

    /// Push a compile time environment to the current `CodeBlock` and return it's index.
    fn push_compile_environment(
        &mut self,
        environment: Gc<GcRefCell<CompileTimeEnvironment>>,
    ) -> usize {
        let index = self.compile_environments.len();
        self.compile_environments.push(environment);
        index
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

        let index = self.names.len() as u32;
        self.names.push(name);
        self.names_map.insert(name, index);
        index
    }

    #[inline]
    fn get_or_insert_private_name(&mut self, name: PrivateName) -> u32 {
        if let Some(index) = self.private_names_map.get(&name) {
            return *index;
        }

        let index = self.private_names.len() as u32;
        self.private_names.push(name);
        self.private_names_map.insert(name, index);
        index
    }

    #[inline]
    fn get_or_insert_binding(&mut self, binding: BindingLocator) -> u32 {
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
                let binding = self.context.initialize_mutable_binding(name, true);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefVar, &[index]);
            }
            BindingOpcode::Let => {
                let binding = self.context.initialize_mutable_binding(name, false);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefLet, &[index]);
            }
            BindingOpcode::InitVar => {
                let binding = if self.context.has_binding(name) {
                    self.context.set_mutable_binding(name)
                } else {
                    self.context.initialize_mutable_binding(name, true)
                };
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefInitVar, &[index]);
            }
            BindingOpcode::InitLet => {
                let binding = self.context.initialize_mutable_binding(name, false);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefInitLet, &[index]);
            }
            BindingOpcode::InitArg => {
                let binding = self.context.initialize_mutable_binding(name, false);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefInitArg, &[index]);
            }
            BindingOpcode::InitConst => {
                let binding = self.context.initialize_immutable_binding(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefInitConst, &[index]);
            }
            BindingOpcode::SetName => {
                let binding = self.context.set_mutable_binding(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::SetName, &[index]);
            }
        }
    }

    fn next_opcode_location(&mut self) -> u32 {
        assert!(self.bytecode.len() < u32::MAX as usize);
        self.bytecode.len() as u32
    }

    fn emit(&mut self, opcode: Opcode, operands: &[u32]) {
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

    fn emit_opcode(&mut self, opcode: Opcode) {
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
        #[allow(clippy::float_cmp)]
        if f64::from(value as i32) == value {
            self.emit_push_integer(value as i32);
        } else {
            self.emit_opcode(Opcode::PushRational);
            self.emit_u64(value.to_bits());
        }
    }

    fn jump(&mut self) -> Label {
        let index = self.next_opcode_location();
        self.emit(Opcode::Jump, &[Self::DUMMY_ADDRESS]);
        Label { index }
    }

    fn jump_if_false(&mut self) -> Label {
        let index = self.next_opcode_location();
        self.emit(Opcode::JumpIfFalse, &[Self::DUMMY_ADDRESS]);

        Label { index }
    }

    fn jump_if_null_or_undefined(&mut self) -> Label {
        let index = self.next_opcode_location();
        self.emit(Opcode::JumpIfNullOrUndefined, &[Self::DUMMY_ADDRESS]);

        Label { index }
    }

    /// Emit an opcode with a dummy operand.
    /// Return the `Label` of the operand.
    fn emit_opcode_with_operand(&mut self, opcode: Opcode) -> Label {
        let index = self.next_opcode_location();
        self.emit(opcode, &[Self::DUMMY_ADDRESS]);
        Label { index }
    }

    /// Emit an opcode with two dummy operands.
    /// Return the `Label`s of the two operands.
    fn emit_opcode_with_two_operands(&mut self, opcode: Opcode) -> (Label, Label) {
        let index = self.next_opcode_location();
        self.emit(opcode, &[Self::DUMMY_ADDRESS, Self::DUMMY_ADDRESS]);
        (Label { index }, Label { index: index + 4 })
    }

    fn patch_jump_with_target(&mut self, label: Label, target: u32) {
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
                let binding = self.context.get_binding_value(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::GetName, &[index]);
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name((*name).into());
                        self.compile_expr(access.target(), true);
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(access.target(), true);
                        self.compile_expr(expr, true);
                        self.emit(Opcode::GetPropertyByValue, &[]);
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
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.emit_opcode(Opcode::Super);
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

    // The wrap is needed so it can match the function signature.
    #[allow(clippy::unnecessary_wraps)]
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
                expr_fn(self, 0);
                if use_expr {
                    self.emit(Opcode::Dup, &[]);
                }
                let binding = self.context.set_mutable_binding(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::SetName, &[index]);
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
                        self.compile_expr(expr, true);
                        expr_fn(self, 2);
                        self.emit(Opcode::SetPropertyByValue, &[]);
                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
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
                        self.emit(Opcode::Super, &[]);
                        self.compile_expr(expr, true);
                        expr_fn(self, 0);
                        self.emit(Opcode::SetPropertyByValue, &[]);
                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
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
                let binding = self.context.get_binding_value(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DeleteName, &[index]);
            }
            Access::This => {
                self.emit_opcode(Opcode::PushTrue);
            }
        }
    }

    /// Compile a [`StatementList`].
    pub fn compile_statement_list(
        &mut self,
        list: &StatementList,
        use_expr: bool,
        configurable_globals: bool,
    ) {
        if use_expr {
            let expr_index = list
                .statements()
                .iter()
                .rev()
                .skip_while(|item| {
                    matches!(
                        item,
                        &&StatementListItem::Statement(Statement::Empty | Statement::Var(_))
                            | &&StatementListItem::Declaration(_)
                    )
                })
                .count();

            for (i, item) in list.statements().iter().enumerate() {
                self.compile_stmt_list_item(item, i + 1 == expr_index, configurable_globals);
            }
        } else {
            for item in list.statements() {
                self.compile_stmt_list_item(item, false, configurable_globals);
            }
        }
    }

    /// Compile a statement list in a new declarative environment.
    pub(crate) fn compile_statement_list_with_new_declarative(
        &mut self,
        list: &StatementList,
        use_expr: bool,
        strict: bool,
    ) {
        self.context.push_compile_time_environment(strict);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        self.create_script_decls(list, true);

        if use_expr {
            let expr_index = list
                .statements()
                .iter()
                .rev()
                .skip_while(|item| {
                    matches!(
                        item,
                        &&StatementListItem::Statement(Statement::Empty | Statement::Var(_))
                            | &&StatementListItem::Declaration(_)
                    )
                })
                .count();

            for (i, item) in list.statements().iter().enumerate() {
                self.compile_stmt_list_item(item, i + 1 == expr_index, true);
            }
        } else {
            for item in list.statements() {
                self.compile_stmt_list_item(item, false, true);
            }
        }

        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        self.emit_opcode(Opcode::PopEnvironment);
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

        match optional.target() {
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

        self.emit_opcode(Opcode::Pop);
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
                match field {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name((*name).into());
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(expr, true);
                        self.emit(Opcode::GetPropertyByValue, &[]);
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
                            self.emit_opcode(Opcode::InitIterator);
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
                                self.emit_binding(BindingOpcode::InitLet, *ident);
                            } else {
                                self.emit_binding(BindingOpcode::Let, *ident);
                            }
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
    fn compile_stmt_list_item(
        &mut self,
        item: &StatementListItem,
        use_expr: bool,
        configurable_globals: bool,
    ) {
        match item {
            StatementListItem::Statement(stmt) => {
                self.compile_stmt(stmt, use_expr, configurable_globals);
            }
            StatementListItem::Declaration(decl) => self.compile_decl(decl),
        }
    }

    /// Compile a [`Declaration`].
    pub fn compile_decl(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Function(function) => {
                self.function(function.into(), NodeKind::Declaration, false);
            }
            Declaration::Generator(function) => {
                self.function(function.into(), NodeKind::Declaration, false);
            }
            Declaration::AsyncFunction(function) => {
                self.function(function.into(), NodeKind::Declaration, false);
            }
            Declaration::AsyncGenerator(function) => {
                self.function(function.into(), NodeKind::Declaration, false);
            }
            Declaration::Class(class) => self.class(class, false),
            Declaration::Lexical(lexical) => self.compile_lexical_decl(lexical),
        }
    }

    /// Compile a function AST Node into bytecode.
    fn function(&mut self, function: FunctionSpec<'_>, node_kind: NodeKind, use_expr: bool) {
        let (generator, r#async, arrow) = (
            function.is_generator(),
            function.is_async(),
            function.is_arrow(),
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
            .strict(self.strict)
            .arrow(arrow)
            .binding_identifier(binding_identifier)
            .compile(parameters, body, self.context);

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
        self.emit_u8(0);

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

    /// Compile a class method AST Node into bytecode.
    fn method(
        &mut self,
        function: FunctionSpec<'_>,
        node_kind: NodeKind,
        class_name: Sym,
        use_expr: bool,
    ) {
        let (generator, r#async, arrow) = (
            function.is_generator(),
            function.is_async(),
            function.is_arrow(),
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
            .compile(parameters, body, self.context);

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
        self.emit_u8(1);

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

    fn call(&mut self, callable: Callable<'_>, use_expr: bool) {
        #[derive(PartialEq)]
        enum CallKind {
            CallEval,
            Call,
            New,
        }

        let (call, kind) = match callable {
            Callable::Call(call) => match call.function() {
                Expression::Identifier(ident) if *ident == Sym::EVAL => (call, CallKind::CallEval),
                _ => (call, CallKind::Call),
            },
            Callable::New(new) => (new.call(), CallKind::New),
        };

        match call.function() {
            Expression::PropertyAccess(access) if kind == CallKind::Call => {
                self.compile_access_preserve_this(access);
            }

            Expression::Optional(opt) if kind == CallKind::Call => {
                self.compile_optional_preserve_this(opt);
            }
            expr => {
                self.compile_expr(expr, true);
                if kind == CallKind::Call || kind == CallKind::CallEval {
                    self.emit_opcode(Opcode::PushUndefined);
                    self.emit_opcode(Opcode::Swap);
                }
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
                    self.emit_opcode(Opcode::InitIterator);
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
            has_binding_identifier: self.has_binding_identifier,
            length: self.length,
            strict: self.strict,
            this_mode: self.this_mode,
            params: self.params,
            bytecode: self.bytecode.into_boxed_slice(),
            literals: self.literals.into_boxed_slice(),
            names: self.names.into_boxed_slice(),
            private_names: self.private_names.into_boxed_slice(),
            bindings: self.bindings.into_boxed_slice(),
            num_bindings: self.num_bindings,
            functions: self.functions.into_boxed_slice(),
            arguments_binding: self.arguments_binding,
            compile_environments: self.compile_environments.into_boxed_slice(),
            is_class_constructor: self.is_class_constructor,
            class_field_initializer_name: self.class_field_initializer_name,
            function_environment_push_location: self.function_environment_push_location,
        }
    }

    fn compile_declaration_pattern(&mut self, pattern: &Pattern, def: BindingOpcode) {
        self.compile_declaration_pattern_impl(pattern, def);
    }

    /// Creates the declarations for a sript.
    pub(crate) fn create_script_decls(
        &mut self,
        stmt_list: &StatementList,
        configurable_globals: bool,
    ) {
        for node in stmt_list.statements() {
            self.create_decls_from_stmt_list_item(node, configurable_globals);
        }
    }

    pub(crate) fn create_decls_from_var_decl(
        &mut self,
        list: &VarDeclaration,
        configurable_globals: bool,
    ) -> bool {
        let mut has_identifier_argument = false;
        for decl in list.0.as_ref() {
            match decl.binding() {
                Binding::Identifier(ident) => {
                    let ident = ident;
                    if *ident == Sym::ARGUMENTS {
                        has_identifier_argument = true;
                    }
                    self.context
                        .create_mutable_binding(*ident, true, configurable_globals);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        if ident == Sym::ARGUMENTS {
                            has_identifier_argument = true;
                        }
                        self.context
                            .create_mutable_binding(ident, true, configurable_globals);
                    }
                }
            }
        }
        has_identifier_argument
    }

    pub(crate) fn create_decls_from_lexical_decl(&mut self, list: &LexicalDeclaration) -> bool {
        let mut has_identifier_argument = false;
        match list {
            LexicalDeclaration::Let(list) => {
                for decl in list.as_ref() {
                    match decl.binding() {
                        Binding::Identifier(ident) => {
                            let ident = ident;
                            if *ident == Sym::ARGUMENTS {
                                has_identifier_argument = true;
                            }
                            self.context.create_mutable_binding(*ident, false, false);
                        }
                        Binding::Pattern(pattern) => {
                            for ident in bound_names(pattern) {
                                if ident == Sym::ARGUMENTS {
                                    has_identifier_argument = true;
                                }
                                self.context.create_mutable_binding(ident, false, false);
                            }
                        }
                    }
                }
            }
            LexicalDeclaration::Const(list) => {
                for decl in list.as_ref() {
                    match decl.binding() {
                        Binding::Identifier(ident) => {
                            let ident = ident;
                            if *ident == Sym::ARGUMENTS {
                                has_identifier_argument = true;
                            }
                            self.context.create_immutable_binding(*ident, true);
                        }
                        Binding::Pattern(pattern) => {
                            for ident in bound_names(pattern) {
                                if ident == Sym::ARGUMENTS {
                                    has_identifier_argument = true;
                                }
                                self.context.create_immutable_binding(ident, true);
                            }
                        }
                    }
                }
            }
        }
        has_identifier_argument
    }

    pub(crate) fn create_decls_from_decl(
        &mut self,
        declaration: &Declaration,
        configurable_globals: bool,
    ) -> bool {
        match declaration {
            Declaration::Lexical(decl) => self.create_decls_from_lexical_decl(decl),
            Declaration::Function(decl) => {
                let ident = decl.name().expect("function declaration must have a name");
                self.context
                    .create_mutable_binding(ident, true, configurable_globals);
                ident == Sym::ARGUMENTS
            }
            Declaration::Generator(decl) => {
                let ident = decl.name().expect("generator declaration must have a name");

                self.context
                    .create_mutable_binding(ident, true, configurable_globals);
                ident == Sym::ARGUMENTS
            }
            Declaration::AsyncFunction(decl) => {
                let ident = decl
                    .name()
                    .expect("async function declaration must have a name");
                self.context
                    .create_mutable_binding(ident, true, configurable_globals);
                ident == Sym::ARGUMENTS
            }
            Declaration::AsyncGenerator(decl) => {
                let ident = decl
                    .name()
                    .expect("async generator declaration must have a name");
                self.context
                    .create_mutable_binding(ident, true, configurable_globals);
                ident == Sym::ARGUMENTS
            }
            Declaration::Class(decl) => {
                let ident = decl.name().expect("class declaration must have a name");
                self.context
                    .create_mutable_binding(ident, false, configurable_globals);
                false
            }
        }
    }

    pub(crate) fn create_decls_from_stmt(
        &mut self,
        statement: &Statement,
        configurable_globals: bool,
    ) -> bool {
        match statement {
            Statement::Var(var) => self.create_decls_from_var_decl(var, configurable_globals),
            Statement::DoWhileLoop(do_while_loop) => {
                if !matches!(do_while_loop.body(), Statement::Block(_)) {
                    self.create_decls_from_stmt(do_while_loop.body(), configurable_globals);
                }
                false
            }
            Statement::ForInLoop(for_in_loop) => {
                if !matches!(for_in_loop.body(), Statement::Block(_)) {
                    self.create_decls_from_stmt(for_in_loop.body(), configurable_globals);
                }
                false
            }
            Statement::ForOfLoop(for_of_loop) => {
                if !matches!(for_of_loop.body(), Statement::Block(_)) {
                    self.create_decls_from_stmt(for_of_loop.body(), configurable_globals);
                }
                false
            }
            _ => false,
        }
    }

    pub(crate) fn create_decls_from_stmt_list_item(
        &mut self,
        item: &StatementListItem,
        configurable_globals: bool,
    ) -> bool {
        match item {
            StatementListItem::Declaration(decl) => {
                self.create_decls_from_decl(decl, configurable_globals)
            }
            StatementListItem::Statement(stmt) => {
                self.create_decls_from_stmt(stmt, configurable_globals)
            }
        }
    }

    fn class(&mut self, class: &Class, expression: bool) {
        self.compile_class(class, expression);
    }
}
