use boa_ast::expression::operator::{
    binary::{ArithmeticOp, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
    Binary, BinaryInPrivate,
};

use crate::{
    bytecompiler::{ByteCompiler, InstructionOperand, Operand2, Reg},
    vm::Opcode,
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_binary(&mut self, binary: &Binary, dst: &Reg) {
        self.compile_expr(binary.lhs(), true);

        match binary.op() {
            BinaryOp::Arithmetic(op) => {
                self.compile_expr(binary.rhs(), true);

                let rhs = self.register_allocator.alloc();
                let lhs = self.register_allocator.alloc();

                self.pop_into_register(&rhs);
                self.pop_into_register(&lhs);
                let opcode = match op {
                    ArithmeticOp::Add => Opcode::Add,
                    ArithmeticOp::Sub => Opcode::Sub,
                    ArithmeticOp::Div => Opcode::Div,
                    ArithmeticOp::Mul => Opcode::Mul,
                    ArithmeticOp::Exp => Opcode::Pow,
                    ArithmeticOp::Mod => Opcode::Mod,
                };

                self.emit2(
                    opcode,
                    &[
                        Operand2::Register(dst),
                        Operand2::Operand(InstructionOperand::Register(&lhs)),
                        Operand2::Operand(InstructionOperand::Register(&rhs)),
                    ],
                );
                self.register_allocator.dealloc(lhs);
                self.register_allocator.dealloc(rhs);
            }
            BinaryOp::Bitwise(op) => {
                self.compile_expr(binary.rhs(), true);

                let rhs = self.register_allocator.alloc();
                let lhs = self.register_allocator.alloc();

                self.pop_into_register(&rhs);
                self.pop_into_register(&lhs);
                let opcode = match op {
                    BitwiseOp::And => Opcode::BitAnd,
                    BitwiseOp::Or => Opcode::BitOr,
                    BitwiseOp::Xor => Opcode::BitXor,
                    BitwiseOp::Shl => Opcode::ShiftLeft,
                    BitwiseOp::Shr => Opcode::ShiftRight,
                    BitwiseOp::UShr => Opcode::UnsignedShiftRight,
                };

                self.emit2(
                    opcode,
                    &[
                        Operand2::Register(dst),
                        Operand2::Operand(InstructionOperand::Register(&lhs)),
                        Operand2::Operand(InstructionOperand::Register(&rhs)),
                    ],
                );
                self.register_allocator.dealloc(lhs);
                self.register_allocator.dealloc(rhs);
            }
            BinaryOp::Relational(op) => {
                self.compile_expr(binary.rhs(), true);

                let rhs = self.register_allocator.alloc();
                let lhs = self.register_allocator.alloc();

                self.pop_into_register(&rhs);
                self.pop_into_register(&lhs);
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

                self.emit2(
                    opcode,
                    &[
                        Operand2::Register(dst),
                        Operand2::Operand(InstructionOperand::Register(&lhs)),
                        Operand2::Operand(InstructionOperand::Register(&rhs)),
                    ],
                );
                self.register_allocator.dealloc(lhs);
                self.register_allocator.dealloc(rhs);
            }
            BinaryOp::Logical(op) => {
                self.pop_into_register(dst);

                let opcode = match op {
                    LogicalOp::And => Opcode::LogicalAnd,
                    LogicalOp::Or => Opcode::LogicalOr,
                    LogicalOp::Coalesce => Opcode::Coalesce,
                };

                let exit =
                    self.emit_opcode_with_operand2(opcode, InstructionOperand::Register(dst));
                self.compile_expr(binary.rhs(), true);
                self.pop_into_register(dst);
                self.patch_jump(exit);
            }
            BinaryOp::Comma => {
                self.emit_opcode(Opcode::Pop);
                self.compile_expr(binary.rhs(), true);
                self.pop_into_register(dst);
            }
        }
    }

    pub(crate) fn compile_binary_in_private(&mut self, binary: &BinaryInPrivate, dst: &Reg) {
        let index = self.get_or_insert_private_name(*binary.lhs());
        self.compile_expr(binary.rhs(), true);
        self.pop_into_register(dst);
        self.emit2(
            Opcode::InPrivate,
            &[
                Operand2::Register(dst),
                Operand2::Varying(index),
                Operand2::Operand(InstructionOperand::Register(dst)),
            ],
        );
    }
}
