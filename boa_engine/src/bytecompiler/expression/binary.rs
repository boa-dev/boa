use boa_ast::expression::operator::{
    binary::{ArithmeticOp, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
    Binary,
};

use crate::{bytecompiler::ByteCompiler, vm::Opcode, JsResult};

pub(crate) fn compile_binary(
    byte_compiler: &mut ByteCompiler<'_>,
    binary: &Binary,
    use_expr: bool,
) -> JsResult<()> {
    byte_compiler.compile_expr(binary.lhs(), true)?;
    match binary.op() {
        BinaryOp::Arithmetic(op) => {
            byte_compiler.compile_expr(binary.rhs(), true)?;
            match op {
                ArithmeticOp::Add => byte_compiler.emit_opcode(Opcode::Add),
                ArithmeticOp::Sub => byte_compiler.emit_opcode(Opcode::Sub),
                ArithmeticOp::Div => byte_compiler.emit_opcode(Opcode::Div),
                ArithmeticOp::Mul => byte_compiler.emit_opcode(Opcode::Mul),
                ArithmeticOp::Exp => byte_compiler.emit_opcode(Opcode::Pow),
                ArithmeticOp::Mod => byte_compiler.emit_opcode(Opcode::Mod),
            }

            if !use_expr {
                byte_compiler.emit(Opcode::Pop, &[]);
            }
        }
        BinaryOp::Bitwise(op) => {
            byte_compiler.compile_expr(binary.rhs(), true)?;
            match op {
                BitwiseOp::And => byte_compiler.emit_opcode(Opcode::BitAnd),
                BitwiseOp::Or => byte_compiler.emit_opcode(Opcode::BitOr),
                BitwiseOp::Xor => byte_compiler.emit_opcode(Opcode::BitXor),
                BitwiseOp::Shl => byte_compiler.emit_opcode(Opcode::ShiftLeft),
                BitwiseOp::Shr => byte_compiler.emit_opcode(Opcode::ShiftRight),
                BitwiseOp::UShr => byte_compiler.emit_opcode(Opcode::UnsignedShiftRight),
            }

            if !use_expr {
                byte_compiler.emit(Opcode::Pop, &[]);
            }
        }
        BinaryOp::Relational(op) => {
            byte_compiler.compile_expr(binary.rhs(), true)?;
            match op {
                RelationalOp::Equal => byte_compiler.emit_opcode(Opcode::Eq),
                RelationalOp::NotEqual => byte_compiler.emit_opcode(Opcode::NotEq),
                RelationalOp::StrictEqual => byte_compiler.emit_opcode(Opcode::StrictEq),
                RelationalOp::StrictNotEqual => byte_compiler.emit_opcode(Opcode::StrictNotEq),
                RelationalOp::GreaterThan => byte_compiler.emit_opcode(Opcode::GreaterThan),
                RelationalOp::GreaterThanOrEqual => {
                    byte_compiler.emit_opcode(Opcode::GreaterThanOrEq);
                }
                RelationalOp::LessThan => byte_compiler.emit_opcode(Opcode::LessThan),
                RelationalOp::LessThanOrEqual => byte_compiler.emit_opcode(Opcode::LessThanOrEq),
                RelationalOp::In => byte_compiler.emit_opcode(Opcode::In),
                RelationalOp::InstanceOf => byte_compiler.emit_opcode(Opcode::InstanceOf),
            }

            if !use_expr {
                byte_compiler.emit(Opcode::Pop, &[]);
            }
        }
        BinaryOp::Logical(op) => {
            match op {
                LogicalOp::And => {
                    let exit = byte_compiler.emit_opcode_with_operand(Opcode::LogicalAnd);
                    byte_compiler.compile_expr(binary.rhs(), true)?;
                    byte_compiler.patch_jump(exit);
                }
                LogicalOp::Or => {
                    let exit = byte_compiler.emit_opcode_with_operand(Opcode::LogicalOr);
                    byte_compiler.compile_expr(binary.rhs(), true)?;
                    byte_compiler.patch_jump(exit);
                }
                LogicalOp::Coalesce => {
                    let exit = byte_compiler.emit_opcode_with_operand(Opcode::Coalesce);
                    byte_compiler.compile_expr(binary.rhs(), true)?;
                    byte_compiler.patch_jump(exit);
                }
            };

            if !use_expr {
                byte_compiler.emit(Opcode::Pop, &[]);
            }
        }
        BinaryOp::Comma => {
            byte_compiler.emit(Opcode::Pop, &[]);
            byte_compiler.compile_expr(binary.rhs(), true)?;

            if !use_expr {
                byte_compiler.emit(Opcode::Pop, &[]);
            }
        }
    };

    Ok(())
}
