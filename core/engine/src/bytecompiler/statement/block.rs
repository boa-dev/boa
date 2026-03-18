use crate::bytecompiler::ByteCompiler;
use boa_ast::statement::Block;

impl ByteCompiler<'_> {
    /// Compile a [`Block`] `boa_ast` node
    pub(crate) fn compile_block(&mut self, block: &Block, use_expr: bool) {
        let scope = self.push_declarative_scope(block.scope());

        // Push a dispose capability marker for `using` declarations.
        self.bytecode.emit_create_dispose_capability();

        // Install an exception handler so `DisposeResources` runs on abnormal completion.
        let handler = self.push_handler();

        self.block_declaration_instantiation(block);
        self.compile_statement_list(block.statement_list(), use_expr, true);

        // Normal completion: dispose resources and jump past the handler.
        self.bytecode.emit_dispose_resources();
        let skip = self.jump();

        // Exception handler: dispose resources and re-throw.
        self.patch_handler(handler);
        self.bytecode.emit_dispose_resources();
        self.bytecode.emit_re_throw();

        self.patch_jump(skip);
        self.pop_declarative_scope(scope);
    }
}
