use crate::bytecompiler::ByteCompiler;
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
        
        // Check if this block has any using declarations
        let has_using = lexically_scoped_declarations(block)
            .iter()
            .any(|decl| matches!(
                decl,
                LexicallyScopedDeclaration::LexicalDeclaration(
                    LexicalDeclaration::Using(_) | LexicalDeclaration::AwaitUsing(_)
                )
            ));
        
        // Push disposal scope if this block has using declarations
        if has_using {
            self.bytecode.emit_push_disposal_scope();
        }
        
        self.compile_statement_list(block.statement_list(), use_expr, true);
        
        // Dispose resources if this block has using declarations
        if has_using {
            self.bytecode.emit_dispose_resources();
        }
        
        self.pop_declarative_scope(scope);
    }
}
