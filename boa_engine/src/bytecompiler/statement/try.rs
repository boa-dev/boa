use boa_ast::{declaration::Binding, operations::bound_names, statement::Try};

use crate::{
    bytecompiler::{ByteCompiler, Label},
    vm::{BindingOpcode, Opcode},
    JsResult,
};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_try(
        &mut self,
        t: &Try,
        use_expr: bool,
        configurable_globals: bool,
    ) -> JsResult<()> {
        self.push_try_control_info(t.finally().is_some());
        let try_start = self.next_opcode_location();
        self.emit(Opcode::TryStart, &[ByteCompiler::DUMMY_ADDRESS, 0]);
        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        self.create_decls(t.block().statement_list(), configurable_globals);
        self.compile_statement_list(t.block().statement_list(), use_expr, configurable_globals)?;

        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        self.emit_opcode(Opcode::PopEnvironment);
        self.emit_opcode(Opcode::TryEnd);

        let finally = self.jump();
        self.patch_jump(Label { index: try_start });

        if let Some(catch) = t.catch() {
            self.set_jump_control_catch_start(true);
            let catch_start = if t.finally().is_some() {
                Some(self.emit_opcode_with_operand(Opcode::CatchStart))
            } else {
                None
            };
            self.context.push_compile_time_environment(false);
            let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
            if let Some(binding) = catch.parameter() {
                match binding {
                    Binding::Identifier(ident) => {
                        self.context.create_mutable_binding(*ident, false, false);
                        self.emit_binding(BindingOpcode::InitLet, *ident);
                    }
                    Binding::Pattern(pattern) => {
                        for ident in bound_names(pattern) {
                            self.context.create_mutable_binding(ident, false, false);
                        }
                        self.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
                    }
                }
            } else {
                self.emit_opcode(Opcode::Pop);
            }

            self.create_decls(catch.block().statement_list(), configurable_globals);
            self.compile_statement_list(
                catch.block().statement_list(),
                use_expr,
                configurable_globals,
            )?;

            let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
            let index_compile_environment = self.push_compile_environment(compile_environment);
            self.patch_jump_with_target(push_env.0, num_bindings as u32);
            self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
            self.emit_opcode(Opcode::PopEnvironment);
            if let Some(catch_start) = catch_start {
                self.emit_opcode(Opcode::CatchEnd);
                self.patch_jump(catch_start);
            } else {
                self.emit_opcode(Opcode::CatchEnd2);
            }
        }

        self.patch_jump(finally);

        if let Some(finally) = t.finally() {
            self.emit_opcode(Opcode::FinallyStart);
            let finally_start_address = self.next_opcode_location();
            self.set_jump_control_finally_start(Label {
                index: finally_start_address,
            });
            self.patch_jump_with_target(
                Label {
                    index: try_start + 4,
                },
                finally_start_address,
            );

            self.context.push_compile_time_environment(false);
            let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

            self.create_decls(finally.block().statement_list(), configurable_globals);
            self.compile_statement_list(
                finally.block().statement_list(),
                false,
                configurable_globals,
            )?;

            let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
            let index_compile_environment = self.push_compile_environment(compile_environment);
            self.patch_jump_with_target(push_env.0, num_bindings as u32);
            self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
            self.emit_opcode(Opcode::PopEnvironment);

            self.emit_opcode(Opcode::FinallyEnd);
            self.pop_try_control_info(Some(finally_start_address));
        } else {
            self.pop_try_control_info(None);
        }

        Ok(())
    }
}
