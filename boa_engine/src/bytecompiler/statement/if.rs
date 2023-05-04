use crate::bytecompiler::ByteCompiler;
use boa_ast::statement::If;

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_if(&mut self, node: &If, use_expr: bool) {
        self.compile_expr(node.cond(), true);
        let jelse = self.jump_if_false();

        self.compile_stmt(node.body(), use_expr);

        match node.else_node() {
            None => {
                self.patch_jump(jelse);
            }
            Some(else_body) => {
                let exit = self.jump();
                self.patch_jump(jelse);
                self.compile_stmt(else_body, use_expr);
                self.patch_jump(exit);
            }
        }
    }
}
