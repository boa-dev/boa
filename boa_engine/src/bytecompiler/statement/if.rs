use crate::{bytecompiler::ByteCompiler, vm::Opcode};
use boa_ast::{operations::returns_value, statement::If};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_if(&mut self, node: &If, use_expr: bool) {
        self.compile_expr(node.cond(), true);
        let jelse = self.jump_if_false();

        if !returns_value(node.body()) {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.compile_stmt(node.body(), true);

        let exit = self.jump();
        self.patch_jump(jelse);
        match node.else_node() {
            None => {
                self.emit_opcode(Opcode::PushUndefined);
            }
            Some(else_body) => {
                if !returns_value(else_body) {
                    self.emit_opcode(Opcode::PushUndefined);
                }
                self.compile_stmt(else_body, true);
            }
        }
        self.patch_jump(exit);

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }
}
