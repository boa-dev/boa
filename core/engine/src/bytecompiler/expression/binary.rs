use crate::bytecompiler::{ByteCompiler, Label, Register};
use boa_ast::{
    Expression,
    expression::operator::{
        Binary, BinaryInPrivate,
        binary::{ArithmeticOp, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
    },
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_binary(&mut self, binary: &Binary, dst: &Register) {
        self.compile_expr(binary.lhs(), dst);

        match binary.op() {
            BinaryOp::Arithmetic(op) => self.compile_binary_arithmetic(op, binary.rhs(), dst),
            BinaryOp::Bitwise(op) => self.compile_binary_bitwise(op, binary.rhs(), dst),
            BinaryOp::Relational(op) => self.compile_binary_relational(op, binary.rhs(), dst),
            BinaryOp::Logical(op) => {
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
                self.compile_expr(binary.rhs(), dst);
            }
        }
    }

    fn compile_binary_arithmetic(&mut self, op: ArithmeticOp, expr: &Expression, dst: &Register) {
        let rhs = self.register_allocator.alloc();
        self.compile_expr(expr, &rhs);
        let bytecode = &mut self.bytecode;
        match op {
            ArithmeticOp::Add => bytecode.emit_add(dst.variable(), dst.variable(), rhs.variable()),
            ArithmeticOp::Sub => bytecode.emit_sub(dst.variable(), dst.variable(), rhs.variable()),
            ArithmeticOp::Div => bytecode.emit_div(dst.variable(), dst.variable(), rhs.variable()),
            ArithmeticOp::Mul => bytecode.emit_mul(dst.variable(), dst.variable(), rhs.variable()),
            ArithmeticOp::Exp => bytecode.emit_pow(dst.variable(), dst.variable(), rhs.variable()),
            ArithmeticOp::Mod => bytecode.emit_mod(dst.variable(), dst.variable(), rhs.variable()),
        }
        self.register_allocator.dealloc(rhs);
    }

    fn compile_binary_bitwise(&mut self, op: BitwiseOp, expr: &Expression, dst: &Register) {
        let rhs = self.register_allocator.alloc();
        self.compile_expr(expr, &rhs);
        let bytecode = &mut self.bytecode;
        match op {
            BitwiseOp::And => bytecode.emit_bit_and(dst.variable(), dst.variable(), rhs.variable()),
            BitwiseOp::Or => bytecode.emit_bit_or(dst.variable(), dst.variable(), rhs.variable()),
            BitwiseOp::Xor => bytecode.emit_bit_xor(dst.variable(), dst.variable(), rhs.variable()),
            BitwiseOp::Shl => {
                bytecode.emit_shift_left(dst.variable(), dst.variable(), rhs.variable());
            }
            BitwiseOp::Shr => {
                bytecode.emit_shift_right(dst.variable(), dst.variable(), rhs.variable());
            }
            BitwiseOp::UShr => {
                bytecode.emit_unsigned_shift_right(dst.variable(), dst.variable(), rhs.variable());
            }
        }
        self.register_allocator.dealloc(rhs);
    }

    fn compile_binary_relational(&mut self, op: RelationalOp, expr: &Expression, dst: &Register) {
        let rhs = self.register_allocator.alloc();
        self.compile_expr(expr, &rhs);
        let bytecode = &mut self.bytecode;
        match op {
            RelationalOp::Equal => bytecode.emit_eq(dst.variable(), dst.variable(), rhs.variable()),
            RelationalOp::NotEqual => {
                bytecode.emit_not_eq(dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::StrictEqual => {
                bytecode.emit_strict_eq(dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::StrictNotEqual => {
                bytecode.emit_strict_not_eq(dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::GreaterThan => {
                bytecode.emit_greater_than(dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::GreaterThanOrEqual => {
                bytecode.emit_greater_than_or_eq(dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::LessThan => {
                bytecode.emit_less_than(dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::LessThanOrEqual => {
                bytecode.emit_less_than_or_eq(dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::In => bytecode.emit_in(dst.variable(), dst.variable(), rhs.variable()),
            RelationalOp::InstanceOf => {
                bytecode.emit_instance_of(dst.variable(), dst.variable(), rhs.variable());
            }
        }
        self.register_allocator.dealloc(rhs);
    }

    pub(crate) fn compile_binary_in_private(&mut self, binary: &BinaryInPrivate, dst: &Register) {
        let index = self.get_or_insert_private_name(*binary.lhs());
        self.compile_expr(binary.rhs(), dst);
        self.bytecode
            .emit_in_private(dst.variable(), index.into(), dst.variable());
    }
}
