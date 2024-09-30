use boa_ast::{
    expression::operator::{unary::UnaryOp, Unary},
    Expression,
};

use crate::{
    bytecompiler::{Access, ByteCompiler, ToJsString},
    vm::Opcode,
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_unary(&mut self, unary: &Unary, use_expr: bool) {
        let opcode = match unary.op() {
            UnaryOp::Delete => {
                if let Some(access) = Access::from_expression(unary.target()) {
                    self.access_delete(access);
                } else {
                    self.compile_expr(unary.target(), false);
                    self.emit(Opcode::PushTrue, &[]);
                }
                None
            }
            UnaryOp::Minus => Some(Opcode::Neg),
            UnaryOp::Plus => Some(Opcode::Pos),
            UnaryOp::Not => Some(Opcode::LogicalNot),
            UnaryOp::Tilde => Some(Opcode::BitNot),
            UnaryOp::TypeOf => {
                match unary.target().flatten() {
                    Expression::Identifier(identifier) => {
                        let identifier = identifier.to_js_string(self.interner());
                        let binding = self.lexical_scope.get_identifier_reference(identifier);
                        let index = self.get_or_insert_binding(binding);
                        self.emit_binding_access(Opcode::GetNameOrUndefined, &index);
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
            self.emit_opcode(Opcode::Pop);
        }
    }
}
