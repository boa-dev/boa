use boa_ast::statement::Switch;

use crate::{bytecompiler::ByteCompiler, vm::Opcode, JsResult};

impl ByteCompiler<'_, '_> {
    /// Compile a [`Swtich`] boa_ast node
    pub(crate) fn compile_switch(
        &mut self,
        switch: &Switch,
        configurable_globals: bool,
    ) -> JsResult<()> {
        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
        for case in switch.cases() {
            self.create_decls(case.body(), configurable_globals);
        }
        self.emit_opcode(Opcode::LoopStart);

        let start_address = self.next_opcode_location();
        self.push_switch_control_info(None, start_address);

        self.compile_expr(switch.val(), true)?;
        let mut labels = Vec::with_capacity(switch.cases().len());
        for case in switch.cases() {
            self.compile_expr(case.condition(), true)?;
            labels.push(self.emit_opcode_with_operand(Opcode::Case));
        }

        let exit = self.emit_opcode_with_operand(Opcode::Default);

        for (label, case) in labels.into_iter().zip(switch.cases()) {
            self.patch_jump(label);
            self.compile_statement_list(case.body(), false, configurable_globals)?;
        }

        self.patch_jump(exit);
        if let Some(body) = switch.default() {
            self.create_decls(body, configurable_globals);
            self.compile_statement_list(body, false, configurable_globals)?;
        }

        self.pop_switch_control_info();

        self.emit_opcode(Opcode::LoopEnd);

        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        self.emit_opcode(Opcode::PopEnvironment);

        Ok(())
    }
}
