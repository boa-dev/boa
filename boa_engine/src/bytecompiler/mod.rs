mod function;

use crate::{
    environments::{BindingLocator, CompileTimeEnvironment},
    syntax::ast::{
        declaration::{Binding, LexicalDeclaration, VarDeclaration},
        expression::{
            access::{PrivatePropertyAccess, PropertyAccess, PropertyAccessField},
            literal::{self, TemplateElement},
            operator::{
                assign::{op::AssignOp, AssignTarget},
                binary::op::{ArithmeticOp, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
                unary::op::UnaryOp,
            },
            Call, Identifier, New,
        },
        function::{
            ArrowFunction, AsyncFunction, AsyncGenerator, Class, ClassElement, FormalParameterList,
            Function, Generator,
        },
        pattern::{Pattern, PatternArrayElement, PatternObjectElement},
        property::{MethodDefinition, PropertyDefinition, PropertyName},
        statement::{
            iteration::{for_loop::ForLoopInitializer, IterableLoopInitializer},
            Block, DoWhileLoop, ForInLoop, ForLoop, ForOfLoop, LabelledItem, WhileLoop,
        },
        Declaration, Expression, Statement, StatementList, StatementListItem,
    },
    vm::{BindingOpcode, CodeBlock, Opcode},
    Context, JsBigInt, JsResult, JsString, JsValue,
};
use boa_gc::Gc;
use boa_interner::{Interner, Sym};
use rustc_hash::FxHashMap;
use std::mem::size_of;

pub(crate) use function::FunctionCompiler;

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
    Async,
    Generator,
    AsyncGenerator,
}

/// Describes the complete specification of a function node.
#[derive(Debug, Clone, Copy, PartialEq)]
struct FunctionSpec<'a> {
    kind: FunctionKind,
    name: Option<Identifier>,
    parameters: &'a FormalParameterList,
    body: &'a StatementList,
}

impl<'a> FunctionSpec<'a> {
    #[inline]
    fn is_arrow(&self) -> bool {
        self.kind == FunctionKind::Arrow
    }
    #[inline]
    fn is_async(&self) -> bool {
        matches!(
            self.kind,
            FunctionKind::Async | FunctionKind::AsyncGenerator
        )
    }
    #[inline]
    fn is_generator(&self) -> bool {
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
struct Label {
    index: u32,
}

#[derive(Debug, Clone)]
struct JumpControlInfo {
    label: Option<Sym>,
    start_address: u32,
    kind: JumpControlInfoKind,
    breaks: Vec<Label>,
    try_continues: Vec<Label>,
    in_catch: bool,
    has_finally: bool,
    finally_start: Option<Label>,
    for_of_in_loop: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum JumpControlInfoKind {
    Loop,
    Switch,
    Try,
    LabelledBlock,
}

#[derive(Debug, Clone, Copy)]
enum Access<'a> {
    Variable { name: Identifier },
    Property { access: &'a PropertyAccess },
    PrivateProperty { access: &'a PrivatePropertyAccess },
    SuperProperty { field: &'a PropertyAccessField },
    This,
}

#[derive(Debug)]
pub struct ByteCompiler<'b> {
    code_block: CodeBlock,
    literals_map: FxHashMap<Literal, u32>,
    names_map: FxHashMap<Identifier, u32>,
    bindings_map: FxHashMap<BindingLocator, u32>,
    jump_info: Vec<JumpControlInfo>,
    in_async_generator: bool,
    context: &'b mut Context,
}

impl<'b> ByteCompiler<'b> {
    /// Represents a placeholder address that will be patched later.
    const DUMMY_ADDRESS: u32 = u32::MAX;

    #[inline]
    pub fn new(name: Sym, strict: bool, context: &'b mut Context) -> Self {
        Self {
            code_block: CodeBlock::new(name, 0, strict),
            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            bindings_map: FxHashMap::default(),
            jump_info: Vec::new(),
            in_async_generator: false,
            context,
        }
    }

    #[inline]
    fn interner(&self) -> &Interner {
        self.context.interner()
    }

    /// Push a compile time environment to the current `CodeBlock` and return it's index.
    #[inline]
    fn push_compile_environment(
        &mut self,
        environment: Gc<boa_gc::Cell<CompileTimeEnvironment>>,
    ) -> usize {
        let index = self.code_block.compile_environments.len();
        self.code_block.compile_environments.push(environment);
        index
    }

    #[inline]
    fn get_or_insert_literal(&mut self, literal: Literal) -> u32 {
        if let Some(index) = self.literals_map.get(&literal) {
            return *index;
        }

        let value = match literal.clone() {
            Literal::String(value) => JsValue::new(value),
            Literal::BigInt(value) => JsValue::new(value),
        };

        let index = self.code_block.literals.len() as u32;
        self.code_block.literals.push(value);
        self.literals_map.insert(literal, index);
        index
    }

    #[inline]
    fn get_or_insert_name(&mut self, name: Identifier) -> u32 {
        if let Some(index) = self.names_map.get(&name) {
            return *index;
        }

        let index = self.code_block.names.len() as u32;
        self.code_block.names.push(name);
        self.names_map.insert(name, index);
        index
    }

    #[inline]
    fn get_or_insert_binding(&mut self, binding: BindingLocator) -> u32 {
        if let Some(index) = self.bindings_map.get(&binding) {
            return *index;
        }

        let index = self.code_block.bindings.len() as u32;
        self.code_block.bindings.push(binding);
        self.bindings_map.insert(binding, index);
        index
    }

    #[inline]
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

    #[inline]
    fn next_opcode_location(&mut self) -> u32 {
        assert!(self.code_block.code.len() < u32::MAX as usize);
        self.code_block.code.len() as u32
    }

    #[inline]
    fn emit(&mut self, opcode: Opcode, operands: &[u32]) {
        self.emit_opcode(opcode);
        for operand in operands {
            self.emit_u32(*operand);
        }
    }

    #[inline]
    fn emit_u64(&mut self, value: u64) {
        self.code_block.code.extend(&value.to_ne_bytes());
    }

    #[inline]
    fn emit_u32(&mut self, value: u32) {
        self.code_block.code.extend(&value.to_ne_bytes());
    }

    #[inline]
    fn emit_u16(&mut self, value: u16) {
        self.code_block.code.extend(&value.to_ne_bytes());
    }

    #[inline]
    fn emit_opcode(&mut self, opcode: Opcode) {
        self.emit_u8(opcode as u8);
    }

    #[inline]
    fn emit_u8(&mut self, value: u8) {
        self.code_block.code.push(value);
    }

    #[inline]
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

    #[inline]
    fn emit_push_literal(&mut self, literal: Literal) {
        let index = self.get_or_insert_literal(literal);
        self.emit(Opcode::PushLiteral, &[index]);
    }

    #[inline]
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

    #[inline]
    fn jump(&mut self) -> Label {
        let index = self.next_opcode_location();
        self.emit(Opcode::Jump, &[Self::DUMMY_ADDRESS]);
        Label { index }
    }

    #[inline]
    fn jump_if_false(&mut self) -> Label {
        let index = self.next_opcode_location();
        self.emit(Opcode::JumpIfFalse, &[Self::DUMMY_ADDRESS]);

        Label { index }
    }

    /// Emit an opcode with a dummy operand.
    /// Return the `Label` of the operand.
    #[inline]
    fn emit_opcode_with_operand(&mut self, opcode: Opcode) -> Label {
        let index = self.next_opcode_location();
        self.emit(opcode, &[Self::DUMMY_ADDRESS]);
        Label { index }
    }

    /// Emit an opcode with two dummy operands.
    /// Return the `Label`s of the two operands.
    #[inline]
    fn emit_opcode_with_two_operands(&mut self, opcode: Opcode) -> (Label, Label) {
        let index = self.next_opcode_location();
        self.emit(opcode, &[Self::DUMMY_ADDRESS, Self::DUMMY_ADDRESS]);
        (Label { index }, Label { index: index + 4 })
    }

    #[inline]
    fn patch_jump_with_target(&mut self, label: Label, target: u32) {
        let Label { index } = label;

        let index = index as usize;

        let bytes = target.to_ne_bytes();
        self.code_block.code[index + 1] = bytes[0];
        self.code_block.code[index + 2] = bytes[1];
        self.code_block.code[index + 3] = bytes[2];
        self.code_block.code[index + 4] = bytes[3];
    }

    #[inline]
    fn patch_jump(&mut self, label: Label) {
        let target = self.next_opcode_location();
        self.patch_jump_with_target(label, target);
    }

    #[inline]
    fn push_loop_control_info(&mut self, label: Option<Sym>, start_address: u32) {
        self.jump_info.push(JumpControlInfo {
            label,
            start_address,
            kind: JumpControlInfoKind::Loop,
            breaks: Vec::new(),
            try_continues: Vec::new(),
            in_catch: false,
            has_finally: false,
            finally_start: None,
            for_of_in_loop: false,
        });
    }

    #[inline]
    fn push_loop_control_info_for_of_in_loop(&mut self, label: Option<Sym>, start_address: u32) {
        self.jump_info.push(JumpControlInfo {
            label,
            start_address,
            kind: JumpControlInfoKind::Loop,
            breaks: Vec::new(),
            try_continues: Vec::new(),
            in_catch: false,
            has_finally: false,
            finally_start: None,
            for_of_in_loop: true,
        });
    }

    #[inline]
    fn pop_loop_control_info(&mut self) {
        let loop_info = self.jump_info.pop().expect("no jump information found");

        assert!(loop_info.kind == JumpControlInfoKind::Loop);

        for label in loop_info.breaks {
            self.patch_jump(label);
        }

        for label in loop_info.try_continues {
            self.patch_jump_with_target(label, loop_info.start_address);
        }
    }

    #[inline]
    fn push_switch_control_info(&mut self, label: Option<Sym>, start_address: u32) {
        self.jump_info.push(JumpControlInfo {
            label,
            start_address,
            kind: JumpControlInfoKind::Switch,
            breaks: Vec::new(),
            try_continues: Vec::new(),
            in_catch: false,
            has_finally: false,
            finally_start: None,
            for_of_in_loop: false,
        });
    }

    #[inline]
    fn pop_switch_control_info(&mut self) {
        let info = self.jump_info.pop().expect("no jump information found");

        assert!(info.kind == JumpControlInfoKind::Switch);

        for label in info.breaks {
            self.patch_jump(label);
        }
    }

    #[inline]
    fn push_try_control_info(&mut self, has_finally: bool) {
        if !self.jump_info.is_empty() {
            let start_address = self
                .jump_info
                .last()
                .expect("no jump information found")
                .start_address;

            self.jump_info.push(JumpControlInfo {
                label: None,
                start_address,
                kind: JumpControlInfoKind::Try,
                breaks: Vec::new(),
                try_continues: Vec::new(),
                in_catch: false,
                has_finally,
                finally_start: None,
                for_of_in_loop: false,
            });
        }
    }

    #[inline]
    fn push_try_control_info_catch_start(&mut self) {
        if !self.jump_info.is_empty() {
            let mut info = self
                .jump_info
                .last_mut()
                .expect("must have try control label");
            assert!(info.kind == JumpControlInfoKind::Try);
            info.in_catch = true;
        }
    }

    #[inline]
    fn push_try_control_info_finally_start(&mut self, start: Label) {
        if !self.jump_info.is_empty() {
            let mut info = self
                .jump_info
                .last_mut()
                .expect("must have try control label");
            assert!(info.kind == JumpControlInfoKind::Try);
            info.finally_start = Some(start);
        }
    }

    #[inline]
    fn pop_try_control_info(&mut self, finally_start_address: Option<u32>) {
        if !self.jump_info.is_empty() {
            let mut info = self.jump_info.pop().expect("no jump information found");

            assert!(info.kind == JumpControlInfoKind::Try);

            let mut breaks = Vec::with_capacity(info.breaks.len());

            if let Some(finally_start_address) = finally_start_address {
                for label in info.try_continues {
                    if label.index < finally_start_address {
                        self.patch_jump_with_target(label, finally_start_address);
                    } else {
                        self.patch_jump_with_target(label, info.start_address);
                    }
                }

                for label in info.breaks {
                    if label.index < finally_start_address {
                        self.patch_jump_with_target(label, finally_start_address);
                        let Label { mut index } = label;
                        index -= size_of::<Opcode>() as u32;
                        index -= size_of::<u32>() as u32;
                        breaks.push(Label { index });
                    } else {
                        breaks.push(label);
                    }
                }
                if let Some(jump_info) = self.jump_info.last_mut() {
                    jump_info.breaks.append(&mut breaks);
                }
            } else if let Some(jump_info) = self.jump_info.last_mut() {
                jump_info.breaks.append(&mut info.breaks);
                jump_info.try_continues.append(&mut info.try_continues);
            }
        }
    }

    #[inline]
    fn push_labelled_block_control_info(&mut self, label: Sym, start_address: u32) {
        self.jump_info.push(JumpControlInfo {
            label: Some(label),
            start_address,
            kind: JumpControlInfoKind::LabelledBlock,
            breaks: Vec::new(),
            try_continues: Vec::new(),
            in_catch: false,
            has_finally: false,
            finally_start: None,
            for_of_in_loop: false,
        });
    }

    #[inline]
    fn pop_labelled_block_control_info(&mut self) {
        let info = self.jump_info.pop().expect("no jump information found");

        assert!(info.kind == JumpControlInfoKind::LabelledBlock);

        for label in info.breaks {
            self.patch_jump(label);
        }

        for label in info.try_continues {
            self.patch_jump_with_target(label, info.start_address);
        }
    }

    #[inline]
    fn compile_access(expr: &Expression) -> Option<Access<'_>> {
        match expr {
            Expression::Identifier(name) => Some(Access::Variable { name: *name }),
            Expression::PropertyAccess(access) => Some(Access::Property { access }),
            Expression::This => Some(Access::This),
            _ => None,
        }
    }

    #[inline]
    fn access_get(&mut self, access: Access<'_>, use_expr: bool) -> JsResult<()> {
        match access {
            Access::Variable { name } => {
                let binding = self.context.get_binding_value(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::GetName, &[index]);
            }
            Access::Property { access } => match access.field() {
                PropertyAccessField::Const(name) => {
                    let index = self.get_or_insert_name((*name).into());
                    self.compile_expr(access.target(), true)?;
                    self.emit(Opcode::GetPropertyByName, &[index]);
                }
                PropertyAccessField::Expr(expr) => {
                    self.compile_expr(expr, true)?;
                    self.compile_expr(access.target(), true)?;
                    self.emit(Opcode::GetPropertyByValue, &[]);
                }
            },
            Access::PrivateProperty { access } => {
                let index = self.get_or_insert_name(access.field().into());
                self.compile_expr(access.target(), true)?;
                self.emit(Opcode::GetPrivateField, &[index]);
            }
            Access::SuperProperty { field } => match field {
                PropertyAccessField::Const(field) => {
                    let index = self.get_or_insert_name((*field).into());
                    self.emit_opcode(Opcode::Super);
                    self.emit(Opcode::GetPropertyByName, &[index]);
                }
                PropertyAccessField::Expr(expr) => {
                    self.compile_expr(&**expr, true)?;
                    self.emit_opcode(Opcode::Super);
                    self.emit_opcode(Opcode::GetPropertyByValue);
                }
            },
            Access::This => {
                self.emit(Opcode::This, &[]);
            }
        }

        if !use_expr {
            self.emit(Opcode::Pop, &[]);
        }
        Ok(())
    }

    #[inline]
    fn access_set(
        &mut self,
        access: Access<'_>,
        expr: Option<&Expression>,
        use_expr: bool,
    ) -> JsResult<()> {
        if let Some(expr) = expr {
            self.compile_expr(expr, true)?;
        }

        if use_expr {
            self.emit(Opcode::Dup, &[]);
        }

        match access {
            Access::Variable { name } => {
                let binding = self.context.set_mutable_binding(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::SetName, &[index]);
            }
            Access::Property { access } => match access.field() {
                PropertyAccessField::Const(name) => {
                    self.compile_expr(access.target(), true)?;
                    let index = self.get_or_insert_name((*name).into());
                    self.emit(Opcode::SetPropertyByName, &[index]);
                }
                PropertyAccessField::Expr(expr) => {
                    self.compile_expr(expr, true)?;
                    self.compile_expr(access.target(), true)?;
                    self.emit(Opcode::SetPropertyByValue, &[]);
                }
            },
            Access::PrivateProperty { access } => {
                self.compile_expr(access.target(), true)?;
                self.emit_opcode(Opcode::Swap);
                let index = self.get_or_insert_name(access.field().into());
                self.emit(Opcode::AssignPrivateField, &[index]);
            }
            // TODO: access_set `super`
            Access::SuperProperty { field: _field } => {}
            Access::This => todo!("access_set `this`"),
        }
        Ok(())
    }

    #[inline]
    pub fn compile_statement_list(&mut self, list: &StatementList, use_expr: bool) -> JsResult<()> {
        if let Some((last, items)) = list.statements().split_last() {
            for node in items {
                self.compile_stmt_list_item(node, false)?;
            }
            self.compile_stmt_list_item(last, use_expr)?;
        }
        Ok(())
    }

    /// Compile a statement list in a new declarative environment.
    #[inline]
    pub(crate) fn compile_statement_list_with_new_declarative(
        &mut self,
        list: &StatementList,
        use_expr: bool,
        strict: bool,
    ) -> JsResult<()> {
        self.context.push_compile_time_environment(strict);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        self.create_decls(list);

        if let Some((last, items)) = list.statements().split_last() {
            for node in items {
                self.compile_stmt_list_item(node, false)?;
            }
            self.compile_stmt_list_item(last, use_expr)?;
        }

        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        self.emit_opcode(Opcode::PopEnvironment);

        Ok(())
    }

    #[inline]
    pub fn compile_expr(&mut self, expr: &Expression, use_expr: bool) -> JsResult<()> {
        match expr {
            Expression::Literal(lit) => {
                match lit {
                    literal::Literal::String(v) => self.emit_push_literal(Literal::String(
                        self.interner().resolve_expect(*v).into_common(false),
                    )),
                    literal::Literal::Int(v) => self.emit_push_integer(*v),
                    literal::Literal::Num(v) => self.emit_push_rational(*v),
                    literal::Literal::BigInt(v) => {
                        self.emit_push_literal(Literal::BigInt(v.clone().into()));
                    }
                    literal::Literal::Bool(true) => self.emit(Opcode::PushTrue, &[]),
                    literal::Literal::Bool(false) => self.emit(Opcode::PushFalse, &[]),
                    literal::Literal::Null => self.emit(Opcode::PushNull, &[]),
                    literal::Literal::Undefined => self.emit(Opcode::PushUndefined, &[]),
                }

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Expression::Unary(unary) => {
                let opcode = match unary.op() {
                    UnaryOp::IncrementPre => {
                        self.compile_expr(unary.target(), true)?;
                        self.emit(Opcode::Inc, &[]);

                        let access = Self::compile_access(unary.target()).ok_or_else(|| {
                            self.context
                                .construct_syntax_error("Invalid increment operand")
                        })?;
                        self.access_set(access, None, true)?;
                        None
                    }
                    UnaryOp::DecrementPre => {
                        self.compile_expr(unary.target(), true)?;
                        self.emit(Opcode::Dec, &[]);

                        let access = Self::compile_access(unary.target()).ok_or_else(|| {
                            self.context
                                .construct_syntax_error("Invalid decrement operand")
                        })?;
                        self.access_set(access, None, true)?;
                        None
                    }
                    UnaryOp::IncrementPost => {
                        self.compile_expr(unary.target(), true)?;
                        self.emit(Opcode::IncPost, &[]);

                        let access = Self::compile_access(unary.target()).ok_or_else(|| {
                            self.context
                                .construct_syntax_error("Invalid increment operand")
                        })?;
                        self.access_set(access, None, false)?;

                        None
                    }
                    UnaryOp::DecrementPost => {
                        self.compile_expr(unary.target(), true)?;
                        self.emit(Opcode::DecPost, &[]);

                        let access = Self::compile_access(unary.target()).ok_or_else(|| {
                            self.context
                                .construct_syntax_error("Invalid decrement operand")
                        })?;
                        self.access_set(access, None, false)?;

                        None
                    }
                    UnaryOp::Delete => match unary.target() {
                        Expression::PropertyAccess(ref access) => {
                            match access.field() {
                                PropertyAccessField::Const(name) => {
                                    let index = self.get_or_insert_name((*name).into());
                                    self.compile_expr(access.target(), true)?;
                                    self.emit(Opcode::DeletePropertyByName, &[index]);
                                }
                                PropertyAccessField::Expr(expr) => {
                                    self.compile_expr(expr, true)?;
                                    self.compile_expr(access.target(), true)?;
                                    self.emit(Opcode::DeletePropertyByValue, &[]);
                                }
                            }
                            None
                        }
                        // TODO: implement delete on references.
                        Expression::Identifier(_) => {
                            self.emit(Opcode::PushFalse, &[]);
                            None
                        }
                        _ => {
                            self.emit(Opcode::PushTrue, &[]);
                            None
                        }
                    },
                    UnaryOp::Minus => Some(Opcode::Neg),
                    UnaryOp::Plus => Some(Opcode::Pos),
                    UnaryOp::Not => Some(Opcode::LogicalNot),
                    UnaryOp::Tilde => Some(Opcode::BitNot),
                    UnaryOp::TypeOf => {
                        match &unary.target() {
                            Expression::Identifier(identifier) => {
                                let binding = self.context.get_binding_value(*identifier);
                                let index = self.get_or_insert_binding(binding);
                                self.emit(Opcode::GetNameOrUndefined, &[index]);
                            }
                            expr => self.compile_expr(expr, true)?,
                        }
                        self.emit_opcode(Opcode::TypeOf);
                        None
                    }
                    UnaryOp::Void => Some(Opcode::Void),
                };

                if let Some(opcode) = opcode {
                    self.compile_expr(unary.target(), true)?;
                    self.emit(opcode, &[]);
                }

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Expression::Binary(binary) => {
                self.compile_expr(binary.lhs(), true)?;
                match binary.op() {
                    BinaryOp::Arithmetic(op) => {
                        self.compile_expr(binary.rhs(), true)?;
                        match op {
                            ArithmeticOp::Add => self.emit_opcode(Opcode::Add),
                            ArithmeticOp::Sub => self.emit_opcode(Opcode::Sub),
                            ArithmeticOp::Div => self.emit_opcode(Opcode::Div),
                            ArithmeticOp::Mul => self.emit_opcode(Opcode::Mul),
                            ArithmeticOp::Exp => self.emit_opcode(Opcode::Pow),
                            ArithmeticOp::Mod => self.emit_opcode(Opcode::Mod),
                        }

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                    BinaryOp::Bitwise(op) => {
                        self.compile_expr(binary.rhs(), true)?;
                        match op {
                            BitwiseOp::And => self.emit_opcode(Opcode::BitAnd),
                            BitwiseOp::Or => self.emit_opcode(Opcode::BitOr),
                            BitwiseOp::Xor => self.emit_opcode(Opcode::BitXor),
                            BitwiseOp::Shl => self.emit_opcode(Opcode::ShiftLeft),
                            BitwiseOp::Shr => self.emit_opcode(Opcode::ShiftRight),
                            BitwiseOp::UShr => self.emit_opcode(Opcode::UnsignedShiftRight),
                        }

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                    BinaryOp::Relational(op) => {
                        self.compile_expr(binary.rhs(), true)?;
                        match op {
                            RelationalOp::Equal => self.emit_opcode(Opcode::Eq),
                            RelationalOp::NotEqual => self.emit_opcode(Opcode::NotEq),
                            RelationalOp::StrictEqual => self.emit_opcode(Opcode::StrictEq),
                            RelationalOp::StrictNotEqual => self.emit_opcode(Opcode::StrictNotEq),
                            RelationalOp::GreaterThan => self.emit_opcode(Opcode::GreaterThan),
                            RelationalOp::GreaterThanOrEqual => {
                                self.emit_opcode(Opcode::GreaterThanOrEq);
                            }
                            RelationalOp::LessThan => self.emit_opcode(Opcode::LessThan),
                            RelationalOp::LessThanOrEqual => self.emit_opcode(Opcode::LessThanOrEq),
                            RelationalOp::In => self.emit_opcode(Opcode::In),
                            RelationalOp::InstanceOf => self.emit_opcode(Opcode::InstanceOf),
                        }

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                    BinaryOp::Logical(op) => {
                        match op {
                            LogicalOp::And => {
                                let exit = self.emit_opcode_with_operand(Opcode::LogicalAnd);
                                self.compile_expr(binary.rhs(), true)?;
                                self.patch_jump(exit);
                            }
                            LogicalOp::Or => {
                                let exit = self.emit_opcode_with_operand(Opcode::LogicalOr);
                                self.compile_expr(binary.rhs(), true)?;
                                self.patch_jump(exit);
                            }
                            LogicalOp::Coalesce => {
                                let exit = self.emit_opcode_with_operand(Opcode::Coalesce);
                                self.compile_expr(binary.rhs(), true)?;
                                self.patch_jump(exit);
                            }
                        };

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                    BinaryOp::Comma => {
                        self.emit(Opcode::Pop, &[]);
                        self.compile_expr(binary.rhs(), true)?;

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                }
            }
            Expression::Assign(assign) if assign.op() == AssignOp::Assign => match assign.lhs() {
                AssignTarget::Identifier(name) => self.access_set(
                    Access::Variable { name: *name },
                    Some(assign.rhs()),
                    use_expr,
                )?,
                AssignTarget::PrivateProperty(access) => self.access_set(
                    Access::PrivateProperty { access },
                    Some(assign.rhs()),
                    use_expr,
                )?,
                AssignTarget::Property(access) => {
                    self.access_set(Access::Property { access }, Some(assign.rhs()), use_expr)?;
                }
                AssignTarget::SuperProperty(access) => {
                    self.access_set(
                        Access::SuperProperty {
                            field: access.field(),
                        },
                        Some(assign.rhs()),
                        use_expr,
                    )?;
                }
                AssignTarget::Pattern(pattern) => {
                    self.compile_expr(assign.rhs(), true)?;
                    if use_expr {
                        self.emit_opcode(Opcode::Dup);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::SetName)?;
                }
            },
            Expression::Assign(assign) => {
                let access = match assign.lhs() {
                    AssignTarget::Identifier(name) => Access::Variable { name: *name },
                    AssignTarget::Property(access) => Access::Property { access },
                    AssignTarget::PrivateProperty(access) => Access::PrivateProperty { access },
                    AssignTarget::SuperProperty(access) => Access::SuperProperty {
                        field: access.field(),
                    },
                    AssignTarget::Pattern(_) => {
                        panic!("tried to use an assignment operator on a pattern")
                    }
                };
                self.access_get(access, true)?;
                let opcode = match assign.op() {
                    AssignOp::Assign => unreachable!(),
                    AssignOp::Add => Opcode::Add,
                    AssignOp::Sub => Opcode::Sub,
                    AssignOp::Mul => Opcode::Mul,
                    AssignOp::Div => Opcode::Div,
                    AssignOp::Mod => Opcode::Mod,
                    AssignOp::Exp => Opcode::Pow,
                    AssignOp::And => Opcode::BitAnd,
                    AssignOp::Or => Opcode::BitOr,
                    AssignOp::Xor => Opcode::BitXor,
                    AssignOp::Shl => Opcode::ShiftLeft,
                    AssignOp::Shr => Opcode::ShiftRight,
                    AssignOp::Ushr => Opcode::UnsignedShiftRight,
                    AssignOp::BoolAnd => {
                        let exit = self.emit_opcode_with_operand(Opcode::LogicalAnd);
                        self.compile_expr(assign.rhs(), true)?;
                        self.access_set(access, None, use_expr)?;
                        self.patch_jump(exit);
                        return Ok(());
                    }
                    AssignOp::BoolOr => {
                        let exit = self.emit_opcode_with_operand(Opcode::LogicalOr);
                        self.compile_expr(assign.rhs(), true)?;
                        self.access_set(access, None, use_expr)?;
                        self.patch_jump(exit);
                        return Ok(());
                    }
                    AssignOp::Coalesce => {
                        let exit = self.emit_opcode_with_operand(Opcode::Coalesce);
                        self.compile_expr(assign.rhs(), true)?;
                        self.access_set(access, None, use_expr)?;
                        self.patch_jump(exit);
                        return Ok(());
                    }
                };

                self.compile_expr(assign.rhs(), true)?;
                self.emit(opcode, &[]);
                self.access_set(access, None, use_expr)?;
            }
            Expression::ObjectLiteral(object) => {
                self.emit_opcode(Opcode::PushEmptyObject);
                for property in object.properties() {
                    self.emit_opcode(Opcode::Dup);
                    match property {
                        PropertyDefinition::IdentifierReference(ident) => {
                            let index = self.get_or_insert_name(*ident);
                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                        }
                        PropertyDefinition::Property(name, expr) => match name {
                            PropertyName::Literal(name) => {
                                self.compile_expr(expr, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.compile_expr(expr, true)?;
                                self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                            }
                        },
                        PropertyDefinition::MethodDefinition(name, kind) => match kind {
                            MethodDefinition::Get(expr) => match name {
                                PropertyName::Literal(name) => {
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::Swap);
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit(Opcode::SetPropertyGetterByName, &[index]);
                                }
                                PropertyName::Computed(name_node) => {
                                    self.compile_expr(name_node, true)?;
                                    self.emit_opcode(Opcode::ToPropertyKey);
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::SetPropertyGetterByValue);
                                }
                            },
                            MethodDefinition::Set(expr) => match name {
                                PropertyName::Literal(name) => {
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::Swap);
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit(Opcode::SetPropertySetterByName, &[index]);
                                }
                                PropertyName::Computed(name_node) => {
                                    self.compile_expr(name_node, true)?;
                                    self.emit_opcode(Opcode::ToPropertyKey);
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::SetPropertySetterByValue);
                                }
                            },
                            MethodDefinition::Ordinary(expr) => match name {
                                PropertyName::Literal(name) => {
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::Swap);
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                                }
                                PropertyName::Computed(name_node) => {
                                    self.compile_expr(name_node, true)?;
                                    self.emit_opcode(Opcode::ToPropertyKey);
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                }
                            },
                            MethodDefinition::Async(expr) => match name {
                                PropertyName::Literal(name) => {
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::Swap);
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                                }
                                PropertyName::Computed(name_node) => {
                                    self.compile_expr(name_node, true)?;
                                    self.emit_opcode(Opcode::ToPropertyKey);
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                }
                            },
                            MethodDefinition::Generator(expr) => match name {
                                PropertyName::Literal(name) => {
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::Swap);
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                                }
                                PropertyName::Computed(name_node) => {
                                    self.compile_expr(name_node, true)?;
                                    self.emit_opcode(Opcode::ToPropertyKey);
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                }
                            },
                            MethodDefinition::AsyncGenerator(expr) => match name {
                                PropertyName::Literal(name) => {
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::Swap);
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                                }
                                PropertyName::Computed(name_node) => {
                                    self.compile_expr(name_node, true)?;
                                    self.emit_opcode(Opcode::ToPropertyKey);
                                    self.function(expr.into(), NodeKind::Expression, true)?;
                                    self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                }
                            },
                        },
                        PropertyDefinition::SpreadObject(expr) => {
                            self.compile_expr(expr, true)?;
                            self.emit_opcode(Opcode::Swap);
                            self.emit(Opcode::CopyDataProperties, &[0, 0]);
                            self.emit_opcode(Opcode::Pop);
                        }
                        PropertyDefinition::CoverInitializedName(_, _) => {
                            return self.context.throw_syntax_error(
                                "invalid assignment pattern in object literal",
                            );
                        }
                    }
                }

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Expression::Identifier(name) => {
                self.access_get(Access::Variable { name: *name }, use_expr)?;
            }
            Expression::PropertyAccess(access) => {
                self.access_get(Access::Property { access }, use_expr)?;
            }
            Expression::PrivatePropertyAccess(access) => {
                self.access_get(Access::PrivateProperty { access }, use_expr)?;
            }
            Expression::SuperPropertyAccess(access) => self.access_get(
                Access::SuperProperty {
                    field: access.field(),
                },
                use_expr,
            )?,
            Expression::Conditional(op) => {
                self.compile_expr(op.cond(), true)?;
                let jelse = self.jump_if_false();
                self.compile_expr(op.if_true(), true)?;
                let exit = self.jump();
                self.patch_jump(jelse);
                self.compile_expr(op.if_false(), true)?;
                self.patch_jump(exit);

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Expression::ArrayLiteral(array) => {
                self.emit_opcode(Opcode::PushNewArray);
                self.emit_opcode(Opcode::PopOnReturnAdd);

                for element in array.as_ref() {
                    if let Some(element) = element {
                        self.compile_expr(element, true)?;
                        if let Expression::Spread(_) = element {
                            self.emit_opcode(Opcode::InitIterator);
                            self.emit_opcode(Opcode::PushIteratorToArray);
                        } else {
                            self.emit_opcode(Opcode::PushValueToArray);
                        }
                    } else {
                        self.emit_opcode(Opcode::PushElisionToArray);
                    }
                }

                self.emit_opcode(Opcode::PopOnReturnSub);
                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Expression::This => {
                self.access_get(Access::This, use_expr)?;
            }
            Expression::Spread(spread) => self.compile_expr(spread.val(), true)?,
            Expression::Function(function) => {
                self.function(function.into(), NodeKind::Expression, use_expr)?;
            }
            Expression::ArrowFunction(function) => {
                self.function(function.into(), NodeKind::Expression, use_expr)?;
            }
            Expression::Generator(function) => {
                self.function(function.into(), NodeKind::Expression, use_expr)?;
            }
            Expression::AsyncFunction(function) => {
                self.function(function.into(), NodeKind::Expression, use_expr)?;
            }
            Expression::AsyncGenerator(function) => {
                self.function(function.into(), NodeKind::Expression, use_expr)?;
            }
            Expression::Call(call) => self.call(Callable::Call(call), use_expr)?,
            Expression::New(new) => self.call(Callable::New(new), use_expr)?,
            Expression::TemplateLiteral(template_literal) => {
                for element in template_literal.elements() {
                    match element {
                        TemplateElement::String(s) => self.emit_push_literal(Literal::String(
                            self.interner().resolve_expect(*s).into_common(false),
                        )),
                        TemplateElement::Expr(expr) => {
                            self.compile_expr(expr, true)?;
                        }
                    }
                }

                self.emit(
                    Opcode::ConcatToString,
                    &[template_literal.elements().len() as u32],
                );

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Expression::Await(expr) => {
                self.compile_expr(expr.expr(), true)?;
                self.emit_opcode(Opcode::Await);
                self.emit_opcode(Opcode::GeneratorNext);
                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Expression::Yield(r#yield) => {
                if let Some(expr) = r#yield.expr() {
                    self.compile_expr(expr, true)?;
                } else {
                    self.emit_opcode(Opcode::PushUndefined);
                }

                if r#yield.delegate() {
                    if self.in_async_generator {
                        self.emit_opcode(Opcode::InitIteratorAsync);
                    } else {
                        self.emit_opcode(Opcode::InitIterator);
                    }
                    self.emit_opcode(Opcode::PushUndefined);
                    let start_address = self.next_opcode_location();
                    let start = self.emit_opcode_with_operand(Opcode::GeneratorNextDelegate);
                    self.emit(Opcode::Jump, &[start_address]);
                    self.patch_jump(start);
                } else if self.in_async_generator {
                    self.emit_opcode(Opcode::Await);
                    self.emit_opcode(Opcode::AsyncGeneratorNext);
                    let jump_return = self.emit_opcode_with_operand(Opcode::JumpIfFalse);
                    let jump = self.emit_opcode_with_operand(Opcode::JumpIfFalse);
                    self.emit_opcode(Opcode::Yield);
                    self.emit_opcode(Opcode::GeneratorNext);
                    self.patch_jump(jump);
                    self.emit_opcode(Opcode::Await);
                    self.emit_opcode(Opcode::GeneratorNext);
                    self.patch_jump(jump_return);
                } else {
                    self.emit_opcode(Opcode::Yield);
                    self.emit_opcode(Opcode::GeneratorNext);
                }

                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Expression::TaggedTemplate(template) => {
                match template.tag() {
                    Expression::PropertyAccess(access) => {
                        self.compile_expr(access.target(), true)?;
                        self.emit(Opcode::Dup, &[]);
                        match access.field() {
                            PropertyAccessField::Const(field) => {
                                let index = self.get_or_insert_name((*field).into());
                                self.emit(Opcode::GetPropertyByName, &[index]);
                            }
                            PropertyAccessField::Expr(field) => {
                                self.compile_expr(field, true)?;
                                self.emit(Opcode::Swap, &[]);
                                self.emit(Opcode::GetPropertyByValue, &[]);
                            }
                        }
                    }
                    expr => {
                        self.compile_expr(expr, true)?;
                        self.emit_opcode(Opcode::This);
                        self.emit_opcode(Opcode::Swap);
                    }
                }

                self.emit_opcode(Opcode::PushNewArray);
                for cooked in template.cookeds() {
                    if let Some(cooked) = cooked {
                        self.emit_push_literal(Literal::String(
                            self.interner().resolve_expect(*cooked).into_common(false),
                        ));
                    } else {
                        self.emit_opcode(Opcode::PushUndefined);
                    }
                    self.emit_opcode(Opcode::PushValueToArray);
                }
                self.emit_opcode(Opcode::Dup);

                self.emit_opcode(Opcode::PushNewArray);
                for raw in template.raws() {
                    self.emit_push_literal(Literal::String(
                        self.interner().resolve_expect(*raw).into_common(false),
                    ));
                    self.emit_opcode(Opcode::PushValueToArray);
                }

                self.emit_opcode(Opcode::Swap);
                let index = self.get_or_insert_name(Sym::RAW.into());
                self.emit(Opcode::SetPropertyByName, &[index]);

                for expr in template.exprs() {
                    self.compile_expr(expr, true)?;
                }

                self.emit(Opcode::Call, &[(template.exprs().len() + 1) as u32]);
            }
            Expression::Class(class) => self.class(class, true)?,
            Expression::SuperCall(super_call) => {
                let contains_spread = super_call
                    .args()
                    .iter()
                    .any(|arg| matches!(arg, Expression::Spread(_)));

                if contains_spread {
                    self.emit_opcode(Opcode::PushNewArray);
                    for arg in super_call.args() {
                        self.compile_expr(arg, true)?;
                        if let Expression::Spread(_) = arg {
                            self.emit_opcode(Opcode::InitIterator);
                            self.emit_opcode(Opcode::PushIteratorToArray);
                        } else {
                            self.emit_opcode(Opcode::PushValueToArray);
                        }
                    }
                } else {
                    for arg in super_call.args() {
                        self.compile_expr(arg, true)?;
                    }
                }

                if contains_spread {
                    self.emit_opcode(Opcode::SuperCallSpread);
                } else {
                    self.emit(Opcode::SuperCall, &[super_call.args().len() as u32]);
                }

                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Expression::NewTarget => {
                if use_expr {
                    self.emit_opcode(Opcode::PushNewTarget);
                }
            }
            // TODO: try to remove this variant somehow
            Expression::FormalParameterList(_) => unreachable!(),
        }
        Ok(())
    }

    pub fn compile_var_decl(&mut self, decl: &VarDeclaration) -> JsResult<()> {
        for variable in decl.0.as_ref() {
            match variable.binding() {
                Binding::Identifier(ident) => {
                    let ident = ident;
                    if let Some(expr) = variable.init() {
                        self.compile_expr(expr, true)?;
                        self.emit_binding(BindingOpcode::InitVar, *ident);
                    } else {
                        self.emit_binding(BindingOpcode::Var, *ident);
                    }
                }
                Binding::Pattern(pattern) => {
                    if let Some(init) = variable.init() {
                        self.compile_expr(init, true)?;
                    } else {
                        self.emit_opcode(Opcode::PushUndefined);
                    };

                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
                }
            }
        }
        Ok(())
    }

    pub fn compile_lexical_decl(&mut self, decl: &LexicalDeclaration) -> JsResult<()> {
        match decl {
            LexicalDeclaration::Let(decls) => {
                for variable in decls.as_ref() {
                    match variable.binding() {
                        Binding::Identifier(ident) => {
                            let ident = ident;
                            if let Some(expr) = variable.init() {
                                self.compile_expr(expr, true)?;
                                self.emit_binding(BindingOpcode::InitLet, *ident);
                            } else {
                                self.emit_binding(BindingOpcode::Let, *ident);
                            }
                        }
                        Binding::Pattern(pattern) => {
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, true)?;
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
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
                            self.compile_expr(init, true)?;
                            self.emit_binding(BindingOpcode::InitConst, *ident);
                        }
                        Binding::Pattern(pattern) => {
                            if let Some(init) = variable.init() {
                                self.compile_expr(init, true)?;
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, BindingOpcode::InitConst)?;
                        }
                    }
                }
                return Ok(());
            }
        };
        Ok(())
    }

    #[inline]
    pub fn compile_stmt_list_item(
        &mut self,
        item: &StatementListItem,
        use_expr: bool,
    ) -> JsResult<()> {
        match item {
            StatementListItem::Statement(stmt) => self.compile_stmt(stmt, use_expr),
            StatementListItem::Declaration(decl) => self.compile_decl(decl),
        }
    }

    #[inline]
    pub fn compile_decl(&mut self, decl: &Declaration) -> JsResult<()> {
        match decl {
            Declaration::Function(function) => {
                self.function(function.into(), NodeKind::Declaration, false)
            }
            Declaration::Generator(function) => {
                self.function(function.into(), NodeKind::Declaration, false)
            }
            Declaration::AsyncFunction(function) => {
                self.function(function.into(), NodeKind::Declaration, false)
            }
            Declaration::AsyncGenerator(function) => {
                self.function(function.into(), NodeKind::Declaration, false)
            }
            Declaration::Class(class) => self.class(class, false),
            Declaration::Lexical(lexical) => self.compile_lexical_decl(lexical),
        }
    }

    #[inline]
    pub fn compile_for_loop(&mut self, for_loop: &ForLoop, label: Option<Sym>) -> JsResult<()> {
        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        if let Some(init) = for_loop.init() {
            match init {
                ForLoopInitializer::Expression(expr) => self.compile_expr(expr, false)?,
                ForLoopInitializer::Var(decl) => {
                    self.create_decls_from_var_decl(decl);
                    self.compile_var_decl(decl)?;
                }
                ForLoopInitializer::Lexical(decl) => {
                    self.create_decls_from_lexical_decl(decl);
                    self.compile_lexical_decl(decl)?;
                }
            }
        }

        self.emit_opcode(Opcode::LoopStart);
        let initial_jump = self.jump();

        let start_address = self.next_opcode_location();
        self.push_loop_control_info(label, start_address);

        self.emit_opcode(Opcode::LoopContinue);
        if let Some(final_expr) = for_loop.final_expr() {
            self.compile_expr(final_expr, false)?;
        }

        self.patch_jump(initial_jump);

        if let Some(condition) = for_loop.condition() {
            self.compile_expr(condition, true)?;
        } else {
            self.emit_opcode(Opcode::PushTrue);
        }
        let exit = self.jump_if_false();

        self.compile_stmt(for_loop.body(), false)?;

        self.emit(Opcode::Jump, &[start_address]);

        self.patch_jump(exit);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);

        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        self.emit_opcode(Opcode::PopEnvironment);
        Ok(())
    }

    #[inline]
    pub fn compile_for_in_loop(
        &mut self,
        for_in_loop: &ForInLoop,
        label: Option<Sym>,
    ) -> JsResult<()> {
        let init_bound_names = for_in_loop.init().bound_names();
        if init_bound_names.is_empty() {
            self.compile_expr(for_in_loop.expr(), true)?;
        } else {
            self.context.push_compile_time_environment(false);
            let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

            for name in init_bound_names {
                self.context.create_mutable_binding(name, false);
            }
            self.compile_expr(for_in_loop.expr(), true)?;

            let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
            let index_compile_environment = self.push_compile_environment(compile_environment);
            self.patch_jump_with_target(push_env.0, num_bindings as u32);
            self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        let early_exit = self.emit_opcode_with_operand(Opcode::ForInLoopInitIterator);

        self.emit_opcode(Opcode::LoopStart);
        let start_address = self.next_opcode_location();
        self.push_loop_control_info_for_of_in_loop(label, start_address);
        self.emit_opcode(Opcode::LoopContinue);

        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
        let exit = self.emit_opcode_with_operand(Opcode::ForInLoopNext);

        match for_in_loop.init() {
            IterableLoopInitializer::Identifier(ident) => {
                self.context.create_mutable_binding(*ident, true);
                let binding = self.context.set_mutable_binding(*ident);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefInitVar, &[index]);
            }
            IterableLoopInitializer::Var(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_mutable_binding(*ident, true);
                    self.emit_binding(BindingOpcode::InitVar, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in pattern.idents() {
                        self.context.create_mutable_binding(ident, true);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
                }
            },
            IterableLoopInitializer::Let(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_mutable_binding(*ident, false);
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in pattern.idents() {
                        self.context.create_mutable_binding(ident, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                }
            },
            IterableLoopInitializer::Const(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_immutable_binding(*ident);
                    self.emit_binding(BindingOpcode::InitConst, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in pattern.idents() {
                        self.context.create_immutable_binding(ident);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitConst)?;
                }
            },
            IterableLoopInitializer::Pattern(pattern) => {
                for ident in pattern.idents() {
                    self.context.create_mutable_binding(ident, true);
                }
                self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
            }
        }

        self.compile_stmt(for_in_loop.body(), false)?;

        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        self.emit_opcode(Opcode::PopEnvironment);

        self.emit(Opcode::Jump, &[start_address]);

        self.patch_jump(exit);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
        self.emit_opcode(Opcode::IteratorClose);

        self.patch_jump(early_exit);
        Ok(())
    }

    #[inline]
    pub fn compile_for_of_loop(
        &mut self,
        for_of_loop: &ForOfLoop,
        label: Option<Sym>,
    ) -> JsResult<()> {
        let init_bound_names = for_of_loop.init().bound_names();
        if init_bound_names.is_empty() {
            self.compile_expr(for_of_loop.iterable(), true)?;
        } else {
            self.context.push_compile_time_environment(false);
            let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

            for name in init_bound_names {
                self.context.create_mutable_binding(name, false);
            }
            self.compile_expr(for_of_loop.iterable(), true)?;

            let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
            let index_compile_environment = self.push_compile_environment(compile_environment);
            self.patch_jump_with_target(push_env.0, num_bindings as u32);
            self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        if for_of_loop.r#await() {
            self.emit_opcode(Opcode::InitIteratorAsync);
        } else {
            self.emit_opcode(Opcode::InitIterator);
        }

        self.emit_opcode(Opcode::LoopStart);
        let start_address = self.next_opcode_location();
        self.push_loop_control_info_for_of_in_loop(label, start_address);
        self.emit_opcode(Opcode::LoopContinue);

        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        let exit = if for_of_loop.r#await() {
            self.emit_opcode(Opcode::ForAwaitOfLoopIterate);
            self.emit_opcode(Opcode::Await);
            self.emit_opcode(Opcode::GeneratorNext);
            self.emit_opcode_with_operand(Opcode::ForAwaitOfLoopNext)
        } else {
            self.emit_opcode_with_operand(Opcode::ForInLoopNext)
        };

        match for_of_loop.init() {
            IterableLoopInitializer::Identifier(ref ident) => {
                self.context.create_mutable_binding(*ident, true);
                let binding = self.context.set_mutable_binding(*ident);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefInitVar, &[index]);
            }
            IterableLoopInitializer::Var(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_mutable_binding(*ident, true);
                    self.emit_binding(BindingOpcode::InitVar, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in pattern.idents() {
                        self.context.create_mutable_binding(ident, true);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
                }
            },
            IterableLoopInitializer::Let(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_mutable_binding(*ident, false);
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in pattern.idents() {
                        self.context.create_mutable_binding(ident, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                }
            },
            IterableLoopInitializer::Const(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_immutable_binding(*ident);
                    self.emit_binding(BindingOpcode::InitConst, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in pattern.idents() {
                        self.context.create_immutable_binding(ident);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitConst)?;
                }
            },
            IterableLoopInitializer::Pattern(pattern) => {
                for ident in pattern.idents() {
                    self.context.create_mutable_binding(ident, true);
                }
                self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
            }
        }

        self.compile_stmt(for_of_loop.body(), false)?;

        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        self.emit_opcode(Opcode::PopEnvironment);

        self.emit(Opcode::Jump, &[start_address]);

        self.patch_jump(exit);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
        self.emit_opcode(Opcode::IteratorClose);
        Ok(())
    }

    #[inline]
    pub fn compile_while_loop(
        &mut self,
        while_loop: &WhileLoop,
        label: Option<Sym>,
    ) -> JsResult<()> {
        self.emit_opcode(Opcode::LoopStart);
        let start_address = self.next_opcode_location();
        self.push_loop_control_info(label, start_address);
        self.emit_opcode(Opcode::LoopContinue);

        self.compile_expr(while_loop.condition(), true)?;
        let exit = self.jump_if_false();
        self.compile_stmt(while_loop.body(), false)?;
        self.emit(Opcode::Jump, &[start_address]);
        self.patch_jump(exit);

        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
        Ok(())
    }

    #[inline]
    pub fn compile_do_while_loop(
        &mut self,
        do_while_loop: &DoWhileLoop,
        label: Option<Sym>,
    ) -> JsResult<()> {
        self.emit_opcode(Opcode::LoopStart);
        let initial_label = self.jump();

        let start_address = self.next_opcode_location();
        self.push_loop_control_info(label, start_address);
        self.emit_opcode(Opcode::LoopContinue);

        let condition_label_address = self.next_opcode_location();
        self.compile_expr(do_while_loop.cond(), true)?;
        let exit = self.jump_if_false();

        self.patch_jump(initial_label);

        self.compile_stmt(do_while_loop.body(), false)?;
        self.emit(Opcode::Jump, &[condition_label_address]);
        self.patch_jump(exit);

        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
        Ok(())
    }

    #[inline]
    pub fn compile_block(
        &mut self,
        block: &Block,
        label: Option<Sym>,
        use_expr: bool,
    ) -> JsResult<()> {
        if let Some(label) = label {
            let next = self.next_opcode_location();
            self.push_labelled_block_control_info(label, next);
        }

        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
        self.create_decls(block.statement_list());
        self.compile_statement_list(block.statement_list(), use_expr)?;
        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);

        if label.is_some() {
            self.pop_labelled_block_control_info();
        }

        self.emit_opcode(Opcode::PopEnvironment);
        Ok(())
    }

    #[inline]
    pub fn compile_stmt(&mut self, node: &Statement, use_expr: bool) -> JsResult<()> {
        match node {
            Statement::Var(var) => self.compile_var_decl(var)?,
            Statement::If(node) => {
                self.compile_expr(node.cond(), true)?;
                let jelse = self.jump_if_false();

                self.compile_stmt(node.body(), false)?;

                match node.else_node() {
                    None => {
                        self.patch_jump(jelse);
                    }
                    Some(else_body) => {
                        let exit = self.jump();
                        self.patch_jump(jelse);
                        self.compile_stmt(else_body, false)?;
                        self.patch_jump(exit);
                    }
                }
            }
            Statement::ForLoop(for_loop) => self.compile_for_loop(for_loop, None)?,
            Statement::ForInLoop(for_in_loop) => self.compile_for_in_loop(for_in_loop, None)?,
            Statement::ForOfLoop(for_of_loop) => self.compile_for_of_loop(for_of_loop, None)?,
            Statement::WhileLoop(while_loop) => self.compile_while_loop(while_loop, None)?,
            Statement::DoWhileLoop(do_while_loop) => {
                self.compile_do_while_loop(do_while_loop, None)?;
            }
            Statement::Block(block) => self.compile_block(block, None, use_expr)?,
            Statement::Labelled(labelled) => match labelled.item() {
                LabelledItem::Statement(stmt) => match stmt {
                    Statement::ForLoop(for_loop) => {
                        self.compile_for_loop(for_loop, Some(labelled.label()))?;
                    }
                    Statement::ForInLoop(for_in_loop) => {
                        self.compile_for_in_loop(for_in_loop, Some(labelled.label()))?;
                    }
                    Statement::ForOfLoop(for_of_loop) => {
                        self.compile_for_of_loop(for_of_loop, Some(labelled.label()))?;
                    }
                    Statement::WhileLoop(while_loop) => {
                        self.compile_while_loop(while_loop, Some(labelled.label()))?;
                    }
                    Statement::DoWhileLoop(do_while_loop) => {
                        self.compile_do_while_loop(do_while_loop, Some(labelled.label()))?;
                    }
                    Statement::Block(block) => {
                        self.compile_block(block, Some(labelled.label()), use_expr)?;
                    }
                    stmt => self.compile_stmt(stmt, use_expr)?,
                },
                LabelledItem::Function(f) => {
                    self.function(f.into(), NodeKind::Declaration, false)?;
                }
            },
            Statement::Continue(node) => {
                let next = self.next_opcode_location();
                if let Some(info) = self
                    .jump_info
                    .last()
                    .filter(|info| info.kind == JumpControlInfoKind::Try)
                {
                    let start_address = info.start_address;
                    let in_finally = if let Some(finally_start) = info.finally_start {
                        next > finally_start.index
                    } else {
                        false
                    };
                    let in_catch_no_finally = !info.has_finally && info.in_catch;

                    if in_finally {
                        self.emit_opcode(Opcode::PopIfThrown);
                    }
                    if in_finally || in_catch_no_finally {
                        self.emit_opcode(Opcode::CatchEnd2);
                        self.emit(Opcode::FinallySetJump, &[start_address]);
                    } else {
                        self.emit_opcode(Opcode::TryEnd);
                        self.emit(Opcode::FinallySetJump, &[start_address]);
                    }
                    let label = self.jump();
                    self.jump_info
                        .last_mut()
                        .expect("no jump information found")
                        .try_continues
                        .push(label);
                } else {
                    let mut items = self
                        .jump_info
                        .iter()
                        .rev()
                        .filter(|info| info.kind == JumpControlInfoKind::Loop);
                    let address = if let Some(label_name) = node.label() {
                        let mut num_loops = 0;
                        let mut emit_for_of_in_exit = 0;
                        let mut address_info = None;
                        for info in items {
                            if info.label == node.label() {
                                address_info = Some(info);
                                break;
                            }
                            num_loops += 1;
                            if info.for_of_in_loop {
                                emit_for_of_in_exit += 1;
                            }
                        }
                        let address = address_info
                            .ok_or_else(|| {
                                self.context.construct_syntax_error(format!(
                                    "Cannot use the undeclared label '{}'",
                                    self.context.interner().resolve_expect(label_name)
                                ))
                            })?
                            .start_address;
                        for _ in 0..emit_for_of_in_exit {
                            self.emit_opcode(Opcode::Pop);
                            self.emit_opcode(Opcode::Pop);
                            self.emit_opcode(Opcode::Pop);
                        }
                        for _ in 0..num_loops {
                            self.emit_opcode(Opcode::LoopEnd);
                        }
                        address
                    } else {
                        items
                            .next()
                            .ok_or_else(|| {
                                self.context
                                    .construct_syntax_error("continue must be inside loop")
                            })?
                            .start_address
                    };
                    self.emit_opcode(Opcode::LoopEnd);
                    self.emit_opcode(Opcode::LoopStart);
                    self.emit(Opcode::Jump, &[address]);
                }
            }
            Statement::Break(node) => {
                let next = self.next_opcode_location();
                if let Some(info) = self
                    .jump_info
                    .last()
                    .filter(|info| info.kind == JumpControlInfoKind::Try)
                {
                    let in_finally = if let Some(finally_start) = info.finally_start {
                        next >= finally_start.index
                    } else {
                        false
                    };
                    let in_catch_no_finally = !info.has_finally && info.in_catch;

                    if in_finally {
                        self.emit_opcode(Opcode::PopIfThrown);
                    }
                    if in_finally || in_catch_no_finally {
                        self.emit_opcode(Opcode::CatchEnd2);
                    } else {
                        self.emit_opcode(Opcode::TryEnd);
                    }
                    self.emit(Opcode::FinallySetJump, &[u32::MAX]);
                }
                let label = self.jump();
                if let Some(label_name) = node.label() {
                    let mut found = false;
                    for info in self.jump_info.iter_mut().rev() {
                        if info.label == Some(label_name) {
                            info.breaks.push(label);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return self.context.throw_syntax_error(format!(
                            "Cannot use the undeclared label '{}'",
                            self.interner().resolve_expect(label_name)
                        ));
                    }
                } else {
                    self.jump_info
                        .last_mut()
                        .ok_or_else(|| {
                            self.context.construct_syntax_error(
                                "unlabeled break must be inside loop or switch",
                            )
                        })?
                        .breaks
                        .push(label);
                }
            }
            Statement::Throw(throw) => {
                self.compile_expr(throw.expr(), true)?;
                self.emit(Opcode::Throw, &[]);
            }
            Statement::Switch(switch) => {
                self.context.push_compile_time_environment(false);
                let push_env =
                    self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
                for case in switch.cases() {
                    self.create_decls(case.body());
                }
                self.emit_opcode(Opcode::LoopStart);

                let start_address = self.next_opcode_location();
                self.push_switch_control_info(None, start_address);

                self.compile_expr(switch.val(), true)?;
                let mut labels = Vec::with_capacity(switch.cases().len());
                for case in switch.cases() {
                    self.compile_expr(case.condition(), true)?;
                    labels.push(self.emit_opcode_with_operand(Opcode::Case));
                }

                let exit = self.emit_opcode_with_operand(Opcode::Default);

                for (label, case) in labels.into_iter().zip(switch.cases()) {
                    self.patch_jump(label);
                    self.compile_statement_list(case.body(), false)?;
                }

                self.patch_jump(exit);
                if let Some(body) = switch.default() {
                    self.create_decls(body);
                    self.compile_statement_list(body, false)?;
                }

                self.pop_switch_control_info();

                self.emit_opcode(Opcode::LoopEnd);

                let (num_bindings, compile_environment) =
                    self.context.pop_compile_time_environment();
                let index_compile_environment = self.push_compile_environment(compile_environment);
                self.patch_jump_with_target(push_env.0, num_bindings as u32);
                self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
                self.emit_opcode(Opcode::PopEnvironment);
            }
            Statement::Return(ret) => {
                if let Some(expr) = ret.expr() {
                    self.compile_expr(expr, true)?;
                } else {
                    self.emit(Opcode::PushUndefined, &[]);
                }
                self.emit(Opcode::Return, &[]);
            }
            Statement::Try(t) => {
                self.push_try_control_info(t.finally().is_some());
                let try_start = self.next_opcode_location();
                self.emit(Opcode::TryStart, &[Self::DUMMY_ADDRESS, 0]);
                self.context.push_compile_time_environment(false);
                let push_env =
                    self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

                self.create_decls(t.block().statement_list());
                self.compile_statement_list(t.block().statement_list(), use_expr)?;

                let (num_bindings, compile_environment) =
                    self.context.pop_compile_time_environment();
                let index_compile_environment = self.push_compile_environment(compile_environment);
                self.patch_jump_with_target(push_env.0, num_bindings as u32);
                self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
                self.emit_opcode(Opcode::PopEnvironment);
                self.emit_opcode(Opcode::TryEnd);

                let finally = self.jump();
                self.patch_jump(Label { index: try_start });

                if let Some(catch) = t.catch() {
                    self.push_try_control_info_catch_start();
                    let catch_start = if t.finally().is_some() {
                        Some(self.emit_opcode_with_operand(Opcode::CatchStart))
                    } else {
                        None
                    };
                    self.context.push_compile_time_environment(false);
                    let push_env =
                        self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
                    if let Some(binding) = catch.parameter() {
                        match binding {
                            Binding::Identifier(ident) => {
                                self.context.create_mutable_binding(*ident, false);
                                self.emit_binding(BindingOpcode::InitLet, *ident);
                            }
                            Binding::Pattern(pattern) => {
                                for ident in pattern.idents() {
                                    self.context.create_mutable_binding(ident, false);
                                }
                                self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                            }
                        }
                    } else {
                        self.emit_opcode(Opcode::Pop);
                    }

                    self.create_decls(catch.block().statement_list());
                    self.compile_statement_list(catch.block().statement_list(), use_expr)?;

                    let (num_bindings, compile_environment) =
                        self.context.pop_compile_time_environment();
                    let index_compile_environment =
                        self.push_compile_environment(compile_environment);
                    self.patch_jump_with_target(push_env.0, num_bindings as u32);
                    self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
                    self.emit_opcode(Opcode::PopEnvironment);
                    if let Some(catch_start) = catch_start {
                        self.emit_opcode(Opcode::CatchEnd);
                        self.patch_jump(catch_start);
                    } else {
                        self.emit_opcode(Opcode::CatchEnd2);
                    }
                }

                self.patch_jump(finally);

                if let Some(finally) = t.finally() {
                    self.emit_opcode(Opcode::FinallyStart);
                    let finally_start_address = self.next_opcode_location();
                    self.push_try_control_info_finally_start(Label {
                        index: finally_start_address,
                    });
                    self.patch_jump_with_target(
                        Label {
                            index: try_start + 4,
                        },
                        finally_start_address,
                    );

                    self.context.push_compile_time_environment(false);
                    let push_env =
                        self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

                    self.create_decls(finally.statement_list());
                    self.compile_statement_list(finally.statement_list(), false)?;

                    let (num_bindings, compile_environment) =
                        self.context.pop_compile_time_environment();
                    let index_compile_environment =
                        self.push_compile_environment(compile_environment);
                    self.patch_jump_with_target(push_env.0, num_bindings as u32);
                    self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
                    self.emit_opcode(Opcode::PopEnvironment);

                    self.emit_opcode(Opcode::FinallyEnd);
                    self.pop_try_control_info(Some(finally_start_address));
                } else {
                    self.pop_try_control_info(None);
                }
            }
            Statement::Empty => {}
            Statement::Expression(expr) => self.compile_expr(expr, use_expr)?,
        }
        Ok(())
    }

    /// Compile a function AST Node into bytecode.
    fn function(
        &mut self,
        function: FunctionSpec<'_>,
        node_kind: NodeKind,
        use_expr: bool,
    ) -> JsResult<()> {
        let (generator, r#async, arrow) = (
            function.is_generator(),
            function.is_async(),
            function.is_arrow(),
        );

        let FunctionSpec {
            name,
            parameters,
            body,
            ..
        } = function;

        let code = FunctionCompiler::new()
            .name(name.map(Identifier::sym))
            .generator(generator)
            .r#async(r#async)
            .strict(self.code_block.strict)
            .arrow(arrow)
            .compile(parameters, body, self.context)?;

        let index = self.code_block.functions.len() as u32;
        self.code_block.functions.push(code);

        if generator && r#async {
            self.emit(Opcode::GetGeneratorAsync, &[index]);
        } else if generator {
            self.emit(Opcode::GetGenerator, &[index]);
        } else if r#async {
            self.emit(Opcode::GetFunctionAsync, &[index]);
        } else {
            self.emit(Opcode::GetFunction, &[index]);
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

        Ok(())
    }

    fn call(&mut self, callable: Callable<'_>, use_expr: bool) -> JsResult<()> {
        #[derive(PartialEq)]
        enum CallKind {
            CallEval,
            Call,
            New,
        }

        let (call, kind) = match callable {
            Callable::Call(call) => match call.expr() {
                Expression::Identifier(ident) if *ident == Sym::EVAL => (call, CallKind::CallEval),
                _ => (call, CallKind::Call),
            },
            Callable::New(new) => (new.call(), CallKind::New),
        };

        match call.expr() {
            Expression::PropertyAccess(access) => {
                self.compile_expr(access.target(), true)?;
                if kind == CallKind::Call {
                    self.emit(Opcode::Dup, &[]);
                }
                match access.field() {
                    PropertyAccessField::Const(field) => {
                        let index = self.get_or_insert_name((*field).into());
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    PropertyAccessField::Expr(field) => {
                        self.compile_expr(field, true)?;
                        self.emit(Opcode::Swap, &[]);
                        self.emit(Opcode::GetPropertyByValue, &[]);
                    }
                }
            }
            Expression::SuperPropertyAccess(access) => {
                if kind == CallKind::Call {
                    self.emit_opcode(Opcode::This);
                }
                self.emit_opcode(Opcode::Super);
                match access.field() {
                    PropertyAccessField::Const(field) => {
                        let index = self.get_or_insert_name((*field).into());
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(expr, true)?;
                        self.emit_opcode(Opcode::Swap);
                        self.emit_opcode(Opcode::GetPropertyByValue);
                    }
                }
            }
            Expression::PrivatePropertyAccess(access) => {
                self.compile_expr(access.target(), true)?;
                if kind == CallKind::Call {
                    self.emit(Opcode::Dup, &[]);
                }
                let index = self.get_or_insert_name(access.field().into());
                self.emit(Opcode::GetPrivateField, &[index]);
            }
            expr => {
                self.compile_expr(expr, true)?;
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
                self.compile_expr(arg, true)?;
                if let Expression::Spread(_) = arg {
                    self.emit_opcode(Opcode::InitIterator);
                    self.emit_opcode(Opcode::PushIteratorToArray);
                } else {
                    self.emit_opcode(Opcode::PushValueToArray);
                }
            }
        } else {
            for arg in call.args() {
                self.compile_expr(arg, true)?;
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
        Ok(())
    }

    #[inline]
    pub fn finish(self) -> CodeBlock {
        self.code_block
    }

    #[inline]
    fn compile_declaration_pattern(
        &mut self,
        pattern: &Pattern,
        def: BindingOpcode,
    ) -> JsResult<()> {
        match pattern {
            Pattern::Object(pattern) => {
                self.emit_opcode(Opcode::ValueNotNullOrUndefined);

                self.emit_opcode(Opcode::RequireObjectCoercible);

                let mut additional_excluded_keys_count = 0;
                let rest_exits = pattern.has_rest();

                for binding in pattern.bindings() {
                    use PatternObjectElement::{
                        AssignmentPropertyAccess, AssignmentRestPropertyAccess, Empty, Pattern,
                        RestProperty, SingleName,
                    };

                    match binding {
                        // ObjectBindingPattern : { }
                        Empty => {}
                        //  SingleNameBinding : BindingIdentifier Initializer[opt]
                        SingleName {
                            ident,
                            name,
                            default_init,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            match name {
                                PropertyName::Literal(name) => {
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit(Opcode::GetPropertyByName, &[index]);
                                }
                                PropertyName::Computed(node) => {
                                    self.compile_expr(node, true)?;
                                    self.emit_opcode(Opcode::Swap);
                                    if rest_exits {
                                        self.emit_opcode(Opcode::GetPropertyByValuePush);
                                    } else {
                                        self.emit_opcode(Opcode::GetPropertyByValue);
                                    }
                                }
                            }

                            if let Some(init) = default_init {
                                let skip =
                                    self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true)?;
                                self.patch_jump(skip);
                            }
                            self.emit_binding(def, *ident);

                            if rest_exits && name.computed().is_some() {
                                self.emit_opcode(Opcode::Swap);
                                additional_excluded_keys_count += 1;
                            }
                        }
                        //  BindingRestProperty : ... BindingIdentifier
                        RestProperty {
                            ident,
                            excluded_keys,
                        } => {
                            self.emit_opcode(Opcode::PushEmptyObject);

                            for key in excluded_keys {
                                self.emit_push_literal(Literal::String(
                                    self.interner().resolve_expect(key.sym()).into_common(false),
                                ));
                            }

                            self.emit(
                                Opcode::CopyDataProperties,
                                &[excluded_keys.len() as u32, additional_excluded_keys_count],
                            );
                            self.emit_binding(def, *ident);
                        }
                        AssignmentRestPropertyAccess {
                            access,
                            excluded_keys,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::PushEmptyObject);
                            for key in excluded_keys {
                                self.emit_push_literal(Literal::String(
                                    self.interner().resolve_expect(key.sym()).into_common(false),
                                ));
                            }
                            self.emit(Opcode::CopyDataProperties, &[excluded_keys.len() as u32, 0]);
                            self.access_set(Access::Property { access }, None, false)?;
                        }
                        AssignmentPropertyAccess {
                            name,
                            access,
                            default_init,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            match name {
                                PropertyName::Literal(name) => {
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit(Opcode::GetPropertyByName, &[index]);
                                }
                                PropertyName::Computed(node) => {
                                    self.compile_expr(node, true)?;
                                    self.emit_opcode(Opcode::Swap);
                                    if rest_exits {
                                        self.emit_opcode(Opcode::GetPropertyByValuePush);
                                    } else {
                                        self.emit_opcode(Opcode::GetPropertyByValue);
                                    }
                                }
                            }

                            if let Some(init) = default_init {
                                let skip =
                                    self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true)?;
                                self.patch_jump(skip);
                            }

                            self.access_set(Access::Property { access }, None, false)?;

                            if rest_exits && name.computed().is_some() {
                                self.emit_opcode(Opcode::Swap);
                                additional_excluded_keys_count += 1;
                            }
                        }
                        Pattern {
                            name,
                            pattern,
                            default_init,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            match name {
                                PropertyName::Literal(name) => {
                                    let index = self.get_or_insert_name((*name).into());
                                    self.emit(Opcode::GetPropertyByName, &[index]);
                                }
                                PropertyName::Computed(node) => {
                                    self.compile_expr(node, true)?;
                                    self.emit_opcode(Opcode::Swap);
                                    self.emit_opcode(Opcode::GetPropertyByValue);
                                }
                            }

                            if let Some(init) = default_init {
                                let skip =
                                    self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true)?;
                                self.patch_jump(skip);
                            }

                            self.compile_declaration_pattern(pattern, def)?;
                        }
                    }
                }

                if !rest_exits {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Pattern::Array(pattern) => {
                self.emit_opcode(Opcode::ValueNotNullOrUndefined);
                self.emit_opcode(Opcode::InitIterator);

                for binding in pattern.bindings().iter() {
                    use PatternArrayElement::{
                        Elision, Empty, Pattern, PatternRest, PropertyAccess, PropertyAccessRest,
                        SingleName, SingleNameRest,
                    };

                    match binding {
                        // ArrayBindingPattern : [ ]
                        Empty => {}
                        // ArrayBindingPattern : [ Elision ]
                        Elision => {
                            self.emit_opcode(Opcode::IteratorNext);
                            self.emit_opcode(Opcode::Pop);
                        }
                        // SingleNameBinding : BindingIdentifier Initializer[opt]
                        SingleName {
                            ident,
                            default_init,
                        } => {
                            self.emit_opcode(Opcode::IteratorNext);
                            if let Some(init) = default_init {
                                let skip =
                                    self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true)?;
                                self.patch_jump(skip);
                            }
                            self.emit_binding(def, *ident);
                        }
                        PropertyAccess { access } => {
                            self.emit_opcode(Opcode::IteratorNext);
                            self.access_set(Access::Property { access }, None, false)?;
                        }
                        // BindingElement : BindingPattern Initializer[opt]
                        Pattern {
                            pattern,
                            default_init,
                        } => {
                            self.emit_opcode(Opcode::IteratorNext);

                            if let Some(init) = default_init {
                                let skip =
                                    self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true)?;
                                self.patch_jump(skip);
                            }

                            self.compile_declaration_pattern(pattern, def)?;
                        }
                        // BindingRestElement : ... BindingIdentifier
                        SingleNameRest { ident } => {
                            self.emit_opcode(Opcode::IteratorToArray);
                            self.emit_binding(def, *ident);
                        }
                        PropertyAccessRest { access } => {
                            self.emit_opcode(Opcode::IteratorToArray);
                            self.access_set(Access::Property { access }, None, false)?;
                        }
                        // BindingRestElement : ... BindingPattern
                        PatternRest { pattern } => {
                            self.emit_opcode(Opcode::IteratorToArray);
                            self.compile_declaration_pattern(pattern, def)?;
                        }
                    }
                }

                self.emit_opcode(Opcode::IteratorClose);
            }
        }
        Ok(())
    }

    pub(crate) fn create_decls(&mut self, stmt_list: &StatementList) {
        for node in stmt_list.statements() {
            self.create_decls_from_stmt_list_item(node);
        }
    }

    pub(crate) fn create_decls_from_var_decl(&mut self, list: &VarDeclaration) -> bool {
        let mut has_identifier_argument = false;
        for decl in list.0.as_ref() {
            match decl.binding() {
                Binding::Identifier(ident) => {
                    let ident = ident;
                    if *ident == Sym::ARGUMENTS {
                        has_identifier_argument = true;
                    }
                    self.context.create_mutable_binding(*ident, true);
                }
                Binding::Pattern(pattern) => {
                    for ident in pattern.idents() {
                        if ident == Sym::ARGUMENTS {
                            has_identifier_argument = true;
                        }
                        self.context.create_mutable_binding(ident, true);
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
                            self.context.create_mutable_binding(*ident, false);
                        }
                        Binding::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                if ident == Sym::ARGUMENTS {
                                    has_identifier_argument = true;
                                }
                                self.context.create_mutable_binding(ident, false);
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
                            self.context.create_immutable_binding(*ident);
                        }
                        Binding::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                if ident == Sym::ARGUMENTS {
                                    has_identifier_argument = true;
                                }
                                self.context.create_immutable_binding(ident);
                            }
                        }
                    }
                }
            }
        }
        has_identifier_argument
    }

    #[inline]
    pub(crate) fn create_decls_from_decl(&mut self, declaration: &Declaration) -> bool {
        match declaration {
            Declaration::Lexical(decl) => self.create_decls_from_lexical_decl(decl),
            Declaration::Function(decl) => {
                let ident = decl.name().expect("function declaration must have a name");
                self.context.create_mutable_binding(ident, true);
                ident == Sym::ARGUMENTS
            }
            Declaration::Generator(decl) => {
                let ident = decl.name().expect("generator declaration must have a name");

                self.context.create_mutable_binding(ident, true);
                ident == Sym::ARGUMENTS
            }
            Declaration::AsyncFunction(decl) => {
                let ident = decl
                    .name()
                    .expect("async function declaration must have a name");
                self.context.create_mutable_binding(ident, true);
                ident == Sym::ARGUMENTS
            }
            Declaration::AsyncGenerator(decl) => {
                let ident = decl
                    .name()
                    .expect("async generator declaration must have a name");
                self.context.create_mutable_binding(ident, true);
                ident == Sym::ARGUMENTS
            }
            Declaration::Class(decl) => {
                let ident = decl.name().expect("class declaration must have a name");
                self.context.create_mutable_binding(ident, false);
                false
            }
        }
    }

    #[inline]
    pub(crate) fn create_decls_from_stmt(&mut self, statement: &Statement) -> bool {
        match statement {
            Statement::Var(var) => self.create_decls_from_var_decl(var),
            Statement::DoWhileLoop(do_while_loop) => {
                if !matches!(do_while_loop.body(), Statement::Block(_)) {
                    self.create_decls_from_stmt(do_while_loop.body());
                }
                false
            }
            Statement::ForInLoop(for_in_loop) => {
                if !matches!(for_in_loop.body(), Statement::Block(_)) {
                    self.create_decls_from_stmt(for_in_loop.body());
                }
                false
            }
            Statement::ForOfLoop(for_of_loop) => {
                if !matches!(for_of_loop.body(), Statement::Block(_)) {
                    self.create_decls_from_stmt(for_of_loop.body());
                }
                false
            }
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn create_decls_from_stmt_list_item(&mut self, item: &StatementListItem) -> bool {
        match item {
            StatementListItem::Declaration(decl) => self.create_decls_from_decl(decl),
            StatementListItem::Statement(stmt) => self.create_decls_from_stmt(stmt),
        }
    }

    /// This function compiles a class declaration or expression.
    ///
    /// The compilation of a class declaration and expression is mostly equal.
    /// A class declaration binds the resulting class object to it's identifier.
    /// A class expression leaves the resulting class object on the stack for following operations.
    fn class(&mut self, class: &Class, expression: bool) -> JsResult<()> {
        let code = CodeBlock::new(
            class.name().map_or(Sym::EMPTY_STRING, Identifier::sym),
            0,
            true,
        );
        let mut compiler = ByteCompiler {
            code_block: code,
            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            bindings_map: FxHashMap::default(),
            jump_info: Vec::new(),
            in_async_generator: false,
            context: self.context,
        };
        compiler.context.push_compile_time_environment(true);

        if let Some(expr) = class.constructor() {
            compiler.code_block.length = expr.parameters().length();
            compiler.code_block.params = expr.parameters().clone();
            compiler
                .context
                .create_mutable_binding(Sym::ARGUMENTS.into(), false);
            compiler.code_block.arguments_binding = Some(
                compiler
                    .context
                    .initialize_mutable_binding(Sym::ARGUMENTS.into(), false),
            );
            for parameter in expr.parameters().parameters.iter() {
                if parameter.is_rest_param() {
                    compiler.emit_opcode(Opcode::RestParameterInit);
                }

                match parameter.variable().binding() {
                    Binding::Identifier(ident) => {
                        compiler.context.create_mutable_binding(*ident, false);
                        if let Some(init) = parameter.variable().init() {
                            let skip =
                                compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                            compiler.compile_expr(init, true)?;
                            compiler.patch_jump(skip);
                        }
                        compiler.emit_binding(BindingOpcode::InitArg, *ident);
                    }
                    Binding::Pattern(pattern) => {
                        for ident in pattern.idents() {
                            compiler.context.create_mutable_binding(ident, false);
                        }
                        compiler.compile_declaration_pattern(pattern, BindingOpcode::InitArg)?;
                    }
                }
            }
            if !expr.parameters().has_rest_parameter() {
                compiler.emit_opcode(Opcode::RestParameterPop);
            }
            let env_label = if expr.parameters().has_expressions() {
                compiler.code_block.num_bindings = compiler.context.get_binding_number();
                compiler.context.push_compile_time_environment(true);
                compiler.code_block.function_environment_push_location =
                    compiler.next_opcode_location();
                Some(compiler.emit_opcode_with_two_operands(Opcode::PushFunctionEnvironment))
            } else {
                None
            };
            compiler.create_decls(expr.body());
            compiler.compile_statement_list(expr.body(), false)?;
            if let Some(env_label) = env_label {
                let (num_bindings, compile_environment) =
                    compiler.context.pop_compile_time_environment();
                let index_compile_environment =
                    compiler.push_compile_environment(compile_environment);
                compiler.patch_jump_with_target(env_label.0, num_bindings as u32);
                compiler.patch_jump_with_target(env_label.1, index_compile_environment as u32);
                let (_, compile_environment) = compiler.context.pop_compile_time_environment();
                compiler.push_compile_environment(compile_environment);
            } else {
                let (num_bindings, compile_environment) =
                    compiler.context.pop_compile_time_environment();
                compiler
                    .code_block
                    .compile_environments
                    .push(compile_environment);
                compiler.code_block.num_bindings = num_bindings;
                compiler.code_block.is_class_constructor = true;
            }
        } else {
            if class.super_ref().is_some() {
                compiler.emit_opcode(Opcode::SuperCallDerived);
            }
            let (num_bindings, compile_environment) =
                compiler.context.pop_compile_time_environment();
            compiler
                .code_block
                .compile_environments
                .push(compile_environment);
            compiler.code_block.num_bindings = num_bindings;
            compiler.code_block.is_class_constructor = true;
        }

        compiler.emit_opcode(Opcode::PushUndefined);
        compiler.emit_opcode(Opcode::Return);

        let code = Gc::new(compiler.finish());
        let index = self.code_block.functions.len() as u32;
        self.code_block.functions.push(code);
        self.emit(Opcode::GetFunction, &[index]);

        self.emit_opcode(Opcode::Dup);
        if let Some(node) = class.super_ref() {
            self.compile_expr(node, true)?;
            self.emit_opcode(Opcode::PushClassPrototype);
        } else {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.emit_opcode(Opcode::SetClassPrototype);
        self.emit_opcode(Opcode::Swap);

        for element in class.elements() {
            match element {
                ClassElement::StaticMethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassGetterByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassGetterByValue);
                            }
                        },
                        MethodDefinition::Set(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassSetterByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassSetterByValue);
                            }
                        },
                        MethodDefinition::Ordinary(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::Async(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::Generator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::AsyncGenerator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                    }
                }
                ClassElement::PrivateStaticMethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::SetPrivateGetter, &[index]);
                        }
                        MethodDefinition::Set(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::SetPrivateSetter, &[index]);
                        }
                        MethodDefinition::Ordinary(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::SetPrivateMethod, &[index]);
                        }
                        MethodDefinition::Async(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::SetPrivateMethod, &[index]);
                        }
                        MethodDefinition::Generator(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::SetPrivateMethod, &[index]);
                        }
                        MethodDefinition::AsyncGenerator(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::SetPrivateMethod, &[index]);
                        }
                    }
                }
                ClassElement::FieldDefinition(name, field) => {
                    self.emit_opcode(Opcode::Dup);
                    match name {
                        PropertyName::Literal(name) => {
                            self.emit_push_literal(Literal::String(
                                self.interner().resolve_expect(*name).into_common(false),
                            ));
                        }
                        PropertyName::Computed(name) => {
                            self.compile_expr(name, true)?;
                        }
                    }
                    let field_code = CodeBlock::new(Sym::EMPTY_STRING, 0, true);
                    let mut field_compiler = ByteCompiler {
                        code_block: field_code,
                        literals_map: FxHashMap::default(),
                        names_map: FxHashMap::default(),
                        bindings_map: FxHashMap::default(),
                        jump_info: Vec::new(),
                        in_async_generator: false,
                        context: self.context,
                    };
                    field_compiler.context.push_compile_time_environment(true);
                    if let Some(node) = field {
                        field_compiler.compile_expr(node, true)?;
                    } else {
                        field_compiler.emit_opcode(Opcode::PushUndefined);
                    }
                    let (num_bindings, compile_environment) =
                        field_compiler.context.pop_compile_time_environment();
                    field_compiler
                        .code_block
                        .compile_environments
                        .push(compile_environment);
                    field_compiler.code_block.num_bindings = num_bindings;
                    field_compiler.emit_opcode(Opcode::Return);

                    let code = Gc::new(field_compiler.finish());
                    let index = self.code_block.functions.len() as u32;
                    self.code_block.functions.push(code);
                    self.emit(Opcode::GetFunction, &[index]);
                    self.emit_opcode(Opcode::PushClassField);
                }
                ClassElement::PrivateFieldDefinition(name, field) => {
                    self.emit_opcode(Opcode::Dup);
                    let name_index = self.get_or_insert_name((*name).into());
                    let field_code = CodeBlock::new(Sym::EMPTY_STRING, 0, true);
                    let mut field_compiler = ByteCompiler {
                        code_block: field_code,
                        literals_map: FxHashMap::default(),
                        names_map: FxHashMap::default(),
                        bindings_map: FxHashMap::default(),
                        jump_info: Vec::new(),
                        in_async_generator: false,
                        context: self.context,
                    };
                    field_compiler.context.push_compile_time_environment(true);
                    if let Some(node) = field {
                        field_compiler.compile_expr(node, true)?;
                    } else {
                        field_compiler.emit_opcode(Opcode::PushUndefined);
                    }
                    let (num_bindings, compile_environment) =
                        field_compiler.context.pop_compile_time_environment();
                    field_compiler
                        .code_block
                        .compile_environments
                        .push(compile_environment);
                    field_compiler.code_block.num_bindings = num_bindings;
                    field_compiler.emit_opcode(Opcode::Return);

                    let code = Gc::new(field_compiler.finish());
                    let index = self.code_block.functions.len() as u32;
                    self.code_block.functions.push(code);
                    self.emit(Opcode::GetFunction, &[index]);
                    self.emit(Opcode::PushClassFieldPrivate, &[name_index]);
                }
                ClassElement::StaticFieldDefinition(name, field) => {
                    self.emit_opcode(Opcode::Dup);
                    match name {
                        PropertyName::Literal(name) => {
                            if let Some(node) = field {
                                self.compile_expr(node, true)?;
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            }
                            self.emit_opcode(Opcode::Swap);
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                        }
                        PropertyName::Computed(name_node) => {
                            self.compile_expr(name_node, true)?;
                            self.emit_opcode(Opcode::ToPropertyKey);
                            if let Some(node) = field {
                                self.compile_expr(node, true)?;
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            }
                            self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                        }
                    }
                }
                ClassElement::PrivateStaticFieldDefinition(name, field) => {
                    self.emit_opcode(Opcode::Dup);
                    if let Some(node) = field {
                        self.compile_expr(node, true)?;
                    } else {
                        self.emit_opcode(Opcode::PushUndefined);
                    }
                    let index = self.get_or_insert_name((*name).into());
                    self.emit(Opcode::SetPrivateField, &[index]);
                }
                ClassElement::StaticBlock(statement_list) => {
                    self.emit_opcode(Opcode::Dup);
                    let mut compiler = ByteCompiler::new(Sym::EMPTY_STRING, true, self.context);
                    compiler.context.push_compile_time_environment(true);
                    compiler.create_decls(statement_list);
                    compiler.compile_statement_list(statement_list, false)?;
                    let (num_bindings, compile_environment) =
                        compiler.context.pop_compile_time_environment();
                    compiler
                        .code_block
                        .compile_environments
                        .push(compile_environment);
                    compiler.code_block.num_bindings = num_bindings;

                    let code = Gc::new(compiler.finish());
                    let index = self.code_block.functions.len() as u32;
                    self.code_block.functions.push(code);
                    self.emit(Opcode::GetFunction, &[index]);
                    self.emit_opcode(Opcode::SetHomeObject);
                    self.emit(Opcode::Call, &[0]);
                    self.emit_opcode(Opcode::Pop);
                }
                ClassElement::PrivateMethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::PushClassPrivateGetter, &[index]);
                        }
                        MethodDefinition::Set(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::PushClassPrivateSetter, &[index]);
                        }
                        MethodDefinition::Ordinary(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::PushClassPrivateMethod, &[index]);
                        }
                        MethodDefinition::Async(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::PushClassPrivateMethod, &[index]);
                        }
                        MethodDefinition::Generator(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::PushClassPrivateMethod, &[index]);
                        }
                        MethodDefinition::AsyncGenerator(expr) => {
                            self.function(expr.into(), NodeKind::Expression, true)?;
                            let index = self.get_or_insert_name((*name).into());
                            self.emit(Opcode::PushClassPrivateMethod, &[index]);
                        }
                    }
                }
                ClassElement::MethodDefinition(..) => {}
            }
        }

        self.emit_opcode(Opcode::Swap);

        for element in class.elements() {
            match element {
                ClassElement::MethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassGetterByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassGetterByValue);
                            }
                        },
                        MethodDefinition::Set(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassSetterByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassSetterByValue);
                            }
                        },
                        MethodDefinition::Ordinary(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::Async(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::Generator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                        MethodDefinition::AsyncGenerator(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name((*name).into());
                                self.emit(Opcode::DefineClassMethodByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_expr(name_node, true)?;
                                self.emit_opcode(Opcode::ToPropertyKey);
                                self.function(expr.into(), NodeKind::Expression, true)?;
                                self.emit_opcode(Opcode::DefineClassMethodByValue);
                            }
                        },
                    }
                }
                ClassElement::PrivateMethodDefinition(..)
                | ClassElement::PrivateFieldDefinition(..)
                | ClassElement::StaticFieldDefinition(..)
                | ClassElement::PrivateStaticFieldDefinition(..)
                | ClassElement::StaticMethodDefinition(..)
                | ClassElement::PrivateStaticMethodDefinition(..)
                | ClassElement::StaticBlock(..)
                | ClassElement::FieldDefinition(..) => {}
            }
        }

        self.emit_opcode(Opcode::Pop);

        if !expression {
            self.emit_binding(
                BindingOpcode::InitVar,
                class.name().expect("class statements must have a name"),
            );
        }
        Ok(())
    }
}
