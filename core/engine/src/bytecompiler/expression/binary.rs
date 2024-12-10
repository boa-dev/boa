use boa_ast::expression::operator::{
    binary::{ArithmeticOp, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
    Binary, BinaryInPrivate,
};

use crate::{
    bytecompiler::{ByteCompiler, Operand, Register},
    vm::Opcode,
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_binary(&mut self, binary: &Binary, dst: &Register) {
        self.compile_expr(binary.lhs(), dst);

        match binary.op() {
            BinaryOp::Arithmetic(op) => {
                let rhs = self.register_allocator.alloc();
                self.compile_expr(binary.rhs(), &rhs);

                let opcode = match op {
                    ArithmeticOp::Add => Opcode::Add,
                    ArithmeticOp::Sub => Opcode::Sub,
                    ArithmeticOp::Div => Opcode::Div,
                    ArithmeticOp::Mul => Opcode::Mul,
                    ArithmeticOp::Exp => Opcode::Pow,
                    ArithmeticOp::Mod => Opcode::Mod,
                };

                self.emit(
                    opcode,
                    &[
                        Operand::Register(dst),
                        Operand::Register(dst),
                        Operand::Register(&rhs),
                    ],
                );

                self.register_allocator.dealloc(rhs);
            }
            BinaryOp::Bitwise(op) => {
                let rhs = self.register_allocator.alloc();
                self.compile_expr(binary.rhs(), &rhs);

                let opcode = match op {
                    BitwiseOp::And => Opcode::BitAnd,
                    BitwiseOp::Or => Opcode::BitOr,
                    BitwiseOp::Xor => Opcode::BitXor,
                    BitwiseOp::Shl => Opcode::ShiftLeft,
                    BitwiseOp::Shr => Opcode::ShiftRight,
                    BitwiseOp::UShr => Opcode::UnsignedShiftRight,
                };

                self.emit(
                    opcode,
                    &[
                        Operand::Register(dst),
                        Operand::Register(dst),
                        Operand::Register(&rhs),
                    ],
                );

                self.register_allocator.dealloc(rhs);
            }
            BinaryOp::Relational(op) => {
                let rhs = self.register_allocator.alloc();
                self.compile_expr(binary.rhs(), &rhs);

                let opcode = match op {
                    RelationalOp::Equal => Opcode::Eq,
                    RelationalOp::NotEqual => Opcode::NotEq,
                    RelationalOp::StrictEqual => Opcode::StrictEq,
                    RelationalOp::StrictNotEqual => Opcode::StrictNotEq,
                    RelationalOp::GreaterThan => Opcode::GreaterThan,
                    RelationalOp::GreaterThanOrEqual => Opcode::GreaterThanOrEq,
                    RelationalOp::LessThan => Opcode::LessThan,
                    RelationalOp::LessThanOrEqual => Opcode::LessThanOrEq,
                    RelationalOp::In => Opcode::In,
                    RelationalOp::InstanceOf => Opcode::InstanceOf,
                };

                self.emit(
                    opcode,
                    &[
                        Operand::Register(dst),
                        Operand::Register(dst),
                        Operand::Register(&rhs),
                    ],
                );

                self.register_allocator.dealloc(rhs);
            }
            BinaryOp::Logical(op) => {
                let opcode = match op {
                    LogicalOp::And => Opcode::LogicalAnd,
                    LogicalOp::Or => Opcode::LogicalOr,
                    LogicalOp::Coalesce => Opcode::Coalesce,
                };

                let exit = self.emit_with_label(opcode, &[Operand::Register(dst)]);
                self.compile_expr(binary.rhs(), dst);
                self.patch_jump(exit);
            }
            BinaryOp::Comma => {
                self.compile_expr(binary.rhs(), dst);
            }
        }
    }

    pub(crate) fn compile_binary_in_private(&mut self, binary: &BinaryInPrivate, dst: &Register) {
        let index = self.get_or_insert_private_name(*binary.lhs());
        self.compile_expr(binary.rhs(), dst);
        self.emit(
            Opcode::InPrivate,
            &[
                Operand::Register(dst),
                Operand::Varying(index),
                Operand::Register(dst),
            ],
        );
    }
}
