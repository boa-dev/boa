use crate::{bytecompiler::ByteCompiler, vm::Opcode};
use boa_ast::statement::Block;

impl ByteCompiler<'_, '_> {
    /// Compile a [`Block`] `boa_ast` node
    pub(crate) fn compile_block(&mut self, block: &Block, use_expr: bool) {
        self.push_compile_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        self.block_declaration_instantiation(block);
        self.compile_statement_list(block.statement_list(), use_expr, true);

        let env_info = self.pop_compile_environment();
        self.patch_jump_with_target(push_env.0, env_info.num_bindings);
        self.patch_jump_with_target(push_env.1, env_info.index);

        self.emit_opcode(Opcode::PopEnvironment);
    }
}
