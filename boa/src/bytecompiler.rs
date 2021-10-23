use gc::Gc;

use crate::{
    builtins::function::ThisMode,
    syntax::ast::{
        node::{
            Declaration, GetConstField, GetField, MethodDefinitionKind, PropertyDefinition,
            PropertyName, StatementList,
        },
        op::{AssignOp, BinOp, BitOp, CompOp, LogOp, NumOp, UnaryOp},
        Const, Node,
    },
    vm::{CodeBlock, Opcode},
    JsBigInt, JsString, JsValue,
};
use std::collections::HashMap;

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
    label: Option<Box<str>>,
    start_address: u32,
    is_loop: bool,
    breaks: Vec<Label>,
}

#[derive(Debug, Clone, Copy)]
enum Access<'a> {
    Variable { index: u32 },
    ByName { node: &'a GetConstField },
    ByValue { node: &'a GetField },
    This,
}

#[derive(Debug)]
pub struct ByteCompiler {
    code_block: CodeBlock,
    literals_map: HashMap<Literal, u32>,
    names_map: HashMap<JsString, u32>,
    functions_map: HashMap<JsString, u32>,
    jump_info: Vec<JumpControlInfo>,
    top_level: bool,
}

impl ByteCompiler {
    /// Represents a placeholder address that will be patched later.
    const DUMMY_ADDRESS: u32 = u32::MAX;

    #[inline]
    pub fn new(name: JsString, strict: bool) -> Self {
        Self {
            code_block: CodeBlock::new(name, 0, strict, false),
            literals_map: HashMap::new(),
            names_map: HashMap::new(),
            functions_map: HashMap::new(),
            jump_info: Vec::new(),
            top_level: true,
        }
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
    fn get_or_insert_name(&mut self, name: &str) -> u32 {
        if let Some(index) = self.names_map.get(name) {
            return *index;
        }

        let name = JsString::new(name);
        let index = self.code_block.variables.len() as u32;
        self.code_block.variables.push(name.clone());
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
    fn push_loop_control_info(&mut self, label: Option<Box<str>>, start_address: u32) {
        self.jump_info.push(JumpControlInfo {
            label,
            start_address,
            is_loop: true,
            breaks: Vec::new(),
        })
    }

    #[inline]
    fn pop_loop_control_info(&mut self) {
        let loop_info = self.jump_info.pop().unwrap();

        assert!(loop_info.is_loop);

        for label in loop_info.breaks {
            self.patch_jump(label);
        }
    }

    #[inline]
    fn push_switch_control_info(&mut self, label: Option<Box<str>>, start_address: u32) {
        self.jump_info.push(JumpControlInfo {
            label,
            start_address,
            is_loop: false,
            breaks: Vec::new(),
        })
    }

    #[inline]
    fn pop_switch_control_info(&mut self) {
        let info = self.jump_info.pop().unwrap();

        assert!(!info.is_loop);

        for label in info.breaks {
            self.patch_jump(label);
        }
    }

    #[inline]
    fn compile_access<'a>(&mut self, node: &'a Node) -> Access<'a> {
        match node {
            Node::Identifier(name) => {
                let index = self.get_or_insert_name(name.as_ref());
                Access::Variable { index }
            }
            Node::GetConstField(node) => Access::ByName { node },
            Node::GetField(node) => Access::ByValue { node },
            Node::This => Access::This,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn access_get(&mut self, access: Access<'_>, use_expr: bool) {
        match access {
            Access::Variable { index: name } => {
                self.emit(Opcode::GetName, &[name]);
            }
            Access::ByName { node } => {
                let index = self.get_or_insert_name(node.field());
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
            Access::Variable { index } => {
                self.emit(Opcode::SetName, &[index]);
            }
            Access::ByName { node } => {
                self.compile_expr(node.obj(), true);
                let index = self.get_or_insert_name(node.field());
                self.emit(Opcode::SetPropertyByName, &[index]);
            }
            Access::ByValue { node } => {
                self.compile_expr(node.field(), true);
                self.compile_expr(node.obj(), true);
                self.emit(Opcode::SetPropertyByValue, &[]);
            }
            Access::This => todo!("access_set 'this'"),
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
                    Const::BigInt(v) => self.emit_push_literal(Literal::BigInt(v.clone())),
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
                        self.compile_expr(unary.target(), true);
                        self.emit(Opcode::Inc, &[]);

                        let access = self.compile_access(unary.target());
                        self.access_set(access, None, use_expr);
                        None
                    }
                    UnaryOp::DecrementPre => {
                        self.compile_expr(unary.target(), true);
                        self.emit(Opcode::Dec, &[]);

                        let access = self.compile_access(unary.target());
                        self.access_set(access, None, use_expr);
                        None
                    }
                    UnaryOp::IncrementPost => {
                        self.compile_expr(unary.target(), true);
                        self.emit(Opcode::Dup, &[]);
                        self.emit(Opcode::Inc, &[]);
                        let access = self.compile_access(unary.target());
                        self.access_set(access, None, false);

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }

                        None
                    }
                    UnaryOp::DecrementPost => {
                        self.compile_expr(unary.target(), true);
                        self.emit(Opcode::Dup, &[]);
                        self.emit(Opcode::Dec, &[]);
                        let access = self.compile_access(unary.target());
                        self.access_set(access, None, false);

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
                        }

                        None
                    }
                    UnaryOp::Delete => match unary.target() {
                        Node::GetConstField(ref get_const_field) => {
                            let index = self.get_or_insert_name(get_const_field.field());
                            self.compile_expr(get_const_field.obj(), true);
                            self.emit(Opcode::DeletePropertyByName, &[index]);
                            None
                        }
                        Node::GetField(ref get_field) => {
                            self.compile_expr(get_field.field(), true);
                            self.compile_expr(get_field.obj(), true);
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
                    UnaryOp::TypeOf => Some(Opcode::TypeOf),
                    UnaryOp::Void => Some(Opcode::Void),
                };

                if let Some(opcode) = opcode {
                    self.compile_expr(unary.target(), true);
                    self.emit(opcode, &[]);
                }

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
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

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
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

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
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

                        if !use_expr {
                            self.emit(Opcode::Pop, &[]);
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
                        PropertyDefinition::IdentifierReference(identifier_reference) => {
                            let index = self.get_or_insert_name(identifier_reference);
                            self.emit(Opcode::SetPropertyByName, &[index]);
                        }
                        PropertyDefinition::Property(name, node) => {
                            self.compile_stmt(node, true);
                            self.emit_opcode(Opcode::Swap);
                            match name {
                                PropertyName::Literal(name) => {
                                    let index = self.get_or_insert_name(name);
                                    self.emit(Opcode::SetPropertyByName, &[index]);
                                }
                                PropertyName::Computed(name_node) => {
                                    self.compile_stmt(name_node, true);
                                    self.emit_opcode(Opcode::Swap);
                                    self.emit_opcode(Opcode::SetPropertyByValue);
                                }
                            }
                        }
                        PropertyDefinition::MethodDefinition(kind, name, func) => {
                            match kind {
                                MethodDefinitionKind::Get => {
                                    self.compile_stmt(&func.clone().into(), true);
                                    self.emit_opcode(Opcode::Swap);
                                    match name {
                                        PropertyName::Literal(name) => {
                                            let index = self.get_or_insert_name(name);
                                            self.emit(Opcode::SetPropertyGetterByName, &[index]);
                                        }
                                        PropertyName::Computed(name_node) => {
                                            self.compile_stmt(name_node, true);
                                            self.emit_opcode(Opcode::Swap);
                                            self.emit_opcode(Opcode::SetPropertyGetterByValue);
                                        }
                                    }
                                }
                                MethodDefinitionKind::Set => {
                                    self.compile_stmt(&func.clone().into(), true);
                                    self.emit_opcode(Opcode::Swap);
                                    match name {
                                        PropertyName::Literal(name) => {
                                            let index = self.get_or_insert_name(name);
                                            self.emit(Opcode::SetPropertySetterByName, &[index]);
                                        }
                                        PropertyName::Computed(name_node) => {
                                            self.compile_stmt(name_node, true);
                                            self.emit_opcode(Opcode::Swap);
                                            self.emit_opcode(Opcode::SetPropertySetterByValue);
                                        }
                                    }
                                }
                                MethodDefinitionKind::Ordinary => {
                                    self.compile_stmt(&func.clone().into(), true);
                                    self.emit_opcode(Opcode::Swap);
                                    match name {
                                        PropertyName::Literal(name) => {
                                            let index = self.get_or_insert_name(name);
                                            self.emit(Opcode::SetPropertyByName, &[index]);
                                        }
                                        PropertyName::Computed(name_node) => {
                                            self.compile_stmt(name_node, true);
                                            self.emit_opcode(Opcode::Swap);
                                            self.emit_opcode(Opcode::SetPropertyByValue);
                                        }
                                    }
                                }
                                MethodDefinitionKind::Generator => {
                                    // TODO: Implement generators
                                    self.emit_opcode(Opcode::PushUndefined);
                                    self.emit_opcode(Opcode::Swap);
                                    match name {
                                        PropertyName::Literal(name) => {
                                            let index = self.get_or_insert_name(name);
                                            self.emit(Opcode::SetPropertyByName, &[index]);
                                        }
                                        PropertyName::Computed(name_node) => {
                                            self.compile_stmt(name_node, true);
                                            self.emit_opcode(Opcode::Swap);
                                            self.emit_opcode(Opcode::SetPropertyByValue);
                                        }
                                    }
                                }
                                MethodDefinitionKind::Async => {
                                    // TODO: Implement async
                                    self.emit_opcode(Opcode::PushUndefined);
                                    self.emit_opcode(Opcode::Swap);
                                    match name {
                                        PropertyName::Literal(name) => {
                                            let index = self.get_or_insert_name(name);
                                            self.emit(Opcode::SetPropertyByName, &[index])
                                        }
                                        PropertyName::Computed(name_node) => {
                                            self.compile_stmt(name_node, true);
                                            self.emit_opcode(Opcode::Swap);
                                            self.emit_opcode(Opcode::SetPropertyByValue);
                                        }
                                    }
                                }
                                MethodDefinitionKind::AsyncGenerator => {
                                    // TODO: Implement async generators
                                    self.emit_opcode(Opcode::PushUndefined);
                                    self.emit_opcode(Opcode::Swap);
                                    match name {
                                        PropertyName::Literal(name) => {
                                            let index = self.get_or_insert_name(name);
                                            self.emit(Opcode::SetPropertyByName, &[index])
                                        }
                                        PropertyName::Computed(name_node) => {
                                            self.compile_stmt(name_node, true);
                                            self.emit_opcode(Opcode::Swap);
                                            self.emit_opcode(Opcode::SetPropertyByValue);
                                        }
                                    }
                                }
                            }
                        }
                        // TODO: Spread Object
                        PropertyDefinition::SpreadObject(_) => todo!(),
                    }
                }

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            Node::Identifier(name) => {
                let index = self.get_or_insert_name(name.as_ref());
                let access = Access::Variable { index };
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
            Node::FunctionExpr(_function) => self.function(expr, use_expr),
            Node::ArrowFunctionDecl(_function) => self.function(expr, use_expr),
            Node::Call(call) => {
                for arg in call.args().iter().rev() {
                    self.compile_expr(arg, true);
                }
                match call.expr() {
                    Node::GetConstField(field) => {
                        self.compile_expr(field.obj(), true);
                        self.emit(Opcode::Dup, &[]);
                        let index = self.get_or_insert_name(field.field());
                        self.emit(Opcode::GetPropertyByName, &[index]);
                    }
                    Node::GetField(field) => {
                        self.compile_expr(field.obj(), true);
                        self.emit(Opcode::Dup, &[]);
                        self.compile_expr(field.field(), true);
                        self.emit(Opcode::Swap, &[]);
                        self.emit(Opcode::GetPropertyByValue, &[]);
                    }
                    expr => {
                        self.emit(Opcode::This, &[]);
                        self.compile_expr(expr, true);
                    }
                }
                self.emit(Opcode::Call, &[call.args().len() as u32]);

                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
            expr => todo!("TODO compile: {}", expr),
        }
    }

    #[inline]
    pub fn compile_stmt(&mut self, node: &Node, use_expr: bool) {
        match node {
            Node::VarDeclList(list) => {
                for decl in list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            let index = self.get_or_insert_name(ident.as_ref());
                            self.emit(Opcode::DefVar, &[index]);

                            if let Some(expr) = decl.init() {
                                self.compile_expr(expr, true);
                                self.emit(Opcode::InitLexical, &[index]);
                            };
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                let index = self.get_or_insert_name(ident);
                                self.emit(Opcode::DefVar, &[index]);

                                if let Some(expr) = decl.init() {
                                    self.compile_expr(expr, true);
                                    self.emit(Opcode::InitLexical, &[index]);
                                };
                            }
                        }
                    }
                }
            }
            Node::LetDeclList(list) => {
                for decl in list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            let index = self.get_or_insert_name(ident.as_ref());
                            self.emit(Opcode::DefLet, &[index]);

                            if let Some(expr) = decl.init() {
                                self.compile_expr(expr, true);
                                self.emit(Opcode::InitLexical, &[index]);
                            };
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                let index = self.get_or_insert_name(ident);
                                self.emit(Opcode::DefLet, &[index]);

                                if let Some(expr) = decl.init() {
                                    self.compile_expr(expr, true);
                                    self.emit(Opcode::InitLexical, &[index]);
                                };
                            }
                        }
                    }
                }
            }
            Node::ConstDeclList(list) => {
                for decl in list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            let index = self.get_or_insert_name(ident.as_ref());
                            self.emit(Opcode::DefConst, &[index]);

                            if let Some(expr) = decl.init() {
                                self.compile_expr(expr, true);
                                self.emit(Opcode::InitLexical, &[index]);
                            };
                        }
                        Declaration::Pattern(pattern) => {
                            for ident in pattern.idents() {
                                let index = self.get_or_insert_name(ident);
                                self.emit(Opcode::DefConst, &[index]);

                                if let Some(expr) = decl.init() {
                                    self.compile_expr(expr, true);
                                    self.emit(Opcode::InitLexical, &[index]);
                                };
                            }
                        }
                    }
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
                let start_address = self.next_opcode_location();
                self.push_loop_control_info(while_.label().map(Into::into), start_address);

                self.compile_expr(while_.cond(), true);
                let exit = self.jump_if_false();
                self.compile_stmt(while_.body(), false);
                self.emit(Opcode::Jump, &[start_address]);
                self.patch_jump(exit);

                self.pop_loop_control_info();
            }
            Node::DoWhileLoop(do_while) => {
                let start_address = self.next_opcode_location();
                self.push_loop_control_info(do_while.label().map(Into::into), start_address);

                self.compile_stmt(do_while.body(), false);

                self.compile_expr(do_while.cond(), true);
                self.emit(Opcode::JumpIfTrue, &[start_address]);

                self.pop_loop_control_info();
            }
            Node::Continue(node) => {
                let label = self.jump();
                let mut items = self.jump_info.iter_mut().rev().filter(|info| info.is_loop);
                let target = if node.label().is_none() {
                    items.next()
                } else {
                    items.find(|info| info.label.as_deref() == node.label())
                }
                .expect("continue target")
                .start_address;

                self.patch_jump_with_target(label, target);
            }
            Node::Break(node) => {
                let label = self.jump();
                if node.label().is_none() {
                    self.jump_info.last_mut().unwrap().breaks.push(label);
                } else {
                    for info in self.jump_info.iter_mut().rev() {
                        if info.label.as_deref() == node.label() {
                            info.breaks.push(label);
                            break;
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
            Node::Switch(switch) => {
                let start_address = self.next_opcode_location();
                self.push_switch_control_info(None, start_address);

                self.compile_expr(switch.val(), true);
                let mut labels = Vec::with_capacity(switch.cases().len());
                for case in switch.cases() {
                    self.compile_expr(case.condition(), true);
                    labels.push(self.jump_with_custom_opcode(Opcode::Case));
                }

                let exit = self.jump_with_custom_opcode(Opcode::Default);

                for (label, case) in labels.into_iter().zip(switch.cases()) {
                    self.patch_jump(label);
                    self.compile_statement_list(case.body(), false);
                }

                self.patch_jump(exit);
                if let Some(body) = switch.default() {
                    for node in body {
                        self.compile_stmt(node, false);
                    }
                }

                self.pop_switch_control_info();
            }
            Node::FunctionDecl(_function) => self.function(node, false),
            Node::Return(ret) => {
                if let Some(expr) = ret.expr() {
                    self.compile_expr(expr, true);
                } else {
                    self.emit(Opcode::PushUndefined, &[]);
                }
                self.emit(Opcode::Return, &[]);
            }
            Node::Empty => {}
            expr => self.compile_expr(expr, use_expr),
        }
    }

    pub(crate) fn function(&mut self, function: &Node, use_expr: bool) {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum FunctionKind {
            Declaration,
            Expression,
            Arrow,
        }

        let (kind, name, parameters, body) = match function {
            Node::FunctionDecl(function) => (
                FunctionKind::Declaration,
                Some(function.name()),
                function.parameters(),
                function.body(),
            ),
            Node::FunctionExpr(function) => (
                FunctionKind::Expression,
                function.name(),
                function.parameters(),
                function.body(),
            ),
            Node::ArrowFunctionDecl(function) => (
                FunctionKind::Arrow,
                None,
                function.params(),
                function.body(),
            ),
            _ => unreachable!(),
        };

        let length = parameters.len() as u32;
        let mut code = CodeBlock::new(name.unwrap_or("").into(), length, false, true);

        if let FunctionKind::Arrow = kind {
            code.constructor = false;
            code.this_mode = ThisMode::Lexical;
        }

        let mut compiler = ByteCompiler {
            code_block: code,
            literals_map: HashMap::new(),
            names_map: HashMap::new(),
            functions_map: HashMap::new(),
            jump_info: Vec::new(),
            top_level: false,
        };

        for node in body.items() {
            compiler.compile_stmt(node, false);
        }

        compiler.code_block.params = parameters.to_owned().into_boxed_slice();

        // TODO These are redundant if a function returns so may need to check if a function returns and adding these if it doesn't
        compiler.emit(Opcode::PushUndefined, &[]);
        compiler.emit(Opcode::Return, &[]);

        let code = Gc::new(compiler.finish());

        let index = self.code_block.functions.len() as u32;
        self.code_block.functions.push(code);

        self.emit(Opcode::GetFunction, &[index]);

        match kind {
            FunctionKind::Declaration => {
                let index = self.get_or_insert_name(name.unwrap());
                let access = Access::Variable { index };
                self.access_set(access, None, false);
            }
            FunctionKind::Expression | FunctionKind::Arrow => {
                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
        }
    }

    #[inline]
    pub fn finish(self) -> CodeBlock {
        self.code_block
    }
}
