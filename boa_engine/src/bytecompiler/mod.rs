mod function;

use crate::{
    environments::{BindingLocator, CompileTimeEnvironment},
    vm::{BindingOpcode, CodeBlock, Opcode},
    Context, JsBigInt, JsNativeError, JsResult, JsString, JsValue,
};
use boa_ast::{
    declaration::{Binding, LexicalDeclaration, VarDeclaration},
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        literal::{self, TemplateElement},
        operator::{
            assign::{AssignOp, AssignTarget},
            binary::{ArithmeticOp, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
            unary::UnaryOp,
        },
        Call, Identifier, New, Optional, OptionalOperationKind,
    },
    function::{
        ArrowFunction, AsyncArrowFunction, AsyncFunction, AsyncGenerator, Class, ClassElement,
        FormalParameterList, Function, Generator,
    },
    operations::bound_names,
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    property::{MethodDefinition, PropertyDefinition, PropertyName},
    statement::{
        iteration::{ForLoopInitializer, IterableLoopInitializer},
        Block, DoWhileLoop, ForInLoop, ForLoop, ForOfLoop, LabelledItem, WhileLoop,
    },
    Declaration, Expression, Statement, StatementList, StatementListItem,
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
    AsyncArrow,
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
    has_binding_identifier: bool,
}

impl<'a> FunctionSpec<'a> {
    #[inline]
    fn is_arrow(&self) -> bool {
        matches!(self.kind, FunctionKind::Arrow | FunctionKind::AsyncArrow)
    }

    #[inline]
    fn is_async(&self) -> bool {
        matches!(
            self.kind,
            FunctionKind::Async | FunctionKind::AsyncGenerator | FunctionKind::AsyncArrow
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
    This,
}

impl Access<'_> {
    fn from_assign_target(target: &AssignTarget) -> Result<Access<'_>, &Pattern> {
        match target {
            AssignTarget::Identifier(ident) => Ok(Access::Variable { name: *ident }),
            AssignTarget::Access(access) => Ok(Access::Property { access }),
            AssignTarget::Pattern(pat) => Err(pat),
        }
    }

    fn from_expression(expr: &Expression) -> Option<Access<'_>> {
        match expr {
            Expression::Identifier(name) => Some(Access::Variable { name: *name }),
            Expression::PropertyAccess(access) => Some(Access::Property { access }),
            Expression::This => Some(Access::This),
            _ => None,
        }
    }
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
        self.code_block.code.extend(value.to_ne_bytes());
    }

    #[inline]
    fn emit_u32(&mut self, value: u32) {
        self.code_block.code.extend(value.to_ne_bytes());
    }

    #[inline]
    fn emit_u16(&mut self, value: u16) {
        self.code_block.code.extend(value.to_ne_bytes());
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

    #[inline]
    fn jump_if_null_or_undefined(&mut self) -> Label {
        let index = self.next_opcode_location();
        self.emit(Opcode::JumpIfNullOrUndefined, &[Self::DUMMY_ADDRESS]);

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
    fn access_get(&mut self, access: Access<'_>, use_expr: bool) -> JsResult<()> {
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
                        self.compile_expr(access.target(), true)?;
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(access.target(), true)?;
                        self.compile_expr(expr, true)?;
                        self.emit(Opcode::GetPropertyByValue, &[]);
                    }
                },
                PropertyAccess::Private(access) => {
                    let index = self.get_or_insert_name(access.field().into());
                    self.compile_expr(access.target(), true)?;
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
                        self.compile_expr(expr, true)?;
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
        Ok(())
    }

    // The wrap is needed so it can match the function signature.
    #[allow(clippy::unnecessary_wraps)]
    fn access_set_top_of_stack_expr_fn(compiler: &mut ByteCompiler<'_>, level: u8) -> JsResult<()> {
        match level {
            0 => {}
            1 => compiler.emit_opcode(Opcode::Swap),
            _ => {
                compiler.emit_opcode(Opcode::RotateLeft);
                compiler.emit_u8(level + 1);
            }
        }
        Ok(())
    }

    #[inline]
    fn access_set<F, R>(&mut self, access: Access<'_>, use_expr: bool, expr_fn: F) -> JsResult<R>
    where
        F: FnOnce(&mut ByteCompiler<'_>, u8) -> JsResult<R>,
    {
        match access {
            Access::Variable { name } => {
                let result = expr_fn(self, 0);
                if use_expr {
                    self.emit(Opcode::Dup, &[]);
                }
                let binding = self.context.set_mutable_binding(name);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::SetName, &[index]);
                result
            }
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        self.compile_expr(access.target(), true)?;
                        let result = expr_fn(self, 1);
                        let index = self.get_or_insert_name((*name).into());

                        self.emit(Opcode::SetPropertyByName, &[index]);
                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                        result
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(access.target(), true)?;
                        self.compile_expr(expr, true)?;
                        let result = expr_fn(self, 2);
                        self.emit(Opcode::SetPropertyByValue, &[]);
                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                        result
                    }
                },
                PropertyAccess::Private(access) => {
                    self.compile_expr(access.target(), true)?;
                    let result = expr_fn(self, 1);
                    let index = self.get_or_insert_name(access.field().into());
                    self.emit(Opcode::AssignPrivateField, &[index]);
                    if !use_expr {
                        self.emit(Opcode::Pop, &[]);
                    }
                    result
                }
                PropertyAccess::Super(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        self.emit(Opcode::Super, &[]);
                        let result = expr_fn(self, 1);
                        let index = self.get_or_insert_name((*name).into());
                        self.emit(Opcode::SetPropertyByName, &[index]);
                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                        result
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.emit(Opcode::Super, &[]);
                        self.compile_expr(expr, true)?;
                        let result = expr_fn(self, 0);
                        self.emit(Opcode::SetPropertyByValue, &[]);
                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                        result
                    }
                },
            },
            Access::This => todo!("access_set `this`"),
        }
    }

    #[inline]
    fn access_delete(&mut self, access: Access<'_>) -> JsResult<()> {
        match access {
            Access::Property { access } => match access {
                PropertyAccess::Simple(access) => match access.field() {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name((*name).into());
                        self.compile_expr(access.target(), true)?;
                        self.emit(Opcode::DeletePropertyByName, &[index]);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(access.target(), true)?;
                        self.compile_expr(expr, true)?;
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
        Ok(())
    }

    #[inline]
    pub fn compile_statement_list(
        &mut self,
        list: &StatementList,
        use_expr: bool,
        configurable_globals: bool,
    ) -> JsResult<()> {
        if let Some((last, items)) = list.statements().split_last() {
            for node in items {
                self.compile_stmt_list_item(node, false, configurable_globals)?;
            }
            self.compile_stmt_list_item(last, use_expr, configurable_globals)?;
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

        self.create_decls(list, true);

        if let Some((last, items)) = list.statements().split_last() {
            for node in items {
                self.compile_stmt_list_item(node, false, true)?;
            }
            self.compile_stmt_list_item(last, use_expr, true)?;
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
                        // TODO: promote to an early error.
                        let access = Access::from_expression(unary.target()).ok_or_else(|| {
                            JsNativeError::syntax().with_message("Invalid increment operand")
                        })?;

                        self.access_set(access, true, |compiler, _| {
                            compiler.compile_expr(unary.target(), true)?;
                            compiler.emit(Opcode::Inc, &[]);
                            Ok(())
                        })?;

                        None
                    }
                    UnaryOp::DecrementPre => {
                        // TODO: promote to an early error.
                        let access = Access::from_expression(unary.target()).ok_or_else(|| {
                            JsNativeError::syntax().with_message("Invalid decrement operand")
                        })?;

                        self.access_set(access, true, |compiler, _| {
                            compiler.compile_expr(unary.target(), true)?;
                            compiler.emit(Opcode::Dec, &[]);
                            Ok(())
                        })?;
                        None
                    }
                    UnaryOp::IncrementPost => {
                        // TODO: promote to an early error.
                        let access = Access::from_expression(unary.target()).ok_or_else(|| {
                            JsNativeError::syntax().with_message("Invalid increment operand")
                        })?;

                        self.access_set(access, false, |compiler, level| {
                            compiler.compile_expr(unary.target(), true)?;
                            compiler.emit(Opcode::IncPost, &[]);
                            compiler.emit_opcode(Opcode::RotateRight);
                            compiler.emit_u8(level + 2);
                            Ok(())
                        })?;

                        None
                    }
                    UnaryOp::DecrementPost => {
                        // TODO: promote to an early error.
                        let access = Access::from_expression(unary.target()).ok_or_else(|| {
                            JsNativeError::syntax().with_message("Invalid decrement operand")
                        })?;

                        self.access_set(access, false, |compiler, level| {
                            compiler.compile_expr(unary.target(), true)?;
                            compiler.emit(Opcode::DecPost, &[]);
                            compiler.emit_opcode(Opcode::RotateRight);
                            compiler.emit_u8(level + 2);
                            Ok(())
                        })?;

                        None
                    }
                    UnaryOp::Delete => {
                        if let Some(access) = Access::from_expression(unary.target()) {
                            self.access_delete(access)?;
                        } else {
                            self.compile_expr(unary.target(), false)?;
                            self.emit(Opcode::PushTrue, &[]);
                        }
                        None
                    }
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
            Expression::Assign(assign) if assign.op() == AssignOp::Assign => {
                match Access::from_assign_target(assign.lhs()) {
                    Ok(access) => self.access_set(access, use_expr, |compiler, _| {
                        compiler.compile_expr(assign.rhs(), true)?;
                        Ok(())
                    })?,
                    Err(pattern) => {
                        self.compile_expr(assign.rhs(), true)?;
                        if use_expr {
                            self.emit_opcode(Opcode::Dup);
                        }
                        self.compile_declaration_pattern(pattern, BindingOpcode::SetName)?;
                    }
                }
            }
            Expression::Assign(assign) => {
                let access = Access::from_assign_target(assign.lhs())
                    .expect("patterns should throw early errors on complex assignment operators");

                let shortcircuit_operator_compilation =
                    |compiler: &mut ByteCompiler<'_>, opcode: Opcode| -> JsResult<()> {
                        let (early_exit, pop_count) =
                            compiler.access_set(access, use_expr, |compiler, level| {
                                compiler.access_get(access, true)?;
                                let early_exit = compiler.emit_opcode_with_operand(opcode);
                                compiler.compile_expr(assign.rhs(), true)?;
                                Ok((early_exit, level))
                            })?;
                        if pop_count == 0 {
                            compiler.patch_jump(early_exit);
                        } else {
                            let exit = compiler.emit_opcode_with_operand(Opcode::Jump);
                            compiler.patch_jump(early_exit);
                            for _ in 0..pop_count {
                                compiler.emit_opcode(Opcode::Swap);
                                compiler.emit_opcode(Opcode::Pop);
                            }
                            compiler.patch_jump(exit);
                        }
                        Ok(())
                    };

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
                        shortcircuit_operator_compilation(self, Opcode::LogicalAnd)?;
                        return Ok(());
                    }
                    AssignOp::BoolOr => {
                        shortcircuit_operator_compilation(self, Opcode::LogicalOr)?;
                        return Ok(());
                    }
                    AssignOp::Coalesce => {
                        shortcircuit_operator_compilation(self, Opcode::Coalesce)?;
                        return Ok(());
                    }
                };

                self.access_set(access, use_expr, |compiler, _| {
                    compiler.access_get(access, true)?;
                    compiler.compile_expr(assign.rhs(), true)?;
                    compiler.emit(opcode, &[]);
                    Ok(())
                })?;
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
                            // TODO: set function name for getter and setters
                            MethodDefinition::Get(expr) => match name {
                                PropertyName::Literal(name) => {
                                    self.function(expr.into(), NodeKind::Expression, true)?;
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
                            // TODO: set function name for getter and setters
                            MethodDefinition::Set(expr) => match name {
                                PropertyName::Literal(name) => {
                                    self.function(expr.into(), NodeKind::Expression, true)?;
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
                        // TODO: Promote to early errors
                        PropertyDefinition::CoverInitializedName(_, _) => {
                            return Err(JsNativeError::syntax()
                                .with_message("invalid assignment pattern in object literal")
                                .into())
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
            Expression::Conditional(op) => {
                self.compile_expr(op.condition(), true)?;
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
            Expression::Spread(spread) => self.compile_expr(spread.target(), true)?,
            Expression::Function(function) => {
                self.function(function.into(), NodeKind::Expression, use_expr)?;
            }
            Expression::ArrowFunction(function) => {
                self.function(function.into(), NodeKind::Expression, use_expr)?;
            }
            Expression::AsyncArrowFunction(function) => {
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
                self.compile_expr(expr.target(), true)?;
                self.emit_opcode(Opcode::Await);
                self.emit_opcode(Opcode::GeneratorNext);
                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Expression::Yield(r#yield) => {
                if let Some(expr) = r#yield.target() {
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
                    Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                        self.compile_expr(access.target(), true)?;
                        self.emit(Opcode::Dup, &[]);
                        match access.field() {
                            PropertyAccessField::Const(field) => {
                                let index = self.get_or_insert_name((*field).into());
                                self.emit(Opcode::GetPropertyByName, &[index]);
                            }
                            PropertyAccessField::Expr(field) => {
                                self.compile_expr(field, true)?;
                                self.emit(Opcode::GetPropertyByValue, &[]);
                            }
                        }
                    }
                    Expression::PropertyAccess(PropertyAccess::Private(access)) => {
                        self.compile_expr(access.target(), true)?;
                        self.emit(Opcode::Dup, &[]);
                        let index = self.get_or_insert_name(access.field().into());
                        self.emit(Opcode::GetPrivateField, &[index]);
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

                let index = self.get_or_insert_name(Sym::RAW.into());
                self.emit(Opcode::SetPropertyByName, &[index]);
                self.emit(Opcode::Pop, &[]);

                for expr in template.exprs() {
                    self.compile_expr(expr, true)?;
                }

                self.emit(Opcode::Call, &[(template.exprs().len() + 1) as u32]);
            }
            Expression::Class(class) => self.class(class, true)?,
            Expression::SuperCall(super_call) => {
                let contains_spread = super_call
                    .arguments()
                    .iter()
                    .any(|arg| matches!(arg, Expression::Spread(_)));

                if contains_spread {
                    self.emit_opcode(Opcode::PushNewArray);
                    for arg in super_call.arguments() {
                        self.compile_expr(arg, true)?;
                        if let Expression::Spread(_) = arg {
                            self.emit_opcode(Opcode::InitIterator);
                            self.emit_opcode(Opcode::PushIteratorToArray);
                        } else {
                            self.emit_opcode(Opcode::PushValueToArray);
                        }
                    }
                } else {
                    for arg in super_call.arguments() {
                        self.compile_expr(arg, true)?;
                    }
                }

                if contains_spread {
                    self.emit_opcode(Opcode::SuperCallSpread);
                } else {
                    self.emit(Opcode::SuperCall, &[super_call.arguments().len() as u32]);
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
            Expression::Optional(opt) => {
                self.compile_optional_preserve_this(opt)?;

                self.emit_opcode(Opcode::Swap);
                self.emit_opcode(Opcode::Pop);

                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            // TODO: try to remove this variant somehow
            Expression::FormalParameterList(_) => unreachable!(),
        }
        Ok(())
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
    fn compile_access_preserve_this(&mut self, access: &PropertyAccess) -> JsResult<()> {
        match access {
            PropertyAccess::Simple(access) => {
                self.compile_expr(access.target(), true)?;
                self.emit_opcode(Opcode::Dup);
                match access.field() {
                    PropertyAccessField::Const(field) => {
                        let index = self.get_or_insert_name((*field).into());
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    PropertyAccessField::Expr(field) => {
                        self.compile_expr(field, true)?;
                        self.emit_opcode(Opcode::GetPropertyByValue);
                    }
                }
            }
            PropertyAccess::Private(access) => {
                self.compile_expr(access.target(), true)?;
                self.emit_opcode(Opcode::Dup);
                let index = self.get_or_insert_name(access.field().into());
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
                        self.compile_expr(expr, true)?;
                        self.emit_opcode(Opcode::GetPropertyByValue);
                    }
                }
            }
        }
        Ok(())
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
    fn compile_optional_preserve_this(&mut self, optional: &Optional) -> JsResult<()> {
        let mut jumps = Vec::with_capacity(optional.chain().len());

        match optional.target() {
            Expression::PropertyAccess(access) => {
                self.compile_access_preserve_this(access)?;
            }
            Expression::Optional(opt) => self.compile_optional_preserve_this(opt)?,
            expr => {
                self.emit(Opcode::PushUndefined, &[]);
                self.compile_expr(expr, true)?;
            }
        }
        jumps.push(self.jump_if_null_or_undefined());

        let (first, rest) = optional
            .chain()
            .split_first()
            .expect("chain must have at least one element");
        assert!(first.shorted());

        self.compile_optional_item_kind(first.kind())?;

        for item in rest {
            if item.shorted() {
                jumps.push(self.jump_if_null_or_undefined());
            }
            self.compile_optional_item_kind(item.kind())?;
        }
        let skip_undef = self.jump();

        for label in jumps {
            self.patch_jump(label);
        }

        self.emit_opcode(Opcode::Pop);
        self.emit_opcode(Opcode::PushUndefined);

        self.patch_jump(skip_undef);

        Ok(())
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
    fn compile_optional_item_kind(&mut self, kind: &OptionalOperationKind) -> JsResult<()> {
        match kind {
            OptionalOperationKind::SimplePropertyAccess { field } => {
                self.emit_opcode(Opcode::Dup);
                match field {
                    PropertyAccessField::Const(name) => {
                        let index = self.get_or_insert_name((*name).into());
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    PropertyAccessField::Expr(expr) => {
                        self.compile_expr(expr, true)?;
                        self.emit(Opcode::GetPropertyByValue, &[]);
                    }
                }
                self.emit_opcode(Opcode::RotateLeft);
                self.emit_u8(3);
                self.emit_opcode(Opcode::Pop);
            }
            OptionalOperationKind::PrivatePropertyAccess { field } => {
                self.emit_opcode(Opcode::Dup);
                let index = self.get_or_insert_name((*field).into());
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
                        self.compile_expr(arg, true)?;
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
                        self.compile_expr(arg, true)?;
                    }
                    self.emit(Opcode::Call, &[args.len() as u32]);
                }

                self.emit_opcode(Opcode::PushUndefined);
                self.emit_opcode(Opcode::Swap);
            }
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
        configurable_globals: bool,
    ) -> JsResult<()> {
        match item {
            StatementListItem::Statement(stmt) => {
                self.compile_stmt(stmt, use_expr, configurable_globals)
            }
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
    pub fn compile_for_loop(
        &mut self,
        for_loop: &ForLoop,
        label: Option<Sym>,
        configurable_globals: bool,
    ) -> JsResult<()> {
        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        if let Some(init) = for_loop.init() {
            match init {
                ForLoopInitializer::Expression(expr) => self.compile_expr(expr, false)?,
                ForLoopInitializer::Var(decl) => {
                    self.create_decls_from_var_decl(decl, configurable_globals);
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

        self.compile_stmt(for_loop.body(), false, configurable_globals)?;

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
        configurable_globals: bool,
    ) -> JsResult<()> {
        let init_bound_names = bound_names(for_in_loop.initializer());
        if init_bound_names.is_empty() {
            self.compile_expr(for_in_loop.target(), true)?;
        } else {
            self.context.push_compile_time_environment(false);
            let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

            for name in init_bound_names {
                self.context.create_mutable_binding(name, false, false);
            }
            self.compile_expr(for_in_loop.target(), true)?;

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

        match for_in_loop.initializer() {
            IterableLoopInitializer::Identifier(ident) => {
                self.context.create_mutable_binding(*ident, true, true);
                let binding = self.context.set_mutable_binding(*ident);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefInitVar, &[index]);
            }
            IterableLoopInitializer::Access(access) => {
                self.access_set(
                    Access::Property { access },
                    false,
                    Self::access_set_top_of_stack_expr_fn,
                )?;
            }
            IterableLoopInitializer::Var(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context
                        .create_mutable_binding(*ident, true, configurable_globals);
                    self.emit_binding(BindingOpcode::InitVar, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.context.create_mutable_binding(ident, true, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
                }
            },
            IterableLoopInitializer::Let(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_mutable_binding(*ident, false, false);
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.context.create_mutable_binding(ident, false, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                }
            },
            IterableLoopInitializer::Const(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_immutable_binding(*ident, true);
                    self.emit_binding(BindingOpcode::InitConst, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.context.create_immutable_binding(ident, true);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitConst)?;
                }
            },
            IterableLoopInitializer::Pattern(pattern) => {
                for ident in bound_names(pattern) {
                    self.context.create_mutable_binding(ident, true, true);
                }
                self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
            }
        }

        self.compile_stmt(for_in_loop.body(), false, configurable_globals)?;

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
        configurable_globals: bool,
    ) -> JsResult<()> {
        let init_bound_names = bound_names(for_of_loop.initializer());
        if init_bound_names.is_empty() {
            self.compile_expr(for_of_loop.iterable(), true)?;
        } else {
            self.context.push_compile_time_environment(false);
            let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

            for name in init_bound_names {
                self.context.create_mutable_binding(name, false, false);
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

        match for_of_loop.initializer() {
            IterableLoopInitializer::Identifier(ref ident) => {
                self.context.create_mutable_binding(*ident, true, true);
                let binding = self.context.set_mutable_binding(*ident);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefInitVar, &[index]);
            }
            IterableLoopInitializer::Access(access) => {
                self.access_set(
                    Access::Property { access },
                    false,
                    Self::access_set_top_of_stack_expr_fn,
                )?;
            }
            IterableLoopInitializer::Var(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_mutable_binding(*ident, true, false);
                    self.emit_binding(BindingOpcode::InitVar, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.context.create_mutable_binding(ident, true, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
                }
            },
            IterableLoopInitializer::Let(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_mutable_binding(*ident, false, false);
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.context.create_mutable_binding(ident, false, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                }
            },
            IterableLoopInitializer::Const(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.context.create_immutable_binding(*ident, true);
                    self.emit_binding(BindingOpcode::InitConst, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.context.create_immutable_binding(ident, true);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitConst)?;
                }
            },
            IterableLoopInitializer::Pattern(pattern) => {
                for ident in bound_names(pattern) {
                    self.context.create_mutable_binding(ident, true, true);
                }
                self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
            }
        }

        self.compile_stmt(for_of_loop.body(), false, configurable_globals)?;

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
        configurable_globals: bool,
    ) -> JsResult<()> {
        self.emit_opcode(Opcode::LoopStart);
        let start_address = self.next_opcode_location();
        self.push_loop_control_info(label, start_address);
        self.emit_opcode(Opcode::LoopContinue);

        self.compile_expr(while_loop.condition(), true)?;
        let exit = self.jump_if_false();
        self.compile_stmt(while_loop.body(), false, configurable_globals)?;
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
        configurable_globals: bool,
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

        self.compile_stmt(do_while_loop.body(), false, configurable_globals)?;
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
        configurable_globals: bool,
    ) -> JsResult<()> {
        if let Some(label) = label {
            let next = self.next_opcode_location();
            self.push_labelled_block_control_info(label, next);
        }

        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
        self.create_decls(block.statement_list(), configurable_globals);
        self.compile_statement_list(block.statement_list(), use_expr, configurable_globals)?;
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
    pub fn compile_stmt(
        &mut self,
        node: &Statement,
        use_expr: bool,
        configurable_globals: bool,
    ) -> JsResult<()> {
        match node {
            Statement::Var(var) => self.compile_var_decl(var)?,
            Statement::If(node) => {
                self.compile_expr(node.cond(), true)?;
                let jelse = self.jump_if_false();

                self.compile_stmt(node.body(), false, configurable_globals)?;

                match node.else_node() {
                    None => {
                        self.patch_jump(jelse);
                    }
                    Some(else_body) => {
                        let exit = self.jump();
                        self.patch_jump(jelse);
                        self.compile_stmt(else_body, false, configurable_globals)?;
                        self.patch_jump(exit);
                    }
                }
            }
            Statement::ForLoop(for_loop) => {
                self.compile_for_loop(for_loop, None, configurable_globals)?;
            }
            Statement::ForInLoop(for_in_loop) => {
                self.compile_for_in_loop(for_in_loop, None, configurable_globals)?;
            }
            Statement::ForOfLoop(for_of_loop) => {
                self.compile_for_of_loop(for_of_loop, None, configurable_globals)?;
            }
            Statement::WhileLoop(while_loop) => {
                self.compile_while_loop(while_loop, None, configurable_globals)?;
            }
            Statement::DoWhileLoop(do_while_loop) => {
                self.compile_do_while_loop(do_while_loop, None, configurable_globals)?;
            }
            Statement::Block(block) => {
                self.compile_block(block, None, use_expr, configurable_globals)?;
            }
            Statement::Labelled(labelled) => match labelled.item() {
                LabelledItem::Statement(stmt) => match stmt {
                    Statement::ForLoop(for_loop) => {
                        self.compile_for_loop(
                            for_loop,
                            Some(labelled.label()),
                            configurable_globals,
                        )?;
                    }
                    Statement::ForInLoop(for_in_loop) => {
                        self.compile_for_in_loop(
                            for_in_loop,
                            Some(labelled.label()),
                            configurable_globals,
                        )?;
                    }
                    Statement::ForOfLoop(for_of_loop) => {
                        self.compile_for_of_loop(
                            for_of_loop,
                            Some(labelled.label()),
                            configurable_globals,
                        )?;
                    }
                    Statement::WhileLoop(while_loop) => {
                        self.compile_while_loop(
                            while_loop,
                            Some(labelled.label()),
                            configurable_globals,
                        )?;
                    }
                    Statement::DoWhileLoop(do_while_loop) => {
                        self.compile_do_while_loop(
                            do_while_loop,
                            Some(labelled.label()),
                            configurable_globals,
                        )?;
                    }
                    Statement::Block(block) => {
                        self.compile_block(
                            block,
                            Some(labelled.label()),
                            use_expr,
                            configurable_globals,
                        )?;
                    }
                    stmt => self.compile_stmt(stmt, use_expr, configurable_globals)?,
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
                        // TODO: promote to an early error.
                        let address = address_info
                            .ok_or_else(|| {
                                JsNativeError::syntax().with_message(format!(
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
                            // TODO: promote to an early error.
                            .ok_or_else(|| {
                                JsNativeError::syntax().with_message("continue must be inside loop")
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
                    // TODO: promote to an early error.
                    if !found {
                        return Err(JsNativeError::syntax()
                            .with_message(format!(
                                "Cannot use the undeclared label '{}'",
                                self.interner().resolve_expect(label_name)
                            ))
                            .into());
                    }
                } else {
                    self.jump_info
                        .last_mut()
                        // TODO: promote to an early error.
                        .ok_or_else(|| {
                            JsNativeError::syntax()
                                .with_message("unlabeled break must be inside loop or switch")
                        })?
                        .breaks
                        .push(label);
                }
            }
            Statement::Throw(throw) => {
                self.compile_expr(throw.target(), true)?;
                self.emit(Opcode::Throw, &[]);
            }
            Statement::Switch(switch) => {
                self.context.push_compile_time_environment(false);
                let push_env =
                    self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
                for case in switch.cases() {
                    self.create_decls(case.body(), configurable_globals);
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
                    self.compile_statement_list(case.body(), false, configurable_globals)?;
                }

                self.patch_jump(exit);
                if let Some(body) = switch.default() {
                    self.create_decls(body, configurable_globals);
                    self.compile_statement_list(body, false, configurable_globals)?;
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
                if let Some(expr) = ret.target() {
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

                self.create_decls(t.block().statement_list(), configurable_globals);
                self.compile_statement_list(
                    t.block().statement_list(),
                    use_expr,
                    configurable_globals,
                )?;

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
                                self.context.create_mutable_binding(*ident, false, false);
                                self.emit_binding(BindingOpcode::InitLet, *ident);
                            }
                            Binding::Pattern(pattern) => {
                                for ident in bound_names(pattern) {
                                    self.context.create_mutable_binding(ident, false, false);
                                }
                                self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                            }
                        }
                    } else {
                        self.emit_opcode(Opcode::Pop);
                    }

                    self.create_decls(catch.block().statement_list(), configurable_globals);
                    self.compile_statement_list(
                        catch.block().statement_list(),
                        use_expr,
                        configurable_globals,
                    )?;

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

                    self.create_decls(finally.block().statement_list(), configurable_globals);
                    self.compile_statement_list(
                        finally.block().statement_list(),
                        false,
                        configurable_globals,
                    )?;

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
            has_binding_identifier,
            ..
        } = function;

        let code = FunctionCompiler::new()
            .name(name.map(Identifier::sym))
            .generator(generator)
            .r#async(r#async)
            .strict(self.code_block.strict)
            .arrow(arrow)
            .has_binding_identifier(has_binding_identifier)
            .compile(parameters, body, self.context)?;

        let index = self.code_block.functions.len() as u32;
        self.code_block.functions.push(code);

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
            Callable::Call(call) => match call.function() {
                Expression::Identifier(ident) if *ident == Sym::EVAL => (call, CallKind::CallEval),
                _ => (call, CallKind::Call),
            },
            Callable::New(new) => (new.call(), CallKind::New),
        };

        match call.function() {
            Expression::PropertyAccess(access) if kind == CallKind::Call => {
                self.compile_access_preserve_this(access)?;
            }

            Expression::Optional(opt) if kind == CallKind::Call => {
                self.compile_optional_preserve_this(opt)?;
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
                    use ObjectPatternElement::{
                        AssignmentPropertyAccess, AssignmentRestPropertyAccess, Pattern,
                        RestProperty, SingleName,
                    };

                    match binding {
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
                            self.access_set(
                                Access::Property { access },
                                false,
                                Self::access_set_top_of_stack_expr_fn,
                            )?;
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

                            self.access_set(
                                Access::Property { access },
                                false,
                                Self::access_set_top_of_stack_expr_fn,
                            )?;

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
                    use ArrayPatternElement::{
                        Elision, Pattern, PatternRest, PropertyAccess, PropertyAccessRest,
                        SingleName, SingleNameRest,
                    };

                    match binding {
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
                            self.access_set(
                                Access::Property { access },
                                false,
                                Self::access_set_top_of_stack_expr_fn,
                            )?;
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
                            self.access_set(
                                Access::Property { access },
                                false,
                                Self::access_set_top_of_stack_expr_fn,
                            )?;
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

    pub(crate) fn create_decls(&mut self, stmt_list: &StatementList, configurable_globals: bool) {
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

    #[inline]
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

    #[inline]
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

    #[inline]
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
                .create_mutable_binding(Sym::ARGUMENTS.into(), false, false);
            compiler.code_block.arguments_binding = Some(
                compiler
                    .context
                    .initialize_mutable_binding(Sym::ARGUMENTS.into(), false),
            );
            for parameter in expr.parameters().as_ref() {
                if parameter.is_rest_param() {
                    compiler.emit_opcode(Opcode::RestParameterInit);
                }

                match parameter.variable().binding() {
                    Binding::Identifier(ident) => {
                        compiler
                            .context
                            .create_mutable_binding(*ident, false, false);
                        if let Some(init) = parameter.variable().init() {
                            let skip =
                                compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                            compiler.compile_expr(init, true)?;
                            compiler.patch_jump(skip);
                        }
                        compiler.emit_binding(BindingOpcode::InitArg, *ident);
                    }
                    Binding::Pattern(pattern) => {
                        for ident in bound_names(pattern) {
                            compiler.context.create_mutable_binding(ident, false, false);
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
            compiler.create_decls(expr.body(), false);
            compiler.compile_statement_list(expr.body(), false, false)?;
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
                compiler.push_compile_environment(compile_environment);
                compiler.code_block.num_bindings = num_bindings;
                compiler.code_block.is_class_constructor = true;
            }
        } else {
            if class.super_ref().is_some() {
                compiler.emit_opcode(Opcode::SuperCallDerived);
            }
            let (num_bindings, compile_environment) =
                compiler.context.pop_compile_time_environment();
            compiler.push_compile_environment(compile_environment);
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

        // TODO: set function name for getter and setters
        for element in class.elements() {
            match element {
                ClassElement::StaticMethodDefinition(name, method_definition) => {
                    self.emit_opcode(Opcode::Dup);
                    match method_definition {
                        MethodDefinition::Get(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
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
                // TODO: set names for private methods
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
                    field_compiler.push_compile_environment(compile_environment);
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
                    field_compiler.push_compile_environment(compile_environment);
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
                    compiler.create_decls(statement_list, false);
                    compiler.compile_statement_list(statement_list, false, false)?;
                    let (num_bindings, compile_environment) =
                        compiler.context.pop_compile_time_environment();
                    compiler.push_compile_environment(compile_environment);
                    compiler.code_block.num_bindings = num_bindings;

                    let code = Gc::new(compiler.finish());
                    let index = self.code_block.functions.len() as u32;
                    self.code_block.functions.push(code);
                    self.emit(Opcode::GetFunction, &[index]);
                    self.emit_opcode(Opcode::SetHomeObject);
                    self.emit(Opcode::Call, &[0]);
                    self.emit_opcode(Opcode::Pop);
                }
                // TODO: set names for private methods
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
                    // TODO: set names for getters and setters
                    match method_definition {
                        MethodDefinition::Get(expr) => match name {
                            PropertyName::Literal(name) => {
                                self.function(expr.into(), NodeKind::Expression, true)?;
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
