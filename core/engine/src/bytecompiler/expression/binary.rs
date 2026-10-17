use crate::{
    bytecompiler::{ByteCompiler, Label, Register},
    vm::opcode::RegisterOperand,
};
use boa_ast::{
    Expression,
    expression::{
        literal::LiteralKind,
        operator::{
            Binary, BinaryInPrivate,
            binary::{ArithmeticOp, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
        },
    },
};

fn is_side_effect_free(expr: &Expression) -> bool {
    matches!(
        expr.flatten(),
        Expression::Literal(_) | Expression::Identifier(_)
    )
}

impl ByteCompiler<'_> {
    /// Compiles binary operands in strict left-to-right order.
    ///
    /// If `rhs` can produce side effects, `lhs` is evaluated into an independent
    /// temporary register to prevent mutations on the right operand from corrupting
    /// the left operand's value.
    pub(crate) fn compile_binary_operands(
        &mut self,
        lhs: &Expression,
        rhs: &Expression,
        inner: impl FnOnce(&mut Self, RegisterOperand, RegisterOperand),
    ) {
        if is_side_effect_free(rhs) {
            self.compile_expr_operand(lhs, |self_, lhs_op| {
                self_.compile_expr_operand(rhs, |self_, rhs_op| {
                    inner(self_, lhs_op, rhs_op);
                });
            });
        } else {
            let lhs_reg = self.register_allocator.alloc();
            self.compile_expr(lhs, &lhs_reg);
            self.compile_expr_operand(rhs, |self_, rhs_op| {
                inner(self_, lhs_reg.variable(), rhs_op);
            });
            self.register_allocator.dealloc(lhs_reg);
        }
    }

    pub(crate) fn compile_binary(&mut self, binary: &Binary, dst: &Register) {
        match binary.op() {
            BinaryOp::Arithmetic(op) => {
                self.compile_binary_operands(binary.lhs(), binary.rhs(), |self_, lhs, rhs| {
                    let bytecode = &mut self_.bytecode;
                    match op {
                        ArithmeticOp::Add => bytecode.emit_add(dst.variable(), lhs, rhs),
                        ArithmeticOp::Sub => bytecode.emit_sub(dst.variable(), lhs, rhs),
                        ArithmeticOp::Div => bytecode.emit_div(dst.variable(), lhs, rhs),
                        ArithmeticOp::Mul => bytecode.emit_mul(dst.variable(), lhs, rhs),
                        ArithmeticOp::Exp => bytecode.emit_pow(dst.variable(), lhs, rhs),
                        ArithmeticOp::Mod => bytecode.emit_mod(dst.variable(), lhs, rhs),
                    }
                });
            }
            BinaryOp::Bitwise(op) => {
                const MAX_UINT32_LITERAL: LiteralKind = LiteralKind::Num(u32::MAX as f64);
                if let Expression::Literal(literal) = binary.rhs().flatten()
                    && (
                        // x | 0
                        (op == BitwiseOp::Or && literal.kind() == &LiteralKind::Int(0))
                            ||
                            // x & 0xFFFFFFFF
                            (op == BitwiseOp::And && literal.kind() == &MAX_UINT32_LITERAL)
                    )
                {
                    self.compile_expr_operand(binary.lhs(), |self_, lhs| {
                        self_.bytecode.emit_to_int32(dst.variable(), lhs);
                    });
                } else {
                    self.compile_binary_operands(binary.lhs(), binary.rhs(), |self_, lhs, rhs| {
                        let bytecode = &mut self_.bytecode;
                        match op {
                            BitwiseOp::And => bytecode.emit_bit_and(dst.variable(), lhs, rhs),
                            BitwiseOp::Or => bytecode.emit_bit_or(dst.variable(), lhs, rhs),
                            BitwiseOp::Xor => bytecode.emit_bit_xor(dst.variable(), lhs, rhs),
                            BitwiseOp::Shl => bytecode.emit_shift_left(dst.variable(), lhs, rhs),
                            BitwiseOp::Shr => bytecode.emit_shift_right(dst.variable(), lhs, rhs),
                            BitwiseOp::UShr => {
                                bytecode.emit_unsigned_shift_right(dst.variable(), lhs, rhs);
                            }
                        }
                    });
                }
            }
            BinaryOp::Relational(op) => {
                self.compile_binary_operands(binary.lhs(), binary.rhs(), |self_, lhs, rhs| {
                    let bytecode = &mut self_.bytecode;
                    match op {
                        RelationalOp::Equal => bytecode.emit_eq(dst.variable(), lhs, rhs),
                        RelationalOp::NotEqual => bytecode.emit_not_eq(dst.variable(), lhs, rhs),
                        RelationalOp::StrictEqual => {
                            bytecode.emit_strict_eq(dst.variable(), lhs, rhs);
                        }
                        RelationalOp::StrictNotEqual => {
                            bytecode.emit_strict_not_eq(dst.variable(), lhs, rhs);
                        }
                        RelationalOp::GreaterThan => {
                            bytecode.emit_greater_than(dst.variable(), lhs, rhs);
                        }
                        RelationalOp::GreaterThanOrEqual => {
                            bytecode.emit_greater_than_or_eq(dst.variable(), lhs, rhs);
                        }
                        RelationalOp::LessThan => bytecode.emit_less_than(dst.variable(), lhs, rhs),
                        RelationalOp::LessThanOrEqual => {
                            bytecode.emit_less_than_or_eq(dst.variable(), lhs, rhs);
                        }
                        RelationalOp::In => bytecode.emit_in(dst.variable(), lhs, rhs),
                        RelationalOp::InstanceOf => {
                            bytecode.emit_instance_of(dst.variable(), lhs, rhs);
                        }
                    }
                });
            }
            BinaryOp::Logical(op) => {
                self.compile_expr(binary.lhs(), dst);
                let exit = self.next_opcode_location();
                match op {
                    LogicalOp::And => self
                        .bytecode
                        .emit_logical_and(Self::DUMMY_ADDRESS, dst.variable()),
                    LogicalOp::Or => self
                        .bytecode
                        .emit_logical_or(Self::DUMMY_ADDRESS, dst.variable()),
                    LogicalOp::Coalesce => self
                        .bytecode
                        .emit_coalesce(Self::DUMMY_ADDRESS, dst.variable()),
                }
                self.compile_expr(binary.rhs(), dst);
                self.patch_jump(Label { index: exit });
            }
            BinaryOp::Comma => {
                // Evaluate LHS for side effects, then RHS is the result.
                self.compile_expr_operand(binary.lhs(), |_, _| {});
                self.compile_expr(binary.rhs(), dst);
            }
        }
    }

    pub(crate) fn compile_binary_in_private(&mut self, binary: &BinaryInPrivate, dst: &Register) {
        let index = self.get_or_insert_private_name(*binary.lhs());
        self.compile_expr(binary.rhs(), dst);
        self.bytecode
            .emit_in_private(dst.variable(), index.into(), dst.variable());
    }
}
