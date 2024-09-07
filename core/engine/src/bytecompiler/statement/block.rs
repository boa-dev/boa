use crate::{bytecompiler::ByteCompiler, vm::Opcode};
use boa_ast::statement::Block;

impl ByteCompiler<'_> {
    /// Compile a [`Block`] `boa_ast` node
    pub(crate) fn compile_block(&mut self, block: &Block, use_expr: bool) {
        let outer_scope = if let Some(scope) = block.scope() {
            let outer_scope = self.lexical_scope.clone();
            let scope_index = self.push_scope(scope);
            self.emit_with_varying_operand(Opcode::PushScope, scope_index);
            Some(outer_scope)
        } else {
            None
        };

        self.block_declaration_instantiation(block);
        self.compile_statement_list(block.statement_list(), use_expr, true);

        if let Some(outer_scope) = outer_scope {
            self.pop_scope();
            self.lexical_scope = outer_scope;
            self.emit_opcode(Opcode::PopEnvironment);
        }
    }
}
