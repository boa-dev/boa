use crate::{
    syntax::ast::{
        node::{GetConstField, GetField, Identifier, StatementList},
        op::{AssignOp, BinOp, BitOp, CompOp, LogOp, NumOp, UnaryOp},
        Const, Node,
    },
    value::RcBigInt,
    vm::{CodeBlock, Opcode},
    JsString, Value,
};

use std::collections::HashMap;

#[inline]
fn u16_to_array(value: u16) -> [u8; 2] {
    // Safety: Transmuting a `u16` primitive to
    // an array of 2 bytes is safe.
    unsafe { std::mem::transmute(value) }
}

#[inline]
fn u32_to_array(value: u32) -> [u8; 4] {
    // Safety: Transmuting a `u32` primitive to
    // an array of 4 bytes is safe.
    unsafe { std::mem::transmute(value) }
}

#[inline]
fn u64_to_array(value: u64) -> [u8; 8] {
    // Safety: Transmuting a `u64` primitive to
    // an array of 8 bytes is safe.
    unsafe { std::mem::transmute(value) }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Literal {
    String(JsString),
    BigInt(RcBigInt),
}

#[must_use]
#[derive(Debug, Clone, Copy)]
struct Label {
    index: u32,
}

#[derive(Debug, Clone)]
struct LoopControlInfo {
    label: Option<Box<str>>,
    loop_start: u32,
    continues: Vec<Label>,
    breaks: Vec<Label>,
}

#[derive(Debug, Clone, Copy)]
enum Access<'a> {
    Variable { name: &'a Identifier },
    ByName { node: &'a GetConstField },
    ByValue { node: &'a GetField },
    This,
}

#[derive(Debug)]
pub struct ByteCompiler {
    code_block: CodeBlock,
    literals_map: HashMap<Literal, u32>,
    names_map: HashMap<JsString, u32>,
    loops: Vec<LoopControlInfo>,
}

impl Default for ByteCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ByteCompiler {
    /// Represents a placeholder address that will be patched later.
    const DUMMY_ADDRESS: u32 = u32::MAX;

    #[inline]
    pub fn new() -> Self {
        Self {
            code_block: CodeBlock::new(),
            literals_map: HashMap::new(),
            names_map: HashMap::new(),
            loops: Vec::new(),
        }
    }

    #[inline]
    fn get_or_insert_literal(&mut self, liternal: Literal) -> u32 {
        if let Some(index) = self.literals_map.get(&liternal) {
            return *index;
        }

        let value = match liternal.clone() {
            Literal::String(value) => Value::from(value),
            Literal::BigInt(value) => Value::from(value),
        };

        let index = self.code_block.literals.len() as u32;
        self.code_block.literals.push(value);
        self.literals_map.insert(liternal, index);
        index
    }

    #[inline]
    fn get_or_insert_name(&mut self, name: JsString) -> u32 {
        if let Some(index) = self.names_map.get(&name) {
            return *index;
        }

        let index = self.code_block.names.len() as u32;
        self.code_block.names.push(name.clone());
        self.names_map.insert(name, index);
        index
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
        self.code_block.code.extend(&u64_to_array(value));
    }

    #[inline]
    fn emit_u32(&mut self, value: u32) {
        self.code_block.code.extend(&u32_to_array(value));
    }

    #[inline]
    fn emit_u16(&mut self, value: u16) {
        self.code_block.code.extend(&u16_to_array(value));
    }

    #[inline]
    fn emit_opcode(&mut self, opcode: Opcode) {
        self.emit_u8(opcode as u8)
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
            x if x as i8 as i32 == x => {
                self.emit_opcode(Opcode::PushInt8);
                self.emit_u8(x as i8 as u8);
            }
            x if x as i16 as i32 == x => {
                self.emit_opcode(Opcode::PushInt16);
                self.emit_u16(x as i16 as u16);
            }
            x => self.emit(Opcode::PushInt32, &[x as _]),
        }
    }

    #[inline]
    fn emit_push_literal(&mut self, liternal: Literal) {
        let index = self.get_or_insert_literal(liternal);
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
            } else {
                return self.emit_opcode(Opcode::PushNegativeInfinity);
            }
        }

        // Check if the f64 value can fit in an i32.
        #[allow(clippy::float_cmp)]
        if value as i32 as f64 == value {
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

        let bytes = u32_to_array(target);
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
    fn push_loop_control_info(&mut self, label: Option<Box<str>>, loop_start: u32) {
        self.loops.push(LoopControlInfo {
            label,
            loop_start,
            continues: Vec::new(),
            breaks: Vec::new(),
        })
    }

    #[inline]
    fn pop_loop_control_info(&mut self) {
        let loop_info = self.loops.pop().unwrap();

        for label in loop_info.continues {
            self.patch_jump_with_target(label, loop_info.loop_start);
        }

        for label in loop_info.breaks {
            self.patch_jump(label);
        }
    }

    #[inline]
    fn compile_access<'a>(&mut self, node: &'a Node) -> Access<'a> {
        match node {
            Node::Identifier(name) => Access::Variable { name },
            Node::GetConstField(node) => Access::ByName { node },
            Node::GetField(node) => Access::ByValue { node },
            Node::This => Access::This,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn access_get(&mut self, access: Access<'_>, use_expr: bool) {
        match access {
            Access::Variable { name } => {
                let index = self.get_or_insert_name(name.as_ref().into());
                self.emit(Opcode::GetName, &[index]);
            }
            Access::ByName { node } => {
                let index = self.get_or_insert_name(node.field().into());
                self.compile_expr(node.obj(), true);
                self.emit(Opcode::GetPropertyByName, &[index]);
            }
            Access::ByValue { node } => {
                self.compile_expr(node.field(), true);
                self.compile_expr(node.obj(), true);
                self.emit(Opcode::GetPropertyByValue, &[]);
            }
            Access::This => {
                self.emit(Opcode::This, &[]);
            }
        }

        if !use_expr {
            self.emit(Opcode::Pop, &[]);
        }
    }

    #[inline]
    fn access_set(&mut self, access: Access<'_>, expr: Option<&Node>, use_expr: bool) {
        if let Some(expr) = expr {
            self.compile_expr(expr, true);
        }

        if use_expr {
            self.emit(Opcode::Dup, &[]);
        }

        match access {
            Access::Variable { name } => {
                let index = self.get_or_insert_name(name.as_ref().into());
                self.emit(Opcode::SetName, &[index]);
            }
            Access::ByName { node } => {
                self.compile_expr(node.obj(), true);
                let index = self.get_or_insert_name(node.field().into());
                self.emit(Opcode::SetPropertyByName, &[index]);
            }
            Access::ByValue { node } => {
                self.compile_expr(node.field(), true);
                self.compile_expr(node.obj(), true);
                self.emit(Opcode::SetPropertyByValue, &[]);
            }
            Access::This => todo!("access_get 'this'"),
        }
    }

    #[inline]
    pub fn compile_statement_list(&mut self, list: &StatementList, use_expr: bool) {
        for (i, node) in list.items().iter().enumerate() {
            if i + 1 == list.items().len() {
                self.compile_stmt(node, use_expr);
                break;
            }

            self.compile_stmt(node, false);
        }
    }

    #[inline]
    pub fn compile_expr(&mut self, expr: &Node, use_expr: bool) {
        match expr {
            Node::Const(c) => {
                match c {
                    Const::String(v) => self.emit_push_literal(Literal::String(v.as_ref().into())),
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
                    UnaryOp::IncrementPre => todo!(),
                    UnaryOp::DecrementPre => todo!(),
                    UnaryOp::IncrementPost => todo!(),
                    UnaryOp::DecrementPost => todo!(),
                    UnaryOp::Delete => todo!(),
                    UnaryOp::Minus => Some(Opcode::Neg),
                    UnaryOp::Plus => Some(Opcode::Pos),
                    UnaryOp::Not => Some(Opcode::LogicalNot),
                    UnaryOp::Tilde => Some(Opcode::BitNot),
                    UnaryOp::TypeOf => Some(Opcode::TypeOf),
                    UnaryOp::Void => Some(Opcode::Void),
                };

                if let Some(opcode) = opcode {
                    self.compile_expr(unary.target(), true);
                    self.emit(opcode, &[]);

                    if !use_expr {
                        self.emit(Opcode::Pop, &[]);
                    }
                }
            }
            Node::BinOp(binary) => {
                self.compile_expr(binary.lhs(), true);
                match binary.op() {
                    BinOp::Num(op) => {
                        self.compile_expr(binary.rhs(), true);
                        match op {
                            NumOp::Add => self.emit_opcode(Opcode::Add),
                            NumOp::Sub => self.emit_opcode(Opcode::Sub),
                            NumOp::Div => self.emit_opcode(Opcode::Div),
                            NumOp::Mul => self.emit_opcode(Opcode::Mul),
                            NumOp::Exp => self.emit_opcode(Opcode::Pow),
                            NumOp::Mod => self.emit_opcode(Opcode::Mod),
                        }
                    }
                    BinOp::Bit(op) => {
                        self.compile_expr(binary.rhs(), true);
                        match op {
                            BitOp::And => self.emit_opcode(Opcode::BitAnd),
                            BitOp::Or => self.emit_opcode(Opcode::BitOr),
                            BitOp::Xor => self.emit_opcode(Opcode::BitXor),
                            BitOp::Shl => self.emit_opcode(Opcode::ShiftLeft),
                            BitOp::Shr => self.emit_opcode(Opcode::ShiftRight),
                            BitOp::UShr => self.emit_opcode(Opcode::UnsignedShiftRight),
                        }
                    }
                    BinOp::Comp(op) => {
                        self.compile_expr(binary.rhs(), true);
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
                    }
                    BinOp::Log(op) => {
                        match op {
                            LogOp::And => {
                                let exit = self.jump_with_custom_opcode(Opcode::LogicalAnd);
                                self.compile_expr(binary.rhs(), true);
                                self.emit(Opcode::ToBoolean, &[]);
                                self.patch_jump(exit);
                            }
                            LogOp::Or => {
                                let exit = self.jump_with_custom_opcode(Opcode::LogicalOr);
                                self.compile_expr(binary.rhs(), true);
                                self.emit(Opcode::ToBoolean, &[]);
                                self.patch_jump(exit);
                            }
                            LogOp::Coalesce => {
                                let exit = self.jump_with_custom_opcode(Opcode::Coalesce);
                                self.compile_expr(binary.rhs(), true);
                                self.patch_jump(exit);
                            }
                        };
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
                                self.compile_expr(binary.rhs(), true);
                                self.emit(Opcode::ToBoolean, &[]);
                                self.patch_jump(exit);

                                None
                            }
                            AssignOp::BoolOr => {
                                let exit = self.jump_with_custom_opcode(Opcode::LogicalOr);
                                self.compile_expr(binary.rhs(), true);
                                self.emit(Opcode::ToBoolean, &[]);
                                self.patch_jump(exit);

                                None
                            }
                            AssignOp::Coalesce => {
                                let exit = self.jump_with_custom_opcode(Opcode::Coalesce);
                                self.compile_expr(binary.rhs(), true);
                                self.patch_jump(exit);

                                None
                            }
                        };

                        if let Some(opcode) = opcode {
                            self.compile_expr(binary.rhs(), true);
                            self.emit(opcode, &[]);
                        }

                        let access = self.compile_access(binary.lhs());
                        self.access_set(access, None, use_expr);
                    }
                    BinOp::Comma => {
                        self.emit(Opcode::Pop, &[]);
                        self.compile_expr(binary.rhs(), true);
                    }
                }

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Node::Object(object) => {
                if object.properties().is_empty() {
                    self.emit(Opcode::PushEmptyObject, &[]);
                } else {
                    todo!("object literal with properties");
                }

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Node::Identifier(name) => {
                let access = Access::Variable { name };
                self.access_get(access, use_expr);
            }
            Node::Assign(assign) => {
                let access = self.compile_access(assign.lhs());
                self.access_set(access, Some(assign.rhs()), use_expr);
            }
            Node::GetConstField(node) => {
                let access = Access::ByName { node };
                self.access_get(access, use_expr);
            }
            Node::GetField(node) => {
                let access = Access::ByValue { node };
                self.access_get(access, use_expr);
            }
            Node::ConditionalOp(op) => {
                self.compile_expr(op.cond(), true);
                let jelse = self.jump_if_false();
                self.compile_expr(op.if_true(), true);
                let exit = self.jump();
                self.patch_jump(jelse);
                self.compile_expr(op.if_false(), true);
                self.patch_jump(exit);

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Node::ArrayDecl(array) => {
                let mut count = 0;
                for element in array.as_ref().iter().rev() {
                    if let Node::Spread(_) = element {
                        todo!("array with spread element");
                    } else {
                        self.compile_expr(element, true);
                    }
                    count += 1;
                }
                self.emit(Opcode::PushNewArray, &[count]);

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Node::This => {
                self.access_get(Access::This, use_expr);
            }
            expr => todo!("TODO compile: {}", expr),
        }
    }

    #[inline]
    pub fn compile_stmt(&mut self, node: &Node, use_expr: bool) {
        match node {
            Node::VarDeclList(list) => {
                for decl in list.as_ref() {
                    let index = self.get_or_insert_name(decl.name().into());
                    self.emit(Opcode::DefVar, &[index]);

                    if let Some(expr) = decl.init() {
                        self.compile_expr(expr, true);
                        self.emit(Opcode::InitLexical, &[index]);
                    };
                }
            }
            Node::LetDeclList(list) => {
                for decl in list.as_ref() {
                    let index = self.get_or_insert_name(decl.name().into());
                    self.emit(Opcode::DefLet, &[index]);

                    if let Some(expr) = decl.init() {
                        self.compile_expr(expr, true);
                        self.emit(Opcode::InitLexical, &[index]);
                    };
                }
            }
            Node::ConstDeclList(list) => {
                for decl in list.as_ref() {
                    let index = self.get_or_insert_name(decl.name().into());
                    self.emit(Opcode::DefConst, &[index]);

                    if let Some(expr) = decl.init() {
                        self.compile_expr(expr, true);
                        self.emit(Opcode::InitLexical, &[index]);
                    };
                }
            }
            Node::If(node) => {
                self.compile_expr(node.cond(), true);
                let jelse = self.jump_if_false();

                self.compile_stmt(node.body(), false);

                match node.else_node() {
                    None => {
                        self.patch_jump(jelse);
                    }
                    Some(else_body) => {
                        let exit = self.jump();
                        self.patch_jump(jelse);
                        self.compile_stmt(else_body, false);
                        self.patch_jump(exit);
                    }
                }
            }
            Node::WhileLoop(while_) => {
                let loop_start = self.next_opcode_location();
                self.push_loop_control_info(while_.label().map(Into::into), loop_start);

                self.compile_expr(while_.cond(), true);
                let exit = self.jump_if_false();
                self.compile_stmt(while_.body(), false);
                self.emit(Opcode::Jump, &[loop_start]);
                self.patch_jump(exit);

                self.pop_loop_control_info();
            }
            Node::DoWhileLoop(do_while) => {
                let loop_start = self.next_opcode_location();
                self.push_loop_control_info(do_while.label().map(Into::into), loop_start);

                self.compile_stmt(do_while.body(), false);

                self.compile_expr(do_while.cond(), true);
                self.emit(Opcode::JumpIfTrue, &[loop_start]);

                self.pop_loop_control_info();
            }
            Node::Continue(node) => {
                let jump_label = self.jump();
                if node.label().is_none() {
                    self.loops.last_mut().unwrap().continues.push(jump_label);
                } else {
                    for loop_ in self.loops.iter_mut().rev() {
                        if loop_.label.as_deref() == node.label() {
                            loop_.continues.push(jump_label);
                        }
                    }
                }
            }
            Node::Break(node) => {
                let jump_label = self.jump();
                if node.label().is_none() {
                    self.loops.last_mut().unwrap().breaks.push(jump_label);
                } else {
                    for loop_ in self.loops.iter_mut().rev() {
                        if loop_.label.as_deref() == node.label() {
                            loop_.breaks.push(jump_label);
                        }
                    }
                }
            }
            Node::Block(block) => {
                for node in block.items() {
                    self.compile_stmt(node, false);
                }
            }
            Node::Throw(throw) => {
                self.compile_expr(throw.expr(), true);
                self.emit(Opcode::Throw, &[]);
            }
            Node::Empty => {}
            expr => self.compile_expr(expr, use_expr),
        }
    }

    #[inline]
    pub fn finish(self) -> CodeBlock {
        self.code_block
    }
}
