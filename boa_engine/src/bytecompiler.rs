use crate::{
    builtins::function::ThisMode,
    environments::BindingLocator,
    syntax::ast::{
        node::{
            declaration::{BindingPatternTypeArray, BindingPatternTypeObject, DeclarationPattern},
            iteration::IterableLoopInitializer,
            object::{MethodDefinition, PropertyDefinition, PropertyName},
            template::TemplateElement,
            Declaration, GetConstField, GetField, StatementList,
        },
        op::{AssignOp, BinOp, BitOp, CompOp, LogOp, NumOp, UnaryOp},
        Const, Node,
    },
    vm::{BindingOpcode, CodeBlock, Opcode},
    Context, JsBigInt, JsResult, JsString, JsValue,
};
use boa_gc::Gc;
use boa_interner::{Interner, Sym};
use rustc_hash::FxHashMap;
use std::mem::size_of;

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
}

#[derive(Debug, Clone, Copy)]
enum Access<'a> {
    Variable { name: Sym },
    ByName { node: &'a GetConstField },
    ByValue { node: &'a GetField },
    This,
}

#[derive(Debug)]
pub struct ByteCompiler<'b> {
    code_block: CodeBlock,
    literals_map: FxHashMap<Literal, u32>,
    names_map: FxHashMap<Sym, u32>,
    bindings_map: FxHashMap<BindingLocator, u32>,
    jump_info: Vec<JumpControlInfo>,
    context: &'b mut Context,
}

impl<'b> ByteCompiler<'b> {
    /// Represents a placeholder address that will be patched later.
    const DUMMY_ADDRESS: u32 = u32::MAX;

    #[inline]
    pub fn new(name: Sym, strict: bool, context: &'b mut Context) -> Self {
        Self {
            code_block: CodeBlock::new(name, 0, strict, false),
            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            bindings_map: FxHashMap::default(),
            jump_info: Vec::new(),
            context,
        }
    }

    #[inline]
    fn interner(&self) -> &Interner {
        self.context.interner()
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
    fn get_or_insert_name(&mut self, name: Sym) -> u32 {
        if let Some(index) = self.names_map.get(&name) {
            return *index;
        }

        let index = self.code_block.variables.len() as u32;
        self.code_block.variables.push(name);
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
    fn emit_binding(&mut self, opcode: BindingOpcode, name: Sym) {
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

    #[inline]
    fn jump_with_custom_opcode(&mut self, opcode: Opcode) -> Label {
        let index = self.next_opcode_location();
        self.emit(opcode, &[Self::DUMMY_ADDRESS]);
        Label { index }
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
        let loop_info = self.jump_info.pop().expect("no jump informatiojn found");

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
    fn compile_access(node: &Node) -> Option<Access<'_>> {
        match node {
            Node::Identifier(name) => Some(Access::Variable { name: name.sym() }),
            Node::GetConstField(node) => Some(Access::ByName { node }),
            Node::GetField(node) => Some(Access::ByValue { node }),
            Node::This => Some(Access::This),
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
            Access::ByName { node } => {
                let index = self.get_or_insert_name(node.field());
                self.compile_expr(node.obj(), true)?;
                self.emit(Opcode::GetPropertyByName, &[index]);
            }
            Access::ByValue { node } => {
                self.compile_expr(node.field(), true)?;
                self.compile_expr(node.obj(), true)?;
                self.emit(Opcode::GetPropertyByValue, &[]);
            }
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
        expr: Option<&Node>,
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
            Access::ByName { node } => {
                self.compile_expr(node.obj(), true)?;
                let index = self.get_or_insert_name(node.field());
                self.emit(Opcode::SetPropertyByName, &[index]);
            }
            Access::ByValue { node } => {
                self.compile_expr(node.field(), true)?;
                self.compile_expr(node.obj(), true)?;
                self.emit(Opcode::SetPropertyByValue, &[]);
            }
            Access::This => todo!("access_set 'this'"),
        }
        Ok(())
    }

    #[inline]
    pub fn compile_statement_list(&mut self, list: &StatementList, use_expr: bool) -> JsResult<()> {
        for (i, node) in list.items().iter().enumerate() {
            if i + 1 == list.items().len() {
                self.compile_stmt(node, use_expr)?;
                break;
            }

            self.compile_stmt(node, false)?;
        }
        Ok(())
    }

    #[inline]
    pub fn compile_expr(&mut self, expr: &Node, use_expr: bool) -> JsResult<()> {
        match expr {
            Node::Const(c) => {
                match c {
                    Const::String(v) => self.emit_push_literal(Literal::String(
                        self.interner().resolve_expect(*v).into(),
                    )),
                    Const::Int(v) => self.emit_push_integer(*v),
                    Const::Num(v) => self.emit_push_rational(*v),
                    Const::BigInt(v) => self.emit_push_literal(Literal::BigInt(v.clone().into())),
                    Const::Bool(true) => self.emit(Opcode::PushTrue, &[]),
                    Const::Bool(false) => self.emit(Opcode::PushFalse, &[]),
                    Const::Null => self.emit(Opcode::PushNull, &[]),
                    Const::Undefined => self.emit(Opcode::PushUndefined, &[]),
                }

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Node::UnaryOp(unary) => {
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
                        self.emit(Opcode::Dup, &[]);
                        self.emit(Opcode::Inc, &[]);

                        let access = Self::compile_access(unary.target()).ok_or_else(|| {
                            self.context
                                .construct_syntax_error("Invalid increment operand")
                        })?;
                        self.access_set(access, None, false)?;

                        None
                    }
                    UnaryOp::DecrementPost => {
                        self.compile_expr(unary.target(), true)?;
                        self.emit(Opcode::Dup, &[]);
                        self.emit(Opcode::Dec, &[]);

                        let access = Self::compile_access(unary.target()).ok_or_else(|| {
                            self.context
                                .construct_syntax_error("Invalid decrement operand")
                        })?;
                        self.access_set(access, None, false)?;

                        None
                    }
                    UnaryOp::Delete => match unary.target() {
                        Node::GetConstField(ref get_const_field) => {
                            let index = self.get_or_insert_name(get_const_field.field());
                            self.compile_expr(get_const_field.obj(), true)?;
                            self.emit(Opcode::DeletePropertyByName, &[index]);
                            None
                        }
                        Node::GetField(ref get_field) => {
                            self.compile_expr(get_field.field(), true)?;
                            self.compile_expr(get_field.obj(), true)?;
                            self.emit(Opcode::DeletePropertyByValue, &[]);
                            None
                        }
                        // TODO: implement delete on references.
                        Node::Identifier(_) => {
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
                            Node::Identifier(identifier) => {
                                let binding = self.context.get_binding_value(identifier.sym());
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
            Node::BinOp(binary) => {
                self.compile_expr(binary.lhs(), true)?;
                match binary.op() {
                    BinOp::Num(op) => {
                        self.compile_expr(binary.rhs(), true)?;
                        match op {
                            NumOp::Add => self.emit_opcode(Opcode::Add),
                            NumOp::Sub => self.emit_opcode(Opcode::Sub),
                            NumOp::Div => self.emit_opcode(Opcode::Div),
                            NumOp::Mul => self.emit_opcode(Opcode::Mul),
                            NumOp::Exp => self.emit_opcode(Opcode::Pow),
                            NumOp::Mod => self.emit_opcode(Opcode::Mod),
                        }

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                    BinOp::Bit(op) => {
                        self.compile_expr(binary.rhs(), true)?;
                        match op {
                            BitOp::And => self.emit_opcode(Opcode::BitAnd),
                            BitOp::Or => self.emit_opcode(Opcode::BitOr),
                            BitOp::Xor => self.emit_opcode(Opcode::BitXor),
                            BitOp::Shl => self.emit_opcode(Opcode::ShiftLeft),
                            BitOp::Shr => self.emit_opcode(Opcode::ShiftRight),
                            BitOp::UShr => self.emit_opcode(Opcode::UnsignedShiftRight),
                        }

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                    BinOp::Comp(op) => {
                        self.compile_expr(binary.rhs(), true)?;
                        match op {
                            CompOp::Equal => self.emit_opcode(Opcode::Eq),
                            CompOp::NotEqual => self.emit_opcode(Opcode::NotEq),
                            CompOp::StrictEqual => self.emit_opcode(Opcode::StrictEq),
                            CompOp::StrictNotEqual => self.emit_opcode(Opcode::StrictNotEq),
                            CompOp::GreaterThan => self.emit_opcode(Opcode::GreaterThan),
                            CompOp::GreaterThanOrEqual => self.emit_opcode(Opcode::GreaterThanOrEq),
                            CompOp::LessThan => self.emit_opcode(Opcode::LessThan),
                            CompOp::LessThanOrEqual => self.emit_opcode(Opcode::LessThanOrEq),
                            CompOp::In => self.emit_opcode(Opcode::In),
                            CompOp::InstanceOf => self.emit_opcode(Opcode::InstanceOf),
                        }

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                    BinOp::Log(op) => {
                        match op {
                            LogOp::And => {
                                let exit = self.jump_with_custom_opcode(Opcode::LogicalAnd);
                                self.compile_expr(binary.rhs(), true)?;
                                self.patch_jump(exit);
                            }
                            LogOp::Or => {
                                let exit = self.jump_with_custom_opcode(Opcode::LogicalOr);
                                self.compile_expr(binary.rhs(), true)?;
                                self.patch_jump(exit);
                            }
                            LogOp::Coalesce => {
                                let exit = self.jump_with_custom_opcode(Opcode::Coalesce);
                                self.compile_expr(binary.rhs(), true)?;
                                self.patch_jump(exit);
                            }
                        };

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                    BinOp::Assign(op) => {
                        let opcode = match op {
                            AssignOp::Add => Some(Opcode::Add),
                            AssignOp::Sub => Some(Opcode::Sub),
                            AssignOp::Mul => Some(Opcode::Mul),
                            AssignOp::Div => Some(Opcode::Div),
                            AssignOp::Mod => Some(Opcode::Mod),
                            AssignOp::Exp => Some(Opcode::Pow),
                            AssignOp::And => Some(Opcode::BitAnd),
                            AssignOp::Or => Some(Opcode::BitOr),
                            AssignOp::Xor => Some(Opcode::BitXor),
                            AssignOp::Shl => Some(Opcode::ShiftLeft),
                            AssignOp::Shr => Some(Opcode::ShiftRight),
                            AssignOp::Ushr => Some(Opcode::UnsignedShiftRight),
                            AssignOp::BoolAnd => {
                                let exit = self.jump_with_custom_opcode(Opcode::LogicalAnd);
                                self.compile_expr(binary.rhs(), true)?;
                                let access =
                                    Self::compile_access(binary.lhs()).ok_or_else(|| {
                                        self.context.construct_syntax_error(
                                            "Invalid left-hand side in assignment",
                                        )
                                    })?;
                                self.access_set(access, None, use_expr)?;
                                self.patch_jump(exit);
                                None
                            }
                            AssignOp::BoolOr => {
                                let exit = self.jump_with_custom_opcode(Opcode::LogicalOr);
                                self.compile_expr(binary.rhs(), true)?;
                                let access =
                                    Self::compile_access(binary.lhs()).ok_or_else(|| {
                                        self.context.construct_syntax_error(
                                            "Invalid left-hand side in assignment",
                                        )
                                    })?;
                                self.access_set(access, None, use_expr)?;
                                self.patch_jump(exit);
                                None
                            }
                            AssignOp::Coalesce => {
                                let exit = self.jump_with_custom_opcode(Opcode::Coalesce);
                                self.compile_expr(binary.rhs(), true)?;
                                let access =
                                    Self::compile_access(binary.lhs()).ok_or_else(|| {
                                        self.context.construct_syntax_error(
                                            "Invalid left-hand side in assignment",
                                        )
                                    })?;
                                self.access_set(access, None, use_expr)?;
                                self.patch_jump(exit);
                                None
                            }
                        };

                        if let Some(opcode) = opcode {
                            self.compile_expr(binary.rhs(), true)?;
                            self.emit(opcode, &[]);
                            let access = Self::compile_access(binary.lhs()).ok_or_else(|| {
                                self.context
                                    .construct_syntax_error("Invalid left-hand side in assignment")
                            })?;
                            self.access_set(access, None, use_expr)?;
                        }
                    }
                    BinOp::Comma => {
                        self.emit(Opcode::Pop, &[]);
                        self.compile_expr(binary.rhs(), true)?;

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                }
            }
            Node::Object(object) => {
                self.emit_opcode(Opcode::PushEmptyObject);
                for property in object.properties() {
                    self.emit_opcode(Opcode::Dup);
                    match property {
                        PropertyDefinition::IdentifierReference(ident) => {
                            let index = self.get_or_insert_name(*ident);
                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                        }
                        PropertyDefinition::Property(name, node) => match name {
                            PropertyName::Literal(name) => {
                                self.compile_stmt(node, true)?;
                                self.emit_opcode(Opcode::Swap);
                                let index = self.get_or_insert_name(*name);
                                self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_stmt(name_node, true)?;
                                self.compile_stmt(node, true)?;
                                self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                            }
                        },
                        PropertyDefinition::MethodDefinition(kind, name) => {
                            match kind {
                                MethodDefinition::Get(expr) => match name {
                                    PropertyName::Literal(name) => {
                                        self.function(&expr.clone().into(), true)?;
                                        self.emit_opcode(Opcode::Swap);
                                        let index = self.get_or_insert_name(*name);
                                        self.emit(Opcode::SetPropertyGetterByName, &[index]);
                                    }
                                    PropertyName::Computed(name_node) => {
                                        self.compile_stmt(name_node, true)?;
                                        self.function(&expr.clone().into(), true)?;
                                        self.emit_opcode(Opcode::SetPropertyGetterByValue);
                                    }
                                },
                                MethodDefinition::Set(expr) => match name {
                                    PropertyName::Literal(name) => {
                                        self.function(&expr.clone().into(), true)?;
                                        self.emit_opcode(Opcode::Swap);
                                        let index = self.get_or_insert_name(*name);
                                        self.emit(Opcode::SetPropertySetterByName, &[index]);
                                    }
                                    PropertyName::Computed(name_node) => {
                                        self.compile_stmt(name_node, true)?;
                                        self.function(&expr.clone().into(), true)?;
                                        self.emit_opcode(Opcode::SetPropertySetterByValue);
                                    }
                                },
                                MethodDefinition::Ordinary(expr) => match name {
                                    PropertyName::Literal(name) => {
                                        self.function(&expr.clone().into(), true)?;
                                        self.emit_opcode(Opcode::Swap);
                                        let index = self.get_or_insert_name(*name);
                                        self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                                    }
                                    PropertyName::Computed(name_node) => {
                                        self.compile_stmt(name_node, true)?;
                                        self.function(&expr.clone().into(), true)?;
                                        self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                    }
                                },
                                MethodDefinition::Generator(expr) => match name {
                                    PropertyName::Literal(name) => {
                                        self.function(&expr.clone().into(), true)?;
                                        self.emit_opcode(Opcode::Swap);
                                        let index = self.get_or_insert_name(*name);
                                        self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                                    }
                                    PropertyName::Computed(name_node) => {
                                        self.compile_stmt(name_node, true)?;
                                        self.function(&expr.clone().into(), true)?;
                                        self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                    }
                                },
                                // TODO: Implement async
                                // TODO: Implement async generators
                                MethodDefinition::Async(_)
                                | MethodDefinition::AsyncGenerator(_) => {
                                    // TODO: Implement async
                                    match name {
                                        PropertyName::Literal(name) => {
                                            self.emit_opcode(Opcode::PushUndefined);
                                            self.emit_opcode(Opcode::Swap);
                                            let index = self.get_or_insert_name(*name);
                                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                                        }
                                        PropertyName::Computed(name_node) => {
                                            self.compile_stmt(name_node, true)?;
                                            self.emit_opcode(Opcode::PushUndefined);
                                            self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                        }
                                    }
                                }
                            }
                        }
                        PropertyDefinition::SpreadObject(expr) => {
                            self.compile_expr(expr, true)?;
                            self.emit_opcode(Opcode::Swap);
                            self.emit(Opcode::CopyDataProperties, &[0]);
                            self.emit_opcode(Opcode::Pop);
                        }
                    }
                }

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Node::Identifier(name) => {
                let access = Access::Variable { name: name.sym() };
                self.access_get(access, use_expr)?;
            }
            Node::Assign(assign) => {
                // Implement destructing assignments like here: https://tc39.es/ecma262/#sec-destructuring-assignment
                if let Node::Object(_) = assign.lhs() {
                    self.emit_opcode(Opcode::PushUndefined);
                } else {
                    let access = Self::compile_access(assign.lhs()).ok_or_else(|| {
                        self.context
                            .construct_syntax_error("Invalid left-hand side in assignment")
                    })?;
                    self.access_set(access, Some(assign.rhs()), use_expr)?;
                }
            }
            Node::GetConstField(node) => {
                let access = Access::ByName { node };
                self.access_get(access, use_expr)?;
            }
            Node::GetField(node) => {
                let access = Access::ByValue { node };
                self.access_get(access, use_expr)?;
            }
            Node::ConditionalOp(op) => {
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
            Node::ArrayDecl(array) => {
                self.emit_opcode(Opcode::PushNewArray);
                self.emit_opcode(Opcode::PopOnReturnAdd);

                for element in array.as_ref() {
                    if let Node::Empty = element {
                        self.emit_opcode(Opcode::PushElisionToArray);
                        continue;
                    }

                    self.compile_expr(element, true)?;
                    if let Node::Spread(_) = element {
                        self.emit_opcode(Opcode::InitIterator);
                        self.emit_opcode(Opcode::PushIteratorToArray);
                    } else {
                        self.emit_opcode(Opcode::PushValueToArray);
                    }
                }

                self.emit_opcode(Opcode::PopOnReturnSub);
                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Node::This => {
                self.access_get(Access::This, use_expr)?;
            }
            Node::Spread(spread) => self.compile_expr(spread.val(), true)?,
            Node::FunctionExpr(_function) => self.function(expr, use_expr)?,
            Node::ArrowFunctionDecl(_function) => self.function(expr, use_expr)?,
            Node::Call(_) | Node::New(_) => self.call(expr, use_expr)?,
            Node::TemplateLit(template_literal) => {
                for element in template_literal.elements() {
                    match element {
                        TemplateElement::String(s) => self.emit_push_literal(Literal::String(
                            self.interner().resolve_expect(*s).into(),
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
            // TODO: implement AsyncFunctionExpr
            // TODO: implement AwaitExpr
            // TODO: implement AsyncGeneratorExpr
            Node::AsyncFunctionExpr(_) | Node::AwaitExpr(_) | Node::AsyncGeneratorExpr(_) => {
                self.emit_opcode(Opcode::PushUndefined);
            }
            Node::GeneratorExpr(_) => self.function(expr, use_expr)?,
            Node::Yield(r#yield) => {
                if let Some(expr) = r#yield.expr() {
                    self.compile_expr(expr, true)?;
                } else {
                    self.emit_opcode(Opcode::PushUndefined);
                }

                if r#yield.delegate() {
                    self.emit_opcode(Opcode::InitIterator);
                    self.emit_opcode(Opcode::PushUndefined);
                    let start_address = self.next_opcode_location();
                    let start = self.jump_with_custom_opcode(Opcode::GeneratorNextDelegate);
                    self.emit(Opcode::Jump, &[start_address]);
                    self.patch_jump(start);
                } else {
                    self.emit_opcode(Opcode::Yield);
                    self.emit_opcode(Opcode::GeneratorNext);
                }

                if !use_expr {
                    self.emit_opcode(Opcode::Pop);
                }
            }
            Node::TaggedTemplate(template) => {
                match template.tag() {
                    Node::GetConstField(field) => {
                        self.compile_expr(field.obj(), true)?;
                        self.emit(Opcode::Dup, &[]);
                        let index = self.get_or_insert_name(field.field());
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    Node::GetField(field) => {
                        self.compile_expr(field.obj(), true)?;
                        self.emit(Opcode::Dup, &[]);
                        self.compile_expr(field.field(), true)?;
                        self.emit(Opcode::Swap, &[]);
                        self.emit(Opcode::GetPropertyByValue, &[]);
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
                            self.interner().resolve_expect(*cooked).into(),
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
                        self.interner().resolve_expect(*raw).into(),
                    ));
                    self.emit_opcode(Opcode::PushValueToArray);
                }

                self.emit_opcode(Opcode::Swap);
                let index = self.get_or_insert_name(Sym::RAW);
                self.emit(Opcode::SetPropertyByName, &[index]);

                for expr in template.exprs() {
                    self.compile_expr(expr, true)?;
                }

                self.emit(Opcode::Call, &[(template.exprs().len() + 1) as u32]);
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    #[inline]
    pub fn compile_stmt(&mut self, node: &Node, use_expr: bool) -> JsResult<()> {
        match node {
            Node::VarDeclList(list) => {
                for decl in list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            let ident = ident.sym();
                            if ident == Sym::ARGUMENTS {
                                self.code_block.lexical_name_argument = true;
                            }

                            if let Some(expr) = decl.init() {
                                self.compile_expr(expr, true)?;
                                self.emit_binding(BindingOpcode::InitVar, ident);
                            } else {
                                self.emit_binding(BindingOpcode::Var, ident);
                            }
                        }
                        Declaration::Pattern(pattern) => {
                            if pattern.idents().contains(&Sym::ARGUMENTS) {
                                self.code_block.lexical_name_argument = true;
                            }

                            if let Some(init) = decl.init() {
                                self.compile_expr(init, true)?;
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
                        }
                    }
                }
            }
            Node::LetDeclList(list) => {
                for decl in list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            if ident.sym() == Sym::ARGUMENTS {
                                self.code_block.lexical_name_argument = true;
                            }

                            if let Some(expr) = decl.init() {
                                self.compile_expr(expr, true)?;
                                self.emit_binding(BindingOpcode::InitLet, ident.sym());
                            } else {
                                self.emit_binding(BindingOpcode::Let, ident.sym());
                            }
                        }
                        Declaration::Pattern(pattern) => {
                            if pattern.idents().contains(&Sym::ARGUMENTS) {
                                self.code_block.lexical_name_argument = true;
                            }

                            if let Some(init) = decl.init() {
                                self.compile_expr(init, true)?;
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                        }
                    }
                }
            }
            Node::ConstDeclList(list) => {
                for decl in list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            if ident.sym() == Sym::ARGUMENTS {
                                self.code_block.lexical_name_argument = true;
                            }
                            let init = decl
                                .init()
                                .expect("const declaration must have initializer");
                            self.compile_expr(init, true)?;
                            self.emit_binding(BindingOpcode::InitConst, ident.sym());
                        }
                        Declaration::Pattern(pattern) => {
                            if pattern.idents().contains(&Sym::ARGUMENTS) {
                                self.code_block.lexical_name_argument = true;
                            }

                            if let Some(init) = decl.init() {
                                self.compile_expr(init, true)?;
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, BindingOpcode::InitConst)?;
                        }
                    }
                }
            }
            Node::If(node) => {
                self.compile_expr(node.cond(), true)?;
                let jelse = self.jump_if_false();

                if !matches!(node.body(), Node::Block(_)) {
                    self.create_declarations(node.body())?;
                }

                self.compile_stmt(node.body(), false)?;

                match node.else_node() {
                    None => {
                        self.patch_jump(jelse);
                    }
                    Some(else_body) => {
                        let exit = self.jump();
                        self.patch_jump(jelse);
                        if !matches!(else_body, Node::Block(_)) {
                            self.create_declarations(else_body)?;
                        }
                        self.compile_stmt(else_body, false)?;
                        self.patch_jump(exit);
                    }
                }
            }
            Node::ForLoop(for_loop) => {
                self.context.push_compile_time_environment(false);
                let push_env = self.jump_with_custom_opcode(Opcode::PushDeclarativeEnvironment);

                if let Some(init) = for_loop.init() {
                    self.create_declarations(init)?;
                    self.compile_stmt(init, false)?;
                }

                self.emit_opcode(Opcode::LoopStart);
                let initial_jump = self.jump();

                let start_address = self.next_opcode_location();
                self.push_loop_control_info(for_loop.label(), start_address);

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

                if !matches!(for_loop.body(), Node::Block(_)) {
                    self.create_declarations(for_loop.body())?;
                }
                self.compile_stmt(for_loop.body(), false)?;

                self.emit(Opcode::Jump, &[start_address]);

                self.patch_jump(exit);
                self.pop_loop_control_info();
                self.emit_opcode(Opcode::LoopEnd);

                let num_bindings = self.context.pop_compile_time_environment().num_bindings();
                self.patch_jump_with_target(push_env, num_bindings as u32);
                self.emit_opcode(Opcode::PopEnvironment);
            }
            Node::ForInLoop(for_in_loop) => {
                self.compile_expr(for_in_loop.expr(), true)?;
                let early_exit = self.jump_with_custom_opcode(Opcode::ForInLoopInitIterator);

                self.emit_opcode(Opcode::LoopStart);
                let start_address = self.next_opcode_location();
                self.push_loop_control_info_for_of_in_loop(for_in_loop.label(), start_address);
                self.emit_opcode(Opcode::LoopContinue);

                self.context.push_compile_time_environment(false);
                let push_env = self.jump_with_custom_opcode(Opcode::PushDeclarativeEnvironment);
                let exit = self.jump_with_custom_opcode(Opcode::ForInLoopNext);

                match for_in_loop.init() {
                    IterableLoopInitializer::Identifier(ref ident) => {
                        self.context
                            .create_mutable_binding(ident.sym(), true, true)?;
                        let binding = self.context.set_mutable_binding(ident.sym());
                        let index = self.get_or_insert_binding(binding);
                        self.emit(Opcode::DefInitVar, &[index]);
                    }
                    IterableLoopInitializer::Var(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            self.context
                                .create_mutable_binding(ident.sym(), true, true)?;
                            self.emit_binding(BindingOpcode::InitVar, ident.sym());
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                self.context.create_mutable_binding(ident, true, true)?;
                            }
                            self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
                        }
                    },
                    IterableLoopInitializer::Let(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            self.context
                                .create_mutable_binding(ident.sym(), false, false)?;
                            self.emit_binding(BindingOpcode::InitLet, ident.sym());
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                self.context.create_mutable_binding(ident, false, false)?;
                            }
                            self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                        }
                    },
                    IterableLoopInitializer::Const(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            self.context.create_immutable_binding(ident.sym())?;
                            self.emit_binding(BindingOpcode::InitConst, ident.sym());
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                self.context.create_immutable_binding(ident)?;
                            }
                            self.compile_declaration_pattern(pattern, BindingOpcode::InitConst)?;
                        }
                    },
                }

                self.compile_stmt(for_in_loop.body(), false)?;

                let num_bindings = self.context.pop_compile_time_environment().num_bindings();
                self.patch_jump_with_target(push_env, num_bindings as u32);
                self.emit_opcode(Opcode::PopEnvironment);

                self.emit(Opcode::Jump, &[start_address]);

                self.patch_jump(exit);
                self.pop_loop_control_info();
                self.emit_opcode(Opcode::LoopEnd);
                self.emit_opcode(Opcode::PushFalse);
                self.emit_opcode(Opcode::IteratorClose);

                self.patch_jump(early_exit);
            }
            Node::ForOfLoop(for_of_loop) => {
                self.compile_expr(for_of_loop.iterable(), true)?;
                self.emit_opcode(Opcode::InitIterator);

                self.emit_opcode(Opcode::LoopStart);
                let start_address = self.next_opcode_location();
                self.push_loop_control_info_for_of_in_loop(for_of_loop.label(), start_address);
                self.emit_opcode(Opcode::LoopContinue);

                self.context.push_compile_time_environment(false);
                let push_env = self.jump_with_custom_opcode(Opcode::PushDeclarativeEnvironment);
                let exit = self.jump_with_custom_opcode(Opcode::ForInLoopNext);

                match for_of_loop.init() {
                    IterableLoopInitializer::Identifier(ref ident) => {
                        self.context
                            .create_mutable_binding(ident.sym(), true, true)?;
                        let binding = self.context.set_mutable_binding(ident.sym());
                        let index = self.get_or_insert_binding(binding);
                        self.emit(Opcode::DefInitVar, &[index]);
                    }
                    IterableLoopInitializer::Var(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            self.context
                                .create_mutable_binding(ident.sym(), true, true)?;
                            self.emit_binding(BindingOpcode::InitVar, ident.sym());
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                self.context.create_mutable_binding(ident, true, true)?;
                            }
                            self.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
                        }
                    },
                    IterableLoopInitializer::Let(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            self.context
                                .create_mutable_binding(ident.sym(), false, false)?;
                            self.emit_binding(BindingOpcode::InitLet, ident.sym());
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                self.context.create_mutable_binding(ident, false, false)?;
                            }
                            self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                        }
                    },
                    IterableLoopInitializer::Const(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            self.context.create_immutable_binding(ident.sym())?;
                            self.emit_binding(BindingOpcode::InitConst, ident.sym());
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                self.context.create_immutable_binding(ident)?;
                            }
                            self.compile_declaration_pattern(pattern, BindingOpcode::InitConst)?;
                        }
                    },
                }

                self.compile_stmt(for_of_loop.body(), false)?;

                let num_bindings = self.context.pop_compile_time_environment().num_bindings();
                self.patch_jump_with_target(push_env, num_bindings as u32);
                self.emit_opcode(Opcode::PopEnvironment);

                self.emit(Opcode::Jump, &[start_address]);

                self.patch_jump(exit);
                self.pop_loop_control_info();
                self.emit_opcode(Opcode::LoopEnd);
                self.emit_opcode(Opcode::PushFalse);
                self.emit_opcode(Opcode::IteratorClose);
            }
            Node::WhileLoop(while_) => {
                self.emit_opcode(Opcode::LoopStart);
                let start_address = self.next_opcode_location();
                self.push_loop_control_info(while_.label(), start_address);
                self.emit_opcode(Opcode::LoopContinue);

                self.compile_expr(while_.cond(), true)?;
                let exit = self.jump_if_false();
                self.compile_stmt(while_.body(), false)?;
                self.emit(Opcode::Jump, &[start_address]);
                self.patch_jump(exit);

                self.pop_loop_control_info();
                self.emit_opcode(Opcode::LoopEnd);
            }
            Node::DoWhileLoop(do_while) => {
                self.emit_opcode(Opcode::LoopStart);
                let initial_label = self.jump();

                let start_address = self.next_opcode_location();
                self.push_loop_control_info(do_while.label(), start_address);
                self.emit_opcode(Opcode::LoopContinue);

                let condition_label_address = self.next_opcode_location();
                self.compile_expr(do_while.cond(), true)?;
                let exit = self.jump_if_false();

                self.patch_jump(initial_label);

                self.compile_stmt(do_while.body(), false)?;
                self.emit(Opcode::Jump, &[condition_label_address]);

                self.pop_loop_control_info();
                self.emit_opcode(Opcode::LoopEnd);

                self.patch_jump(exit);
            }
            Node::Continue(node) => {
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

                    if in_finally || (!info.has_finally && info.in_catch) {
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
            Node::Break(node) => {
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

                    if in_finally || (!info.has_finally && info.in_catch) {
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
                        .expect("no jump information found")
                        .breaks
                        .push(label);
                }
            }
            Node::Block(block) => {
                self.context.push_compile_time_environment(false);
                let push_env = self.jump_with_custom_opcode(Opcode::PushDeclarativeEnvironment);
                for node in block.items() {
                    self.create_declarations(node)?;
                }
                for node in block.items() {
                    self.compile_stmt(node, use_expr)?;
                }
                let num_bindings = self.context.pop_compile_time_environment().num_bindings();
                self.patch_jump_with_target(push_env, num_bindings as u32);
                self.emit_opcode(Opcode::PopEnvironment);
            }
            Node::Throw(throw) => {
                self.compile_expr(throw.expr(), true)?;
                self.emit(Opcode::Throw, &[]);
            }
            Node::Switch(switch) => {
                self.context.push_compile_time_environment(false);
                let push_env = self.jump_with_custom_opcode(Opcode::PushDeclarativeEnvironment);
                for case in switch.cases() {
                    for node in case.body().items() {
                        self.create_declarations(node)?;
                    }
                }
                self.emit_opcode(Opcode::LoopStart);

                let start_address = self.next_opcode_location();
                self.push_switch_control_info(None, start_address);

                self.compile_expr(switch.val(), true)?;
                let mut labels = Vec::with_capacity(switch.cases().len());
                for case in switch.cases() {
                    self.compile_expr(case.condition(), true)?;
                    labels.push(self.jump_with_custom_opcode(Opcode::Case));
                }

                let exit = self.jump_with_custom_opcode(Opcode::Default);

                for (label, case) in labels.into_iter().zip(switch.cases()) {
                    self.patch_jump(label);
                    self.compile_statement_list(case.body(), false)?;
                }

                self.patch_jump(exit);
                if let Some(body) = switch.default() {
                    for node in body {
                        self.create_declarations(node)?;
                    }
                    for node in body {
                        self.compile_stmt(node, false)?;
                    }
                }

                self.pop_switch_control_info();

                self.emit_opcode(Opcode::LoopEnd);
                let num_bindings = self.context.pop_compile_time_environment().num_bindings();
                self.patch_jump_with_target(push_env, num_bindings as u32);
                self.emit_opcode(Opcode::PopEnvironment);
            }
            Node::FunctionDecl(_function) => self.function(node, false)?,
            Node::Return(ret) => {
                if let Some(expr) = ret.expr() {
                    self.compile_expr(expr, true)?;
                } else {
                    self.emit(Opcode::PushUndefined, &[]);
                }
                self.emit(Opcode::Return, &[]);
            }
            Node::Try(t) => {
                self.push_try_control_info(t.finally().is_some());
                let try_start = self.next_opcode_location();
                self.emit(Opcode::TryStart, &[Self::DUMMY_ADDRESS, 0]);
                self.context.push_compile_time_environment(false);
                let push_env = self.jump_with_custom_opcode(Opcode::PushDeclarativeEnvironment);
                for node in t.block().items() {
                    self.create_declarations(node)?;
                }
                for node in t.block().items() {
                    self.compile_stmt(node, false)?;
                }
                let num_bindings = self.context.pop_compile_time_environment().num_bindings();
                self.patch_jump_with_target(push_env, num_bindings as u32);
                self.emit_opcode(Opcode::PopEnvironment);
                self.emit_opcode(Opcode::TryEnd);

                let finally = self.jump();
                self.patch_jump(Label { index: try_start });

                if let Some(catch) = t.catch() {
                    self.push_try_control_info_catch_start();
                    let catch_start = if t.finally().is_some() {
                        Some(self.jump_with_custom_opcode(Opcode::CatchStart))
                    } else {
                        None
                    };
                    self.context.push_compile_time_environment(false);
                    let push_env = self.jump_with_custom_opcode(Opcode::PushDeclarativeEnvironment);
                    if let Some(decl) = catch.parameter() {
                        match decl {
                            Declaration::Identifier { ident, .. } => {
                                self.context
                                    .create_mutable_binding(ident.sym(), false, false)?;
                                self.emit_binding(BindingOpcode::InitLet, ident.sym());
                            }
                            Declaration::Pattern(pattern) => {
                                for ident in pattern.idents() {
                                    self.context.create_mutable_binding(ident, false, false)?;
                                }
                                self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                            }
                        }
                    } else {
                        self.emit_opcode(Opcode::Pop);
                    }
                    for node in catch.block().items() {
                        self.create_declarations(node)?;
                    }
                    for node in catch.block().items() {
                        self.compile_stmt(node, use_expr)?;
                    }
                    let num_bindings = self.context.pop_compile_time_environment().num_bindings();
                    self.patch_jump_with_target(push_env, num_bindings as u32);
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

                    for node in finally.items() {
                        self.compile_stmt(node, false)?;
                    }
                    self.emit_opcode(Opcode::FinallyEnd);
                    self.pop_try_control_info(Some(finally_start_address));
                } else {
                    self.pop_try_control_info(None);
                }
            }
            // TODO: implement AsyncFunctionDecl
            Node::GeneratorDecl(_) => self.function(node, false)?,
            // TODO: implement AsyncGeneratorDecl
            Node::AsyncFunctionDecl(_) | Node::AsyncGeneratorDecl(_) => {
                self.emit_opcode(Opcode::PushUndefined);
            }
            Node::Empty => {}
            expr => self.compile_expr(expr, use_expr)?,
        }
        Ok(())
    }

    pub(crate) fn function(&mut self, function: &Node, use_expr: bool) -> JsResult<()> {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum FunctionKind {
            Declaration,
            Expression,
            Arrow,
        }

        let (kind, name, parameters, body, generator) = match function {
            Node::FunctionDecl(function) => (
                FunctionKind::Declaration,
                Some(function.name()),
                function.parameters(),
                function.body(),
                false,
            ),
            Node::GeneratorDecl(generator) => (
                FunctionKind::Declaration,
                Some(generator.name()),
                generator.parameters(),
                generator.body(),
                true,
            ),
            Node::FunctionExpr(function) => (
                FunctionKind::Expression,
                function.name(),
                function.parameters(),
                function.body(),
                false,
            ),
            Node::GeneratorExpr(generator) => (
                FunctionKind::Expression,
                generator.name(),
                generator.parameters(),
                generator.body(),
                true,
            ),
            Node::ArrowFunctionDecl(function) => (
                FunctionKind::Arrow,
                function.name(),
                function.params(),
                function.body(),
                false,
            ),
            _ => unreachable!(),
        };

        let strict = body.strict() || self.code_block.strict;
        let length = parameters.parameters.len() as u32;
        let mut code = CodeBlock::new(name.unwrap_or(Sym::EMPTY_STRING), length, strict, true);

        if let FunctionKind::Arrow = kind {
            code.constructor = false;
            code.this_mode = ThisMode::Lexical;
        }

        if generator {
            code.constructor = false;
        }

        let mut compiler = ByteCompiler {
            code_block: code,
            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            bindings_map: FxHashMap::default(),
            jump_info: Vec::new(),
            context: self.context,
        };

        compiler.context.push_compile_time_environment(true);

        // An arguments object is added when all of the following conditions are met
        // - If not in an arrow function (10.2.11.16)
        // - If the parameter list does not contain `arguments` (10.2.11.17)
        // Note: This following just means, that we add an extra environment for the arguments.
        // - If there are default parameters or if lexical names and function names do not contain `arguments` (10.2.11.18)
        if !(kind == FunctionKind::Arrow) && !parameters.has_arguments() {
            compiler
                .context
                .create_mutable_binding(Sym::ARGUMENTS, false, true)?;
            compiler.code_block.arguments_binding = Some(
                compiler
                    .context
                    .initialize_mutable_binding(Sym::ARGUMENTS, false),
            );
        }

        for parameter in parameters.parameters.iter() {
            if parameter.is_rest_param() {
                compiler.emit_opcode(Opcode::RestParameterInit);
            }

            match parameter.declaration() {
                Declaration::Identifier { ident, .. } => {
                    compiler
                        .context
                        .create_mutable_binding(ident.sym(), false, true)?;
                    if let Some(init) = parameter.declaration().init() {
                        let skip = compiler.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                        compiler.compile_expr(init, true)?;
                        compiler.patch_jump(skip);
                    }
                    compiler.emit_binding(BindingOpcode::InitArg, ident.sym());
                }
                Declaration::Pattern(pattern) => {
                    for ident in pattern.idents() {
                        compiler
                            .context
                            .create_mutable_binding(ident, false, true)?;
                    }
                    compiler.compile_declaration_pattern(pattern, BindingOpcode::InitArg)?;
                }
            }
        }

        if !parameters.has_rest_parameter() {
            compiler.emit_opcode(Opcode::RestParameterPop);
        }

        let env_label = if parameters.has_expressions() {
            compiler.code_block.num_bindings = compiler.context.get_binding_number();
            compiler.context.push_compile_time_environment(true);
            Some(compiler.jump_with_custom_opcode(Opcode::PushFunctionEnvironment))
        } else {
            None
        };

        // When a generator object is created from a generator function, the generator executes until here to init parameters.
        if generator {
            compiler.emit_opcode(Opcode::PushUndefined);
            compiler.emit_opcode(Opcode::Yield);
        }

        for node in body.items() {
            compiler.create_declarations(node)?;
        }

        compiler.compile_statement_list(body, false)?;

        if let Some(env_label) = env_label {
            let num_bindings = compiler
                .context
                .pop_compile_time_environment()
                .num_bindings();
            compiler.patch_jump_with_target(env_label, num_bindings as u32);
            compiler.context.pop_compile_time_environment();
        } else {
            compiler.code_block.num_bindings = compiler
                .context
                .pop_compile_time_environment()
                .num_bindings();
        }

        compiler.code_block.params = parameters.clone();

        // TODO These are redundant if a function returns so may need to check if a function returns and adding these if it doesn't
        compiler.emit(Opcode::PushUndefined, &[]);
        compiler.emit(Opcode::Return, &[]);

        let code = Gc::new(compiler.finish());

        let index = self.code_block.functions.len() as u32;
        self.code_block.functions.push(code);

        if generator {
            self.emit(Opcode::GetGenerator, &[index]);
        } else {
            self.emit(Opcode::GetFunction, &[index]);
        }

        match kind {
            FunctionKind::Declaration => {
                self.emit_binding(
                    BindingOpcode::InitVar,
                    name.expect("function declaration must have a name"),
                );
            }
            FunctionKind::Expression | FunctionKind::Arrow => {
                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn call(&mut self, node: &Node, use_expr: bool) -> JsResult<()> {
        #[derive(PartialEq)]
        enum CallKind {
            Call,
            New,
        }

        let (call, kind) = match node {
            Node::Call(call) => (call, CallKind::Call),
            Node::New(new) => (new.call(), CallKind::New),
            _ => unreachable!(),
        };

        match call.expr() {
            Node::GetConstField(field) => {
                self.compile_expr(field.obj(), true)?;
                if kind == CallKind::Call {
                    self.emit(Opcode::Dup, &[]);
                }
                let index = self.get_or_insert_name(field.field());
                self.emit(Opcode::GetPropertyByName, &[index]);
            }
            Node::GetField(field) => {
                self.compile_expr(field.obj(), true)?;
                if kind == CallKind::Call {
                    self.emit(Opcode::Dup, &[]);
                }
                self.compile_expr(field.field(), true)?;
                self.emit(Opcode::Swap, &[]);
                self.emit(Opcode::GetPropertyByValue, &[]);
            }
            expr => {
                self.compile_expr(expr, true)?;
                if kind == CallKind::Call {
                    self.emit_opcode(Opcode::This);
                    self.emit_opcode(Opcode::Swap);
                }
            }
        }

        for arg in call.args().iter() {
            self.compile_expr(arg, true)?;
        }

        let last_is_rest_parameter = matches!(call.args().last(), Some(Node::Spread(_)));

        match kind {
            CallKind::Call if last_is_rest_parameter => {
                self.emit(Opcode::CallWithRest, &[call.args().len() as u32]);
            }
            CallKind::Call => self.emit(Opcode::Call, &[call.args().len() as u32]),
            CallKind::New if last_is_rest_parameter => {
                self.emit(Opcode::NewWithRest, &[call.args().len() as u32]);
            }
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
        pattern: &DeclarationPattern,
        def: BindingOpcode,
    ) -> JsResult<()> {
        match pattern {
            DeclarationPattern::Object(pattern) => {
                let skip_init = self.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                if let Some(init) = pattern.init() {
                    self.compile_expr(init, true)?;
                } else {
                    self.emit_opcode(Opcode::PushUndefined);
                }
                self.patch_jump(skip_init);
                self.emit_opcode(Opcode::ValueNotNullOrUndefined);

                self.emit_opcode(Opcode::RequireObjectCoercible);

                for binding in pattern.bindings() {
                    use BindingPatternTypeObject::{
                        BindingPattern, Empty, RestProperty, SingleName,
                    };

                    match binding {
                        // ObjectBindingPattern : { }
                        Empty => {}
                        //  SingleNameBinding : BindingIdentifier Initializer[opt]
                        SingleName {
                            ident,
                            property_name,
                            default_init,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            let index = self.get_or_insert_name(*property_name);
                            self.emit(Opcode::GetPropertyByName, &[index]);

                            if let Some(init) = default_init {
                                let skip = self.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true)?;
                                self.patch_jump(skip);
                            }
                            self.emit_binding(def, *ident);
                        }
                        //  BindingRestProperty : ... BindingIdentifier
                        RestProperty {
                            ident,
                            excluded_keys,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            self.emit_opcode(Opcode::PushEmptyObject);

                            for key in excluded_keys {
                                self.emit_push_literal(Literal::String(
                                    self.interner().resolve_expect(*key).into(),
                                ));
                            }

                            self.emit(Opcode::CopyDataProperties, &[excluded_keys.len() as u32]);
                            self.emit_binding(def, *ident);
                        }
                        BindingPattern {
                            ident,
                            pattern,
                            default_init,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            let index = self.get_or_insert_name(*ident);
                            self.emit(Opcode::GetPropertyByName, &[index]);

                            if let Some(init) = default_init {
                                let skip = self.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true)?;
                                self.patch_jump(skip);
                            }

                            self.compile_declaration_pattern(pattern, def)?;
                        }
                    }
                }

                self.emit_opcode(Opcode::Pop);
            }
            DeclarationPattern::Array(pattern) => {
                let skip_init = self.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                if let Some(init) = pattern.init() {
                    self.compile_expr(init, true)?;
                } else {
                    self.emit_opcode(Opcode::PushUndefined);
                }
                self.patch_jump(skip_init);
                self.emit_opcode(Opcode::ValueNotNullOrUndefined);
                self.emit_opcode(Opcode::InitIterator);

                for (i, binding) in pattern.bindings().iter().enumerate() {
                    use BindingPatternTypeArray::{
                        BindingPattern, BindingPatternRest, Elision, Empty, SingleName,
                        SingleNameRest,
                    };

                    let next = if i == pattern.bindings().len() - 1 {
                        Opcode::IteratorNextFull
                    } else {
                        Opcode::IteratorNext
                    };

                    match binding {
                        // ArrayBindingPattern : [ ]
                        Empty => {
                            self.emit_opcode(Opcode::PushFalse);
                        }
                        // ArrayBindingPattern : [ Elision ]
                        Elision => {
                            self.emit_opcode(next);
                            self.emit_opcode(Opcode::Pop);
                        }
                        // SingleNameBinding : BindingIdentifier Initializer[opt]
                        SingleName {
                            ident,
                            default_init,
                        } => {
                            self.emit_opcode(next);
                            if let Some(init) = default_init {
                                let skip = self.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true)?;
                                self.patch_jump(skip);
                            }
                            self.emit_binding(def, *ident);
                        }
                        // BindingElement : BindingPattern Initializer[opt]
                        BindingPattern { pattern } => {
                            self.emit_opcode(next);
                            self.compile_declaration_pattern(pattern, def)?;
                        }
                        // BindingRestElement : ... BindingIdentifier
                        SingleNameRest { ident } => {
                            self.emit_opcode(Opcode::IteratorToArray);
                            self.emit_binding(def, *ident);
                            self.emit_opcode(Opcode::PushTrue);
                        }
                        // BindingRestElement : ... BindingPattern
                        BindingPatternRest { pattern } => {
                            self.emit_opcode(Opcode::IteratorToArray);
                            self.compile_declaration_pattern(pattern, def)?;
                            self.emit_opcode(Opcode::PushTrue);
                        }
                    }
                }

                if pattern.bindings().is_empty() {
                    self.emit_opcode(Opcode::PushFalse);
                }

                self.emit_opcode(Opcode::IteratorClose);
            }
        }
        Ok(())
    }

    pub(crate) fn create_declarations(&mut self, node: &Node) -> JsResult<bool> {
        let mut has_identifier_argument = false;

        match node {
            Node::VarDeclList(list) => {
                for decl in list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            let ident = ident.sym();
                            if ident == Sym::ARGUMENTS {
                                has_identifier_argument = true;
                            }
                            self.context.create_mutable_binding(ident, true, true)?;
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                if ident == Sym::ARGUMENTS {
                                    has_identifier_argument = true;
                                }
                                self.context.create_mutable_binding(ident, true, true)?;
                            }
                        }
                    }
                }
            }
            Node::LetDeclList(list) => {
                for decl in list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            let ident = ident.sym();
                            if ident == Sym::ARGUMENTS {
                                has_identifier_argument = true;
                            }
                            self.context.create_mutable_binding(ident, false, false)?;
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                if ident == Sym::ARGUMENTS {
                                    has_identifier_argument = true;
                                }
                                self.context.create_mutable_binding(ident, false, false)?;
                            }
                        }
                    }
                }
            }
            Node::ConstDeclList(list) => {
                for decl in list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            let ident = ident.sym();
                            if ident == Sym::ARGUMENTS {
                                has_identifier_argument = true;
                            }
                            self.context.create_immutable_binding(ident)?;
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                if ident == Sym::ARGUMENTS {
                                    has_identifier_argument = true;
                                }
                                self.context.create_immutable_binding(ident)?;
                            }
                        }
                    }
                }
            }
            Node::FunctionDecl(decl) => {
                let ident = decl.name();
                if ident == Sym::ARGUMENTS {
                    has_identifier_argument = true;
                }
                self.context.create_mutable_binding(ident, true, true)?;
            }
            Node::GeneratorDecl(decl) => {
                let ident = decl.name();
                if ident == Sym::ARGUMENTS {
                    has_identifier_argument = true;
                }
                self.context.create_mutable_binding(ident, true, true)?;
            }
            Node::AsyncFunctionDecl(decl) => {
                let ident = decl.name();
                if ident == Sym::ARGUMENTS {
                    has_identifier_argument = true;
                }
                self.context.create_mutable_binding(ident, true, true)?;
            }
            Node::AsyncGeneratorDecl(decl) => {
                let ident = decl.name();
                if ident == Sym::ARGUMENTS {
                    has_identifier_argument = true;
                }
                self.context.create_mutable_binding(ident, true, true)?;
            }
            Node::DoWhileLoop(do_while_loop) => {
                if !matches!(do_while_loop.body(), Node::Block(_)) {
                    self.create_declarations(do_while_loop.body())?;
                }
            }
            Node::ForInLoop(for_in_loop) => {
                if !matches!(for_in_loop.body(), Node::Block(_)) {
                    self.create_declarations(for_in_loop.body())?;
                }
            }
            Node::ForOfLoop(for_of_loop) => {
                if !matches!(for_of_loop.body(), Node::Block(_)) {
                    self.create_declarations(for_of_loop.body())?;
                }
            }
            _ => {}
        }
        Ok(has_identifier_argument)
    }
}
