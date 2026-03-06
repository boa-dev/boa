use crate::bytecompiler::ByteCompiler;
use boa_ast::statement::If;

impl<'arena> ByteCompiler<'arena, '_> {
    pub(crate) fn compile_if(&mut self, node: &'arena If<'arena>, use_expr: bool) {
        let value = self.register_allocator.alloc();
        self.compile_expr(node.cond(), &value);
        self.if_else_with_dealloc(
            value,
            |compiler| {
                compiler.compile_stmt(node.body(), use_expr, true);
            },
            |compiler| {
                if let Some(else_body) = node.else_node() {
                    compiler.compile_stmt(else_body, use_expr, true);
                }
            },
        );
    }
}
