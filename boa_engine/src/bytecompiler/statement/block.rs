use crate::{bytecompiler::ByteCompiler, vm::Opcode};
use boa_ast::statement::Block;

impl ByteCompiler<'_, '_> {
    /// Compile a [`Block`] `boa_ast` node
    pub(crate) fn compile_block(&mut self, block: &Block, use_expr: bool) {
        let env_index = self.push_compile_environment(false);
        if !self.can_optimize_local_variables {
            self.emit_with_varying_operand(Opcode::PushDeclarativeEnvironment, env_index);
        }

        self.block_declaration_instantiation(block);
        self.compile_statement_list(block.statement_list(), use_expr, true);

        self.pop_compile_environment();
        if !self.can_optimize_local_variables {
            self.emit_opcode(Opcode::PopEnvironment);
        }
    }
}
