use boa_ast::{
    expression::operator::{unary::UnaryOp, Unary},
    Expression,
};

use crate::{
    bytecompiler::{Access, ByteCompiler},
    vm::Opcode,
    JsResult,
};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_unary(&mut self, unary: &Unary, use_expr: bool) -> JsResult<()> {
        let opcode = match unary.op() {
            UnaryOp::Delete => {
                if let Some(access) = Access::from_expression(unary.target()) {
                    self.access_delete(access)?;
                } else {
                    self.compile_expr(unary.target(), false)?;
                    self.emit(Opcode::PushTrue, &[]);
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
                        let binding = self.context.get_binding_value(*identifier);
                        let index = self.get_or_insert_binding(binding);
                        self.emit(Opcode::GetNameOrUndefined, &[index]);
                    }
                    expr => self.compile_expr(expr, true)?,
                }
                self.emit_opcode(Opcode::TypeOf);
                None
            }
            UnaryOp::Void => Some(Opcode::Void),
        };

        if let Some(opcode) = opcode {
            self.compile_expr(unary.target(), true)?;
            self.emit(opcode, &[]);
        }

        if !use_expr {
            self.emit(Opcode::Pop, &[]);
        }

        Ok(())
    }
}
