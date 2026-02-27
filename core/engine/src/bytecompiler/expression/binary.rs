use crate::bytecompiler::{ByteCompiler, Label, Register};
use crate::vm::opcode::{
    Add, BitAnd, BitOr, BitXor, Coalesce, Div, Eq, GreaterThan, GreaterThanOrEq, In, InPrivate,
    InstanceOf, LessThan, LessThanOrEq, LogicalAnd, LogicalOr, Mod, Mul, NotEq, Pow, ShiftLeft,
    ShiftRight, StrictEq, StrictNotEq, Sub, UnsignedShiftRight,
};
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
                    LogicalOp::And => {
                        LogicalAnd::emit(self, Self::DUMMY_ADDRESS, dst.variable());
                    }
                    LogicalOp::Or => {
                        LogicalOr::emit(self, Self::DUMMY_ADDRESS, dst.variable());
                    }
                    LogicalOp::Coalesce => {
                        Coalesce::emit(self, Self::DUMMY_ADDRESS, dst.variable());
                    }
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
        match op {
            ArithmeticOp::Add => Add::emit(self, dst.variable(), dst.variable(), rhs.variable()),
            ArithmeticOp::Sub => Sub::emit(self, dst.variable(), dst.variable(), rhs.variable()),
            ArithmeticOp::Div => Div::emit(self, dst.variable(), dst.variable(), rhs.variable()),
            ArithmeticOp::Mul => Mul::emit(self, dst.variable(), dst.variable(), rhs.variable()),
            ArithmeticOp::Exp => Pow::emit(self, dst.variable(), dst.variable(), rhs.variable()),
            ArithmeticOp::Mod => Mod::emit(self, dst.variable(), dst.variable(), rhs.variable()),
        }
        self.register_allocator.dealloc(rhs);
    }

    fn compile_binary_bitwise(&mut self, op: BitwiseOp, expr: &Expression, dst: &Register) {
        let rhs = self.register_allocator.alloc();
        self.compile_expr(expr, &rhs);
        match op {
            BitwiseOp::And => BitAnd::emit(self, dst.variable(), dst.variable(), rhs.variable()),
            BitwiseOp::Or => BitOr::emit(self, dst.variable(), dst.variable(), rhs.variable()),
            BitwiseOp::Xor => BitXor::emit(self, dst.variable(), dst.variable(), rhs.variable()),
            BitwiseOp::Shl => {
                ShiftLeft::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
            BitwiseOp::Shr => {
                ShiftRight::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
            BitwiseOp::UShr => {
                UnsignedShiftRight::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
        }
        self.register_allocator.dealloc(rhs);
    }

    fn compile_binary_relational(&mut self, op: RelationalOp, expr: &Expression, dst: &Register) {
        let rhs = self.register_allocator.alloc();
        self.compile_expr(expr, &rhs);
        match op {
            RelationalOp::Equal => Eq::emit(self, dst.variable(), dst.variable(), rhs.variable()),
            RelationalOp::NotEqual => {
                NotEq::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::StrictEqual => {
                StrictEq::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::StrictNotEqual => {
                StrictNotEq::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::GreaterThan => {
                GreaterThan::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::GreaterThanOrEqual => {
                GreaterThanOrEq::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::LessThan => {
                LessThan::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::LessThanOrEqual => {
                LessThanOrEq::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
            RelationalOp::In => In::emit(self, dst.variable(), dst.variable(), rhs.variable()),
            RelationalOp::InstanceOf => {
                InstanceOf::emit(self, dst.variable(), dst.variable(), rhs.variable());
            }
        }
        self.register_allocator.dealloc(rhs);
    }

    pub(crate) fn compile_binary_in_private(&mut self, binary: &BinaryInPrivate, dst: &Register) {
        let index = self.get_or_insert_private_name(*binary.lhs());
        self.compile_expr(binary.rhs(), dst);
        InPrivate::emit(self, dst.variable(), index.into(), dst.variable());
    }
}
