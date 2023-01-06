use crate::{bytecompiler::ByteCompiler, JsResult};
use boa_ast::statement::If;

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_if(&mut self, node: &If, configurable_globals: bool) -> JsResult<()> {
        self.compile_expr(node.cond(), true)?;
        let jelse = self.jump_if_false();

        self.compile_stmt(node.body(), false, configurable_globals)?;

        match node.else_node() {
            None => {
                self.patch_jump(jelse);
            }
            Some(else_body) => {
                let exit = self.jump();
                self.patch_jump(jelse);
                self.compile_stmt(else_body, false, configurable_globals)?;
                self.patch_jump(exit);
            }
        }

        Ok(())
    }
}
