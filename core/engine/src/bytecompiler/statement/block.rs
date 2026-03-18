use crate::bytecompiler::ByteCompiler;
#[cfg(not(feature = "experimental"))]
use boa_ast::statement::Block;
#[cfg(feature = "experimental")]
use boa_ast::{
    declaration::LexicalDeclaration,
    operations::{LexicallyScopedDeclaration, lexically_scoped_declarations},
    statement::Block,
};

impl ByteCompiler<'_> {
    /// Compile a [`Block`] `boa_ast` node
    pub(crate) fn compile_block(&mut self, block: &Block, use_expr: bool) {
        let scope = self.push_declarative_scope(block.scope());
        self.block_declaration_instantiation(block);

        // Count how many `using` bindings are in this block (statically known at compile time)
        #[cfg(feature = "experimental")]
        let using_count: u32 = lexically_scoped_declarations(block)
            .iter()
            .filter_map(|decl| {
                if let LexicallyScopedDeclaration::LexicalDeclaration(
                    LexicalDeclaration::Using(u) | LexicalDeclaration::AwaitUsing(u),
                ) = decl
                {
                    Some(u.as_ref().len() as u32)
                } else {
                    None
                }
            })
            .sum();

        self.compile_statement_list(block.statement_list(), use_expr, true);

        // Emit DisposeResources with the static count if there are any using declarations
        #[cfg(feature = "experimental")]
        if using_count > 0 {
            self.bytecode.emit_dispose_resources(using_count.into());
        }

        self.pop_declarative_scope(scope);
    }
}
