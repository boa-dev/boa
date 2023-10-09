use crate::{bytecompiler::ByteCompiler, vm::Opcode};
use boa_ast::statement::Block;

impl ByteCompiler<'_, '_> {
    /// Compile a [`Block`] `boa_ast` node
    pub(crate) fn compile_block(&mut self, block: &Block, use_expr: bool) {
        let old_lex_env = self.lexical_environment.clone();
        let env_index = self.push_compile_environment(false);
        self.emit_with_varying_operand(Opcode::PushDeclarativeEnvironment, env_index);
        let env = self.lexical_environment.clone();

        self.block_declaration_instantiation(block, &env);
        self.compile_statement_list(block.statement_list(), use_expr, true);

        self.pop_compile_environment();
        self.lexical_environment = old_lex_env;
        self.emit_opcode(Opcode::PopEnvironment);
    }
}
