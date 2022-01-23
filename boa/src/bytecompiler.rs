use crate::{
    builtins::function::ThisMode,
    gc::Gc,
    syntax::ast::{
        node::{
            declaration::{BindingPatternTypeArray, BindingPatternTypeObject, DeclarationPattern},
            iteration::IterableLoopInitializer,
            template::TemplateElement,
            Declaration, GetConstField, GetField, MethodDefinitionKind, PropertyDefinition,
            PropertyName, StatementList,
        },
        op::{AssignOp, BinOp, BitOp, CompOp, LogOp, NumOp, UnaryOp},
        Const, Node,
    },
    vm::{CodeBlock, Opcode},
    JsBigInt, JsString, JsValue,
};
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
    Variable { index: u32 },
    ByName { node: &'a GetConstField },
    ByValue { node: &'a GetField },
    This,
}

#[derive(Debug)]
pub struct ByteCompiler<'b> {
    code_block: CodeBlock,
    literals_map: FxHashMap<Literal, u32>,
    names_map: FxHashMap<JsString, u32>,
    jump_info: Vec<JumpControlInfo>,
    interner: &'b Interner,
}

impl<'b> ByteCompiler<'b> {
    /// Represents a placeholder address that will be patched later.
    const DUMMY_ADDRESS: u32 = u32::MAX;

    #[inline]
    pub fn new(name: Sym, strict: bool, interner: &'b Interner) -> Self {
        Self {
            code_block: CodeBlock::new(name, 0, strict, false),
            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            jump_info: Vec::new(),
            interner,
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
    fn push_loop_control_info(&mut self, label: Option<Sym>, start_address: u32) {
        self.jump_info.push(JumpControlInfo {
            label,
            start_address,
            kind: JumpControlInfoKind::Loop,
            breaks: Vec::new(),
            try_continues: Vec::new(),
            for_of_in_loop: false,
        })
    }

    #[inline]
    fn push_loop_control_info_for_of_in_loop(&mut self, label: Option<Sym>, start_address: u32) {
        self.jump_info.push(JumpControlInfo {
            label,
            start_address,
            kind: JumpControlInfoKind::Loop,
            breaks: Vec::new(),
            try_continues: Vec::new(),
            for_of_in_loop: true,
        })
    }

    #[inline]
    fn pop_loop_control_info(&mut self) {
        let loop_info = self.jump_info.pop().unwrap();

        assert!(loop_info.kind == JumpControlInfoKind::Loop);

        for label in loop_info.breaks {
            self.patch_jump(label);
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
            for_of_in_loop: false,
        })
    }

    #[inline]
    fn pop_switch_control_info(&mut self) {
        let info = self.jump_info.pop().unwrap();

        assert!(info.kind == JumpControlInfoKind::Switch);

        for label in info.breaks {
            self.patch_jump(label);
        }
    }

    #[inline]
    fn push_try_control_info(&mut self) {
        if !self.jump_info.is_empty() {
            let start_address = self.jump_info.last().unwrap().start_address;

            self.jump_info.push(JumpControlInfo {
                label: None,
                start_address,
                kind: JumpControlInfoKind::Try,
                breaks: Vec::new(),
                try_continues: Vec::new(),
                for_of_in_loop: false,
            })
        }
    }

    #[inline]
    fn pop_try_control_info(&mut self, finally_start_address: Option<u32>) {
        if !self.jump_info.is_empty() {
            let mut info = self.jump_info.pop().unwrap();

            assert!(info.kind == JumpControlInfoKind::Try);

            let mut breaks = Vec::with_capacity(info.breaks.len());

            if let Some(finally_start_address) = finally_start_address {
                for label in info.try_continues {
                    if label.index < finally_start_address {
                        self.patch_jump_with_target(label, finally_start_address);
                    } else {
                        self.patch_jump_with_target(label, info.start_address)
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
            }
        }
    }

    #[inline]
    fn compile_access<'a>(&mut self, node: &'a Node) -> Access<'a> {
        match node {
            Node::Identifier(name) => {
                let index = self.get_or_insert_name(
                    self.interner
                        .resolve(name.sym())
                        .expect("string disappeared"),
                );
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
                let index = self.get_or_insert_name(
                    self.interner
                        .resolve(node.field())
                        .expect("string disappeared"),
                );
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
                let index = self.get_or_insert_name(
                    self.interner
                        .resolve(node.field())
                        .expect("string disappeared"),
                );
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
                    Const::String(v) => self.emit_push_literal(Literal::String(
                        self.interner
                            .resolve(*v)
                            .expect("string disappeared")
                            .into(),
                    )),
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
                        self.access_set(access, None, true);
                        None
                    }
                    UnaryOp::DecrementPre => {
                        self.compile_expr(unary.target(), true);
                        self.emit(Opcode::Dec, &[]);

                        let access = self.compile_access(unary.target());
                        self.access_set(access, None, true);
                        None
                    }
                    UnaryOp::IncrementPost => {
                        self.compile_expr(unary.target(), true);
                        self.emit(Opcode::Dup, &[]);
                        self.emit(Opcode::Inc, &[]);
                        let access = self.compile_access(unary.target());
                        self.access_set(access, None, false);

                        None
                    }
                    UnaryOp::DecrementPost => {
                        self.compile_expr(unary.target(), true);
                        self.emit(Opcode::Dup, &[]);
                        self.emit(Opcode::Dec, &[]);
                        let access = self.compile_access(unary.target());
                        self.access_set(access, None, false);

                        None
                    }
                    UnaryOp::Delete => match unary.target() {
                        Node::GetConstField(ref get_const_field) => {
                            let index = self.get_or_insert_name(
                                self.interner
                                    .resolve(get_const_field.field())
                                    .expect("string disappeared"),
                            );
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
                    UnaryOp::TypeOf => {
                        match &unary.target() {
                            Node::Identifier(identifier) => {
                                let index = self.get_or_insert_name(
                                    self.interner
                                        .resolve(identifier.sym())
                                        .expect("string disappeared"),
                                );
                                self.emit(Opcode::GetNameOrUndefined, &[index]);
                            }
                            expr => self.compile_expr(expr, true),
                        }
                        self.emit_opcode(Opcode::TypeOf);
                        None
                    }
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
                                self.patch_jump(exit);
                            }
                            LogOp::Or => {
                                let exit = self.jump_with_custom_opcode(Opcode::LogicalOr);
                                self.compile_expr(binary.rhs(), true);
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
                                let access = self.compile_access(binary.lhs());
                                self.access_set(access, None, use_expr);
                                self.patch_jump(exit);
                                None
                            }
                            AssignOp::BoolOr => {
                                let exit = self.jump_with_custom_opcode(Opcode::LogicalOr);
                                self.compile_expr(binary.rhs(), true);
                                let access = self.compile_access(binary.lhs());
                                self.access_set(access, None, use_expr);
                                self.patch_jump(exit);
                                None
                            }
                            AssignOp::Coalesce => {
                                let exit = self.jump_with_custom_opcode(Opcode::Coalesce);
                                self.compile_expr(binary.rhs(), true);
                                let access = self.compile_access(binary.lhs());
                                self.access_set(access, None, use_expr);
                                self.patch_jump(exit);
                                None
                            }
                        };

                        if let Some(opcode) = opcode {
                            self.compile_expr(binary.rhs(), true);
                            self.emit(opcode, &[]);
                            let access = self.compile_access(binary.lhs());
                            self.access_set(access, None, use_expr);
                        }
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
                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                        }
                        PropertyDefinition::Property(name, node) => match name {
                            PropertyName::Literal(name) => {
                                self.compile_stmt(node, true);
                                self.emit_opcode(Opcode::Swap);
                                let name =
                                    self.interner.resolve(*name).expect("string disappeared");
                                let index = self.get_or_insert_name(name);
                                self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                            }
                            PropertyName::Computed(name_node) => {
                                self.compile_stmt(name_node, true);
                                self.compile_stmt(node, true);
                                self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                            }
                        },
                        PropertyDefinition::MethodDefinition(kind, name, func) => {
                            match kind {
                                MethodDefinitionKind::Get => match name {
                                    PropertyName::Literal(name) => {
                                        self.compile_stmt(&func.clone().into(), true);
                                        self.emit_opcode(Opcode::Swap);
                                        let name = self
                                            .interner
                                            .resolve(*name)
                                            .expect("string disappeared");
                                        let index = self.get_or_insert_name(name);
                                        self.emit(Opcode::SetPropertyGetterByName, &[index]);
                                    }
                                    PropertyName::Computed(name_node) => {
                                        self.compile_stmt(name_node, true);
                                        self.compile_stmt(&func.clone().into(), true);
                                        self.emit_opcode(Opcode::SetPropertyGetterByValue);
                                    }
                                },
                                MethodDefinitionKind::Set => match name {
                                    PropertyName::Literal(name) => {
                                        self.compile_stmt(&func.clone().into(), true);
                                        self.emit_opcode(Opcode::Swap);
                                        let name = self
                                            .interner
                                            .resolve(*name)
                                            .expect("string disappeared");
                                        let index = self.get_or_insert_name(name);
                                        self.emit(Opcode::SetPropertySetterByName, &[index]);
                                    }
                                    PropertyName::Computed(name_node) => {
                                        self.compile_stmt(name_node, true);
                                        self.compile_stmt(&func.clone().into(), true);
                                        self.emit_opcode(Opcode::SetPropertySetterByValue);
                                    }
                                },
                                MethodDefinitionKind::Ordinary => match name {
                                    PropertyName::Literal(name) => {
                                        self.compile_stmt(&func.clone().into(), true);
                                        self.emit_opcode(Opcode::Swap);
                                        let name = self
                                            .interner
                                            .resolve(*name)
                                            .expect("string disappeared");
                                        let index = self.get_or_insert_name(name);
                                        self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                                    }
                                    PropertyName::Computed(name_node) => {
                                        self.compile_stmt(name_node, true);
                                        self.compile_stmt(&func.clone().into(), true);
                                        self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                    }
                                },
                                MethodDefinitionKind::Generator => {
                                    // TODO: Implement generators
                                    match name {
                                        PropertyName::Literal(name) => {
                                            self.emit_opcode(Opcode::PushUndefined);
                                            self.emit_opcode(Opcode::Swap);
                                            let name = self
                                                .interner
                                                .resolve(*name)
                                                .expect("string disappeared");
                                            let index = self.get_or_insert_name(name);
                                            self.emit(Opcode::DefineOwnPropertyByName, &[index]);
                                        }
                                        PropertyName::Computed(name_node) => {
                                            self.compile_stmt(name_node, true);
                                            self.emit_opcode(Opcode::PushUndefined);
                                            self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                        }
                                    }
                                }
                                MethodDefinitionKind::Async => {
                                    // TODO: Implement async
                                    match name {
                                        PropertyName::Literal(name) => {
                                            self.emit_opcode(Opcode::PushUndefined);
                                            self.emit_opcode(Opcode::Swap);
                                            let name = self
                                                .interner
                                                .resolve(*name)
                                                .expect("string disappeared");
                                            let index = self.get_or_insert_name(name);
                                            self.emit(Opcode::DefineOwnPropertyByName, &[index])
                                        }
                                        PropertyName::Computed(name_node) => {
                                            self.compile_stmt(name_node, true);
                                            self.emit_opcode(Opcode::PushUndefined);
                                            self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                        }
                                    }
                                }
                                MethodDefinitionKind::AsyncGenerator => {
                                    // TODO: Implement async generators
                                    match name {
                                        PropertyName::Literal(name) => {
                                            self.emit_opcode(Opcode::PushUndefined);
                                            self.emit_opcode(Opcode::Swap);
                                            let name = self
                                                .interner
                                                .resolve(*name)
                                                .expect("string disappeared");
                                            let index = self.get_or_insert_name(name);
                                            self.emit(Opcode::DefineOwnPropertyByName, &[index])
                                        }
                                        PropertyName::Computed(name_node) => {
                                            self.compile_stmt(name_node, true);
                                            self.emit_opcode(Opcode::PushUndefined);
                                            self.emit_opcode(Opcode::DefineOwnPropertyByValue);
                                        }
                                    }
                                }
                            }
                        }
                        PropertyDefinition::SpreadObject(expr) => {
                            self.compile_expr(expr, true);
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
                let index = self.get_or_insert_name(
                    self.interner
                        .resolve(name.sym())
                        .expect("string disappeared"),
                );
                let access = Access::Variable { index };
                self.access_get(access, use_expr);
            }
            Node::Assign(assign) => {
                // Implement destructing assignments like here: https://tc39.es/ecma262/#sec-destructuring-assignment
                if let Node::Object(_) = assign.lhs() {
                    self.emit_opcode(Opcode::PushUndefined);
                } else {
                    let access = self.compile_access(assign.lhs());
                    self.access_set(access, Some(assign.rhs()), use_expr);
                }
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
                self.emit_opcode(Opcode::PushNewArray);
                self.emit_opcode(Opcode::PopOnReturnAdd);

                for element in array.as_ref() {
                    self.compile_expr(element, true);
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
                self.access_get(Access::This, use_expr);
            }
            Node::Spread(spread) => self.compile_expr(spread.val(), true),
            Node::FunctionExpr(_function) => self.function(expr, use_expr),
            Node::ArrowFunctionDecl(_function) => self.function(expr, use_expr),
            Node::Call(_) => self.call(expr, use_expr),
            Node::New(_) => self.call(expr, use_expr),
            Node::TemplateLit(template_literal) => {
                for element in template_literal.elements() {
                    match element {
                        TemplateElement::String(s) => self.emit_push_literal(Literal::String(
                            self.interner
                                .resolve(*s)
                                .expect("string disappeared")
                                .into(),
                        )),
                        TemplateElement::Expr(expr) => {
                            self.compile_expr(expr, true);
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
            Node::AsyncFunctionExpr(_) => {
                self.emit_opcode(Opcode::PushUndefined);
            }
            // TODO: implement AwaitExpr
            Node::AwaitExpr(_) => {
                self.emit_opcode(Opcode::PushUndefined);
            }
            // TODO: implement GeneratorExpr
            Node::GeneratorExpr(_) => {
                self.emit_opcode(Opcode::PushUndefined);
            }
            // TODO: implement AsyncGeneratorExpr
            Node::AsyncGeneratorExpr(_) => {
                self.emit_opcode(Opcode::PushUndefined);
            }
            // TODO: implement Yield
            Node::Yield(_) => {
                self.emit_opcode(Opcode::PushUndefined);
            }
            Node::TaggedTemplate(template) => {
                match template.tag() {
                    Node::GetConstField(field) => {
                        self.compile_expr(field.obj(), true);
                        self.emit(Opcode::Dup, &[]);
                        let index = self.get_or_insert_name(
                            self.interner
                                .resolve(field.field())
                                .expect("string disappeared"),
                        );
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
                        self.compile_expr(expr, true);
                        self.emit_opcode(Opcode::This);
                        self.emit_opcode(Opcode::Swap);
                    }
                }

                self.emit_opcode(Opcode::PushNewArray);
                for cooked in template.cookeds() {
                    if let Some(cooked) = cooked {
                        self.emit_push_literal(Literal::String(
                            self.interner
                                .resolve(*cooked)
                                .expect("string disappeared")
                                .into(),
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
                        self.interner
                            .resolve(*raw)
                            .expect("string disappeared")
                            .into(),
                    ));
                    self.emit_opcode(Opcode::PushValueToArray);
                }

                self.emit_opcode(Opcode::Swap);
                let index = self.get_or_insert_name("raw");
                self.emit(Opcode::SetPropertyByName, &[index]);

                for expr in template.exprs() {
                    self.compile_expr(expr, true);
                }

                self.emit(Opcode::Call, &[(template.exprs().len() + 1) as u32]);
            }
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn compile_stmt(&mut self, node: &Node, use_expr: bool) {
        match node {
            Node::VarDeclList(list) => {
                for decl in list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            let ident = self
                                .interner
                                .resolve(ident.sym())
                                .expect("string disappeared");
                            if ident == "arguments" {
                                self.code_block.lexical_name_argument = true;
                            }

                            let index = self.get_or_insert_name(ident);

                            if let Some(expr) = decl.init() {
                                self.compile_expr(expr, true);
                                self.emit(Opcode::DefInitVar, &[index]);
                            } else {
                                self.emit(Opcode::DefVar, &[index]);
                            }
                        }
                        Declaration::Pattern(pattern) => {
                            if pattern.idents().contains(&Sym::ARGUMENTS) {
                                self.code_block.lexical_name_argument = true;
                            }

                            if let Some(init) = decl.init() {
                                self.compile_expr(init, true);
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, Opcode::DefInitVar);
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

                            let index = self.get_or_insert_name(
                                self.interner
                                    .resolve(ident.sym())
                                    .expect("string disappeared"),
                            );

                            if let Some(expr) = decl.init() {
                                self.compile_expr(expr, true);
                                self.emit(Opcode::DefInitLet, &[index]);
                            } else {
                                self.emit(Opcode::DefLet, &[index]);
                            }
                        }
                        Declaration::Pattern(pattern) => {
                            if pattern.idents().contains(&Sym::ARGUMENTS) {
                                self.code_block.lexical_name_argument = true;
                            }

                            if let Some(init) = decl.init() {
                                self.compile_expr(init, true);
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, Opcode::DefInitLet);
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

                            let index = self.get_or_insert_name(
                                self.interner
                                    .resolve(ident.sym())
                                    .expect("string disappeared"),
                            );
                            let init = decl
                                .init()
                                .expect("const declaration must have initializer");
                            self.compile_expr(init, true);
                            self.emit(Opcode::DefInitConst, &[index]);
                        }
                        Declaration::Pattern(pattern) => {
                            if pattern.idents().contains(&Sym::ARGUMENTS) {
                                self.code_block.lexical_name_argument = true;
                            }

                            if let Some(init) = decl.init() {
                                self.compile_expr(init, true);
                            } else {
                                self.emit_opcode(Opcode::PushUndefined);
                            };

                            self.compile_declaration_pattern(pattern, Opcode::DefInitConst);
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
            Node::ForLoop(for_loop) => {
                self.emit_opcode(Opcode::PushDeclarativeEnvironment);

                if let Some(init) = for_loop.init() {
                    self.compile_stmt(init, false);
                }

                let initial_jump = self.jump();

                let start_address = self.next_opcode_location();
                self.push_loop_control_info(for_loop.label(), start_address);

                if let Some(final_expr) = for_loop.final_expr() {
                    self.compile_expr(final_expr, false);
                }

                self.patch_jump(initial_jump);

                if let Some(condition) = for_loop.condition() {
                    self.compile_expr(condition, true);
                } else {
                    self.emit_opcode(Opcode::PushTrue);
                }
                let exit = self.jump_if_false();

                self.compile_stmt(for_loop.body(), false);

                self.emit(Opcode::Jump, &[start_address]);

                self.patch_jump(exit);
                self.pop_loop_control_info();

                self.emit_opcode(Opcode::PopEnvironment);
            }
            Node::ForInLoop(for_in_loop) => {
                self.compile_expr(for_in_loop.expr(), true);
                let early_exit = self.jump_with_custom_opcode(Opcode::ForInLoopInitIterator);

                let start_address = self.next_opcode_location();
                self.push_loop_control_info_for_of_in_loop(for_in_loop.label(), start_address);

                self.emit_opcode(Opcode::PushDeclarativeEnvironment);
                let exit = self.jump_with_custom_opcode(Opcode::ForInLoopNext);

                match for_in_loop.init() {
                    IterableLoopInitializer::Identifier(ref ident) => {
                        let ident = self
                            .interner
                            .resolve(ident.sym())
                            .expect("string disappeared");
                        let index = self.get_or_insert_name(ident);
                        self.emit(Opcode::SetName, &[index]);
                    }
                    IterableLoopInitializer::Var(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            let ident = self
                                .interner
                                .resolve(ident.sym())
                                .expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(Opcode::DefInitVar, &[index]);
                        }
                        Declaration::Pattern(pattern) => {
                            self.compile_declaration_pattern(pattern, Opcode::DefInitVar);
                        }
                    },
                    IterableLoopInitializer::Let(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            let ident = self
                                .interner
                                .resolve(ident.sym())
                                .expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(Opcode::DefInitLet, &[index]);
                        }
                        Declaration::Pattern(pattern) => {
                            self.compile_declaration_pattern(pattern, Opcode::DefInitLet);
                        }
                    },
                    IterableLoopInitializer::Const(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            let ident = self
                                .interner
                                .resolve(ident.sym())
                                .expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(Opcode::DefInitConst, &[index]);
                        }
                        Declaration::Pattern(pattern) => {
                            self.compile_declaration_pattern(pattern, Opcode::DefInitConst);
                        }
                    },
                }

                self.compile_stmt(for_in_loop.body(), false);
                self.emit_opcode(Opcode::PopEnvironment);

                self.emit(Opcode::Jump, &[start_address]);

                self.patch_jump(exit);
                self.pop_loop_control_info();
                self.emit_opcode(Opcode::PushFalse);
                self.emit_opcode(Opcode::IteratorClose);

                self.patch_jump(early_exit);
            }
            Node::ForOfLoop(for_of_loop) => {
                self.compile_expr(for_of_loop.iterable(), true);
                self.emit_opcode(Opcode::InitIterator);

                let start_address = self.next_opcode_location();
                self.push_loop_control_info_for_of_in_loop(for_of_loop.label(), start_address);

                self.emit_opcode(Opcode::PushDeclarativeEnvironment);
                let exit = self.jump_with_custom_opcode(Opcode::ForInLoopNext);

                match for_of_loop.init() {
                    IterableLoopInitializer::Identifier(ref ident) => {
                        let ident = self
                            .interner
                            .resolve(ident.sym())
                            .expect("string disappeared");
                        let index = self.get_or_insert_name(ident);
                        self.emit(Opcode::SetName, &[index]);
                    }
                    IterableLoopInitializer::Var(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            let ident = self
                                .interner
                                .resolve(ident.sym())
                                .expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(Opcode::DefInitVar, &[index]);
                        }
                        Declaration::Pattern(pattern) => {
                            self.compile_declaration_pattern(pattern, Opcode::DefInitVar);
                        }
                    },
                    IterableLoopInitializer::Let(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            let ident = self
                                .interner
                                .resolve(ident.sym())
                                .expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(Opcode::DefInitLet, &[index]);
                        }
                        Declaration::Pattern(pattern) => {
                            self.compile_declaration_pattern(pattern, Opcode::DefInitLet);
                        }
                    },
                    IterableLoopInitializer::Const(declaration) => match declaration {
                        Declaration::Identifier { ident, .. } => {
                            let ident = self
                                .interner
                                .resolve(ident.sym())
                                .expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(Opcode::DefInitConst, &[index]);
                        }
                        Declaration::Pattern(pattern) => {
                            self.compile_declaration_pattern(pattern, Opcode::DefInitConst);
                        }
                    },
                }

                self.compile_stmt(for_of_loop.body(), false);
                self.emit_opcode(Opcode::PopEnvironment);

                self.emit(Opcode::Jump, &[start_address]);

                self.patch_jump(exit);
                self.pop_loop_control_info();
                self.emit_opcode(Opcode::PushFalse);
                self.emit_opcode(Opcode::IteratorClose);
            }
            Node::WhileLoop(while_) => {
                let start_address = self.next_opcode_location();
                self.push_loop_control_info(while_.label(), start_address);

                self.compile_expr(while_.cond(), true);
                let exit = self.jump_if_false();
                self.compile_stmt(while_.body(), false);
                self.emit(Opcode::Jump, &[start_address]);
                self.patch_jump(exit);

                self.pop_loop_control_info();
            }
            Node::DoWhileLoop(do_while) => {
                let initial_label = self.jump();

                let start_address = self.next_opcode_location();
                self.push_loop_control_info(do_while.label(), start_address);

                let condition_label_address = self.next_opcode_location();
                self.compile_expr(do_while.cond(), true);
                let exit = self.jump_if_false();

                self.patch_jump(initial_label);

                self.compile_stmt(do_while.body(), false);
                self.emit(Opcode::Jump, &[condition_label_address]);

                self.pop_loop_control_info();

                self.patch_jump(exit);
            }
            Node::Continue(node) => {
                if let Some(start_address) = self
                    .jump_info
                    .last()
                    .filter(|info| info.kind == JumpControlInfoKind::Try)
                    .map(|info| info.start_address)
                {
                    self.emit_opcode(Opcode::TryEnd);
                    self.emit(Opcode::FinallySetJump, &[start_address]);
                    let label = self.jump();
                    self.jump_info.last_mut().unwrap().try_continues.push(label);
                } else {
                    let mut items = self
                        .jump_info
                        .iter()
                        .rev()
                        .filter(|info| info.kind == JumpControlInfoKind::Loop);
                    let address = if node.label().is_none() {
                        items.next().expect("continue target").start_address
                    } else {
                        let mut emit_for_of_in_exit = 0;
                        let mut address_info = None;
                        for info in items {
                            if info.label == node.label() {
                                address_info = Some(info);
                                break;
                            }
                            if info.for_of_in_loop {
                                emit_for_of_in_exit += 1;
                            }
                        }
                        let address = address_info.expect("continue target").start_address;
                        for _ in 0..emit_for_of_in_exit {
                            self.emit_opcode(Opcode::PopEnvironment);
                            self.emit_opcode(Opcode::PopEnvironment);
                            self.emit_opcode(Opcode::Pop);
                            self.emit_opcode(Opcode::Pop);
                        }
                        address
                    };
                    self.emit(Opcode::Jump, &[address]);
                }
            }
            Node::Break(node) => {
                if self
                    .jump_info
                    .last()
                    .filter(|info| info.kind == JumpControlInfoKind::Try)
                    .is_some()
                {
                    self.emit_opcode(Opcode::TryEnd);
                    self.emit(Opcode::FinallySetJump, &[u32::MAX]);
                }
                let label = self.jump();
                if node.label().is_none() {
                    self.jump_info.last_mut().unwrap().breaks.push(label);
                } else {
                    for info in self.jump_info.iter_mut().rev() {
                        if info.label == node.label() {
                            info.breaks.push(label);
                            break;
                        }
                    }
                }
            }
            Node::Block(block) => {
                self.emit_opcode(Opcode::PushDeclarativeEnvironment);
                for node in block.items() {
                    self.compile_stmt(node, use_expr);
                }
                self.emit_opcode(Opcode::PopEnvironment);
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
            Node::Try(t) => {
                self.push_try_control_info();

                let try_start = self.next_opcode_location();
                self.emit(Opcode::TryStart, &[Self::DUMMY_ADDRESS, 0]);
                self.emit_opcode(Opcode::PushDeclarativeEnvironment);
                for node in t.block().items() {
                    self.compile_stmt(node, false);
                }
                self.emit_opcode(Opcode::PopEnvironment);
                self.emit_opcode(Opcode::TryEnd);

                let finally = self.jump();
                self.patch_jump(Label { index: try_start });

                if let Some(catch) = t.catch() {
                    let catch_start = if t.finally().is_some() {
                        Some(self.jump_with_custom_opcode(Opcode::CatchStart))
                    } else {
                        None
                    };
                    self.emit_opcode(Opcode::PushDeclarativeEnvironment);
                    if let Some(decl) = catch.parameter() {
                        match decl {
                            Declaration::Identifier { ident, .. } => {
                                let ident = self
                                    .interner
                                    .resolve(ident.sym())
                                    .expect("string disappeared");
                                let index = self.get_or_insert_name(ident);
                                self.emit(Opcode::DefInitLet, &[index]);
                            }
                            Declaration::Pattern(pattern) => {
                                self.compile_declaration_pattern(pattern, Opcode::DefInitLet);
                            }
                        }
                    } else {
                        self.emit_opcode(Opcode::Pop);
                    }
                    for node in catch.block().items() {
                        self.compile_stmt(node, use_expr);
                    }
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
                    self.patch_jump_with_target(
                        Label {
                            index: try_start + 4,
                        },
                        finally_start_address,
                    );

                    for node in finally.items() {
                        self.compile_stmt(node, false);
                    }
                    self.emit_opcode(Opcode::FinallyEnd);
                    self.pop_try_control_info(Some(finally_start_address));
                } else {
                    self.pop_try_control_info(None);
                }
            }
            // TODO: implement AsyncFunctionDecl
            Node::AsyncFunctionDecl(_) => {
                self.emit_opcode(Opcode::PushUndefined);
            }
            // TODO: implement GeneratorDecl
            Node::GeneratorDecl(_) => {
                self.emit_opcode(Opcode::PushUndefined);
            }
            // TODO: implement AsyncGeneratorDecl
            Node::AsyncGeneratorDecl(_) => {
                self.emit_opcode(Opcode::PushUndefined);
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

        let strict = body.strict() || self.code_block.strict;
        let length = parameters.len() as u32;
        let mut code = CodeBlock::new(name.unwrap_or(Sym::EMPTY_STRING), length, strict, true);

        if let FunctionKind::Arrow = kind {
            code.constructor = false;
            code.this_mode = ThisMode::Lexical;
        }

        let mut compiler = ByteCompiler {
            code_block: code,
            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            jump_info: Vec::new(),
            interner: self.interner,
        };

        let mut has_rest_parameter = false;
        let mut has_parameter_expressions = false;
        for parameter in parameters {
            has_parameter_expressions = has_parameter_expressions || parameter.init().is_some();

            if parameter.is_rest_param() {
                has_rest_parameter = true;
                compiler.emit_opcode(Opcode::RestParameterInit);
            }

            match parameter.declaration() {
                Declaration::Identifier { ident, .. } => {
                    let ident = self
                        .interner
                        .resolve(ident.sym())
                        .expect("string disappeared");
                    let index = compiler.get_or_insert_name(ident);
                    if let Some(init) = parameter.declaration().init() {
                        let skip = compiler.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                        compiler.compile_expr(init, true);
                        compiler.patch_jump(skip);
                    }
                    compiler.emit(Opcode::DefInitArg, &[index]);
                }
                Declaration::Pattern(pattern) => {
                    compiler.compile_declaration_pattern(pattern, Opcode::DefInitArg);
                }
            }
        }

        if !has_rest_parameter {
            compiler.emit_opcode(Opcode::RestParameterPop);
        }

        if has_parameter_expressions {
            compiler.emit_opcode(Opcode::PushFunctionEnvironment)
        }

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
                let index = self.get_or_insert_name(
                    self.interner
                        .resolve(name.unwrap())
                        .expect("string disappeared"),
                );
                self.emit(Opcode::DefInitVar, &[index]);
            }
            FunctionKind::Expression | FunctionKind::Arrow => {
                if !use_expr {
                    self.emit(Opcode::Pop, &[]);
                }
            }
        }
    }

    pub(crate) fn call(&mut self, node: &Node, use_expr: bool) {
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
                self.compile_expr(field.obj(), true);
                self.emit(Opcode::Dup, &[]);
                let index = self.get_or_insert_name(
                    self.interner
                        .resolve(field.field())
                        .expect("string disappeared"),
                );
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
                self.compile_expr(expr, true);
                if kind == CallKind::Call {
                    self.emit_opcode(Opcode::This);
                    self.emit_opcode(Opcode::Swap);
                }
            }
        }

        for arg in call.args().iter() {
            self.compile_expr(arg, true);
        }

        let last_is_rest_parameter = matches!(call.args().last(), Some(Node::Spread(_)));

        match kind {
            CallKind::Call if last_is_rest_parameter => {
                self.emit(Opcode::CallWithRest, &[call.args().len() as u32])
            }
            CallKind::Call => self.emit(Opcode::Call, &[call.args().len() as u32]),
            CallKind::New if last_is_rest_parameter => {
                self.emit(Opcode::NewWithRest, &[call.args().len() as u32])
            }
            CallKind::New => self.emit(Opcode::New, &[call.args().len() as u32]),
        }

        if !use_expr {
            self.emit(Opcode::Pop, &[]);
        }
    }

    #[inline]
    pub fn finish(self) -> CodeBlock {
        self.code_block
    }

    #[inline]
    fn compile_declaration_pattern(&mut self, pattern: &DeclarationPattern, def: Opcode) {
        match pattern {
            DeclarationPattern::Object(pattern) => {
                let skip_init = self.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                if let Some(init) = pattern.init() {
                    self.compile_expr(init, true);
                } else {
                    self.emit_opcode(Opcode::PushUndefined);
                }
                self.patch_jump(skip_init);
                self.emit_opcode(Opcode::ValueNotNullOrUndefined);

                self.emit_opcode(Opcode::RequireObjectCoercible);

                for binding in pattern.bindings() {
                    use BindingPatternTypeObject::*;

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
                            let index = self.get_or_insert_name(
                                self.interner
                                    .resolve(*property_name)
                                    .expect("string disappeared"),
                            );
                            self.emit(Opcode::GetPropertyByName, &[index]);

                            if let Some(init) = default_init {
                                let skip = self.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true);
                                self.patch_jump(skip);
                            }

                            let ident = self.interner.resolve(*ident).expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(def, &[index]);
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
                                    self.interner.resolve(*key).expect("get_or_intern").into(),
                                ));
                            }

                            self.emit(Opcode::CopyDataProperties, &[excluded_keys.len() as u32]);

                            let ident = self.interner.resolve(*ident).expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(def, &[index]);
                        }
                        BindingPattern {
                            ident,
                            pattern,
                            default_init,
                        } => {
                            self.emit_opcode(Opcode::Dup);
                            let ident = self.interner.resolve(*ident).expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(Opcode::GetPropertyByName, &[index]);

                            if let Some(init) = default_init {
                                let skip = self.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                                self.compile_expr(init, true);
                                self.patch_jump(skip);
                            }

                            self.compile_declaration_pattern(pattern, def);
                        }
                    }
                }

                self.emit_opcode(Opcode::Pop);
            }
            DeclarationPattern::Array(pattern) => {
                let skip_init = self.jump_with_custom_opcode(Opcode::JumpIfNotUndefined);
                if let Some(init) = pattern.init() {
                    self.compile_expr(init, true);
                } else {
                    self.emit_opcode(Opcode::PushUndefined);
                }
                self.patch_jump(skip_init);
                self.emit_opcode(Opcode::ValueNotNullOrUndefined);
                self.emit_opcode(Opcode::InitIterator);

                for (i, binding) in pattern.bindings().iter().enumerate() {
                    use BindingPatternTypeArray::*;

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
                                self.compile_expr(init, true);
                                self.patch_jump(skip);
                            }

                            let ident = self.interner.resolve(*ident).expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(def, &[index]);
                        }
                        // BindingElement : BindingPattern Initializer[opt]
                        BindingPattern { pattern } => {
                            self.emit_opcode(next);
                            self.compile_declaration_pattern(pattern, def)
                        }
                        // BindingRestElement : ... BindingIdentifier
                        SingleNameRest { ident } => {
                            self.emit_opcode(Opcode::IteratorToArray);

                            let ident = self.interner.resolve(*ident).expect("string disappeared");
                            let index = self.get_or_insert_name(ident);
                            self.emit(def, &[index]);
                            self.emit_opcode(Opcode::PushTrue);
                        }
                        // BindingRestElement : ... BindingPattern
                        BindingPatternRest { pattern } => {
                            self.emit_opcode(Opcode::IteratorToArray);
                            self.compile_declaration_pattern(pattern, def);
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
    }
}
