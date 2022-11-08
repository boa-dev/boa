use crate::{
    bytecompiler::{ByteCompiler, Literal},
    vm::Opcode,
    JsNativeError, JsResult,
};

use boa_ast::{
    expression::{
        literal::Literal as AstLiteral,
        operator::{unary::UnaryOp, Unary},
    },
    Expression,
};

use super::Access;

mod assign;
mod binary;
mod object_literal;

pub(crate) use assign::compile_assign;
pub(crate) use binary::compile_binary;
pub(crate) use object_literal::compile_object_literal;

pub(crate) fn compile_literal<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
    lit: &AstLiteral,
    use_expr: bool,
) {
    match lit {
        AstLiteral::String(v) => byte_compiler.emit_push_literal(Literal::String(
            byte_compiler
                .interner()
                .resolve_expect(*v)
                .into_common(false),
        )),
        AstLiteral::Int(v) => byte_compiler.emit_push_integer(*v),
        AstLiteral::Num(v) => byte_compiler.emit_push_rational(*v),
        AstLiteral::BigInt(v) => {
            byte_compiler.emit_push_literal(Literal::BigInt(v.clone().into()));
        }
        AstLiteral::Bool(true) => byte_compiler.emit(Opcode::PushTrue, &[]),
        AstLiteral::Bool(false) => byte_compiler.emit(Opcode::PushFalse, &[]),
        AstLiteral::Null => byte_compiler.emit(Opcode::PushNull, &[]),
        AstLiteral::Undefined => byte_compiler.emit(Opcode::PushUndefined, &[]),
    }

    if !use_expr {
        byte_compiler.emit(Opcode::Pop, &[]);
    }
}

pub(crate) fn compile_unary<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
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
