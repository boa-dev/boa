
use crate::{
    bytecompiler::ByteCompiler,
    vm::Opcode,
    JsResult,
};

use boa_ast::statement::Block;
use boa_interner::Sym;

impl ByteCompiler<'_, '_> {
    /// Compile a [`Block`] *boa_ast* node
    pub(crate) fn compile_block(
        &mut self,
        block: &Block,
        label: Option<Sym>,
        use_expr: bool,
        configurable_globals: bool,
    ) -> JsResult<()> {
        if let Some(label) = label {
            let next = self.next_opcode_location();
            self.push_labelled_block_control_info(label, next);
        }

        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
        self.create_decls(block.statement_list(), configurable_globals);
        self.compile_statement_list(block.statement_list(), use_expr, configurable_globals)?;
        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);

        if label.is_some() {
            self.pop_labelled_block_control_info();
        }

        self.emit_opcode(Opcode::PopEnvironment);
        Ok(())
    }
}