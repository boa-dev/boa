use crate::bytecompiler::{Access, BindingAccessOpcode, ByteCompiler, Register, ToJsString};
use crate::vm::opcode::{BitNot, LogicalNot, Neg, Pos, PushTrue, PushUndefined, TypeOf};
use boa_ast::Expression;
use boa_ast::expression::operator::{Unary, unary::UnaryOp};

impl ByteCompiler<'_> {
    pub(crate) fn compile_unary(&mut self, unary: &Unary, dst: &Register) {
        match unary.op() {
            UnaryOp::Delete => {
                let mut compiler = self.position_guard(unary);

                if let Some(access) = Access::from_expression(unary.target()) {
                    compiler.access_delete(access, dst);
                } else if let Expression::Optional(opt) = unary.target() {
                    compiler.compile_optional_delete(opt, dst);
                } else {
                    compiler.compile_expr(unary.target(), dst);
                    PushTrue::emit(&mut compiler, dst.variable());
                }
            }
            UnaryOp::Minus => {
                self.compile_expr(unary.target(), dst);
                Neg::emit(self, dst.variable());
            }
            UnaryOp::Plus => {
                self.compile_expr(unary.target(), dst);
                Pos::emit(self, dst.variable());
            }
            UnaryOp::Not => {
                self.compile_expr(unary.target(), dst);
                LogicalNot::emit(self, dst.variable());
            }
            UnaryOp::Tilde => {
                self.compile_expr(unary.target(), dst);
                BitNot::emit(self, dst.variable());
            }
            UnaryOp::TypeOf => {
                match unary.target().flatten() {
                    Expression::Identifier(identifier) => {
                        let identifier = identifier.to_js_string(self.interner());
                        let binding = self.lexical_scope.get_identifier_reference(identifier);
                        let index = self.get_binding(&binding);
                        let opcode = if binding.is_lexical() {
                            BindingAccessOpcode::GetName
                        } else {
                            BindingAccessOpcode::GetNameOrUndefined
                        };
                        self.emit_binding_access(opcode, &index, dst);
                    }
                    expr => self.compile_expr(expr, dst),
                }
                TypeOf::emit(self, dst.variable());
            }
            UnaryOp::Void => {
                self.compile_expr(unary.target(), dst);
                PushUndefined::emit(self, dst.variable());
            }
        }
    }
}
