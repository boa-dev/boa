use boa_ast::{
    expression::operator::{unary::UnaryOp, Unary},
    Expression,
};

use crate::{
    bytecompiler::{Access, ByteCompiler, Operand, Register, ToJsString},
    vm::Opcode,
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_unary(&mut self, unary: &Unary, dst: &Register) {
        let opcode = match unary.op() {
            UnaryOp::Delete => {
                if let Some(access) = Access::from_expression(unary.target()) {
                    self.access_delete(access, dst);
                } else {
                    self.compile_expr(unary.target(), dst);
                    self.push_true(dst);
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
                        self.emit_binding_access(Opcode::GetNameOrUndefined, &index, dst);
                    }
                    expr => self.compile_expr(expr, dst),
                }
                self.emit(Opcode::TypeOf, &[Operand::Register(dst)]);
                None
            }
            UnaryOp::Void => {
                self.compile_expr(unary.target(), dst);
                self.push_undefined(dst);
                None
            }
        };

        if let Some(opcode) = opcode {
            self.compile_expr(unary.target(), dst);
            self.emit(opcode, &[Operand::Register(dst)]);
        }
    }
}
