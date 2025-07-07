use crate::bytecompiler::{
    Access, BindingAccessOpcode, ByteCompiler, Register, SourcePositionGuard, ToJsString,
};
use boa_ast::{
    Expression,
    expression::operator::{Unary, unary::UnaryOp},
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_unary(&mut self, unary: &Unary, dst: &Register) {
        match unary.op() {
            UnaryOp::Delete => {
                let mut compiler = SourcePositionGuard::new(self, unary.span().start());

                if let Some(access) = Access::from_expression(unary.target()) {
                    compiler.access_delete(access, dst);
                } else {
                    compiler.compile_expr(unary.target(), dst);
                    compiler.bytecode.emit_push_true(dst.variable());
                }
            }
            UnaryOp::Minus => {
                self.compile_expr(unary.target(), dst);
                self.bytecode.emit_neg(dst.variable());
            }
            UnaryOp::Plus => {
                self.compile_expr(unary.target(), dst);
                self.bytecode.emit_pos(dst.variable());
            }
            UnaryOp::Not => {
                self.compile_expr(unary.target(), dst);
                self.bytecode.emit_logical_not(dst.variable());
            }
            UnaryOp::Tilde => {
                self.compile_expr(unary.target(), dst);
                self.bytecode.emit_bit_not(dst.variable());
            }
            UnaryOp::TypeOf => {
                match unary.target().flatten() {
                    Expression::Identifier(identifier) => {
                        let identifier = identifier.to_js_string(self.interner());
                        let binding = self.lexical_scope.get_identifier_reference(identifier);
                        let index = self.get_binding(&binding);
                        self.emit_binding_access(
                            BindingAccessOpcode::GetNameOrUndefined,
                            &index,
                            dst,
                        );
                    }
                    expr => self.compile_expr(expr, dst),
                }
                self.bytecode.emit_type_of(dst.variable());
            }
            UnaryOp::Void => {
                self.compile_expr(unary.target(), dst);
                self.bytecode.emit_push_undefined(dst.variable());
            }
        }
    }
}
