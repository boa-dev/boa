use boa_ast::{
    expression::operator::{unary::UnaryOp, Unary},
    Expression,
};

use crate::{
    bytecompiler::{Access, ByteCompiler},
    vm::Opcode,
    JsNativeError, JsResult,
};

pub(crate) fn compile_unary(
    byte_compiler: &mut ByteCompiler<'_>,
    unary: &Unary,
    use_expr: bool,
) -> JsResult<()> {
    let opcode = match unary.op() {
        UnaryOp::IncrementPre => {
            // TODO: promote to an early error.
            let access = Access::from_expression(unary.target())
                .ok_or_else(|| JsNativeError::syntax().with_message("Invalid increment operand"))?;

            byte_compiler.access_set(access, true, |compiler, _| {
                compiler.compile_expr(unary.target(), true)?;
                compiler.emit(Opcode::Inc, &[]);
                Ok(())
            })?;

            None
        }
        UnaryOp::DecrementPre => {
            // TODO: promote to an early error.
            let access = Access::from_expression(unary.target())
                .ok_or_else(|| JsNativeError::syntax().with_message("Invalid decrement operand"))?;

            byte_compiler.access_set(access, true, |compiler, _| {
                compiler.compile_expr(unary.target(), true)?;
                compiler.emit(Opcode::Dec, &[]);
                Ok(())
            })?;
            None
        }
        UnaryOp::IncrementPost => {
            // TODO: promote to an early error.
            let access = Access::from_expression(unary.target())
                .ok_or_else(|| JsNativeError::syntax().with_message("Invalid increment operand"))?;

            byte_compiler.access_set(access, false, |compiler, level| {
                compiler.compile_expr(unary.target(), true)?;
                compiler.emit(Opcode::IncPost, &[]);
                compiler.emit_opcode(Opcode::RotateRight);
                compiler.emit_u8(level + 2);
                Ok(())
            })?;

            None
        }
        UnaryOp::DecrementPost => {
            // TODO: promote to an early error.
            let access = Access::from_expression(unary.target())
                .ok_or_else(|| JsNativeError::syntax().with_message("Invalid decrement operand"))?;

            byte_compiler.access_set(access, false, |compiler, level| {
                compiler.compile_expr(unary.target(), true)?;
                compiler.emit(Opcode::DecPost, &[]);
                compiler.emit_opcode(Opcode::RotateRight);
                compiler.emit_u8(level + 2);
                Ok(())
            })?;

            None
        }
        UnaryOp::Delete => {
            if let Some(access) = Access::from_expression(unary.target()) {
                byte_compiler.access_delete(access)?;
            } else {
                byte_compiler.compile_expr(unary.target(), false)?;
                byte_compiler.emit(Opcode::PushTrue, &[]);
            }
            None
        }
        UnaryOp::Minus => Some(Opcode::Neg),
        UnaryOp::Plus => Some(Opcode::Pos),
        UnaryOp::Not => Some(Opcode::LogicalNot),
        UnaryOp::Tilde => Some(Opcode::BitNot),
        UnaryOp::TypeOf => {
            match &unary.target() {
                Expression::Identifier(identifier) => {
                    let binding = byte_compiler.context.get_binding_value(*identifier);
                    let index = byte_compiler.get_or_insert_binding(binding);
                    byte_compiler.emit(Opcode::GetNameOrUndefined, &[index]);
                }
                expr => byte_compiler.compile_expr(expr, true)?,
            }
            byte_compiler.emit_opcode(Opcode::TypeOf);
            None
        }
        UnaryOp::Void => Some(Opcode::Void),
    };

    if let Some(opcode) = opcode {
        byte_compiler.compile_expr(unary.target(), true)?;
        byte_compiler.emit(opcode, &[]);
    }

    if !use_expr {
        byte_compiler.emit(Opcode::Pop, &[]);
    }

    Ok(())
}
