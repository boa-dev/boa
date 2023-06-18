use crate::bytecompiler::ByteCompiler;
use boa_ast::statement::If;

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_if(&mut self, node: &If, use_expr: bool) {
        self.compile_expr(node.cond(), true);
        let jelse = self.jump_if_false();

        self.compile_stmt(node.body(), use_expr, true);

        let exit = self.jump();
        self.patch_jump(jelse);
        if let Some(else_body) = node.else_node() {
            self.compile_stmt(else_body, use_expr, true);
        }
        self.patch_jump(exit);
    }
}
