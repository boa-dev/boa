use crate::{bytecompiler::ByteCompiler, vm::Opcode};

use boa_ast::statement::Block;

impl ByteCompiler<'_, '_> {
    /// Compile a [`Block`] `boa_ast` node
    pub(crate) fn compile_block(
        &mut self,
        block: &Block,
        use_expr: bool,
        configurable_globals: bool,
    ) {
        self.push_compile_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        self.compile_statement_list(block.statement_list(), use_expr, configurable_globals);

        let env_info = self.pop_compile_environment();
        self.patch_jump_with_target(push_env.0, env_info.num_bindings as u32);
        self.patch_jump_with_target(push_env.1, env_info.index as u32);

        self.emit_opcode(Opcode::PopEnvironment);
    }
}
