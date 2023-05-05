use crate::{
    bytecompiler::{ByteCompiler, Label},
    vm::{BindingOpcode, Opcode},
};
use boa_ast::{
    declaration::Binding,
    operations::bound_names,
    statement::{Finally, Try},
};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_try(&mut self, t: &Try, use_expr: bool) {
        let try_start = self.next_opcode_location();
        let (catch_start, finally_loc) = self.emit_opcode_with_two_operands(Opcode::TryStart);
        self.patch_jump_with_target(finally_loc, u32::MAX);

        // If there is a finally block, initialize the finally control block prior to pushing the try block jump_control
        if t.finally().is_some() {
            self.push_init_finally_control_info();
        }
        self.push_try_control_info(t.finally().is_some(), try_start);

        self.compile_block(t.block(), use_expr);

        self.emit_opcode(Opcode::TryEnd);

        let finally = self.jump();
        self.patch_jump(catch_start);

        if t.catch().is_some() {
            self.compile_catch_stmt(t, use_expr);
        }

        self.patch_jump(finally);

        if let Some(finally) = t.finally() {
            // Pop and push control loops post FinallyStart, as FinallyStart resets flow control variables.
            // Handle finally header operations
            let finally_start = self.next_opcode_location();
            let finally_end = self.emit_opcode_with_operand(Opcode::FinallyStart);
            self.pop_try_control_info(finally_start);
            self.set_jump_control_start_address(finally_start);
            self.patch_jump_with_target(finally_loc, finally_start);
            // Compile finally statement body
            self.compile_finally_stmt(finally, finally_end);
        } else {
            let try_end = self.next_opcode_location();
            self.pop_try_control_info(try_end);
        }
    }

    pub(crate) fn compile_catch_stmt(&mut self, parent_try: &Try, use_expr: bool) {
        let catch = parent_try
            .catch()
            .expect("Catch must exist for compile_catch_stmt to have been invoked");
        self.set_jump_control_in_catch(true);
        let catch_end = self.emit_opcode_with_operand(Opcode::CatchStart);

        self.push_compile_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        if let Some(binding) = catch.parameter() {
            match binding {
                Binding::Identifier(ident) => {
                    self.create_mutable_binding(*ident, false);
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_mutable_binding(ident, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLet);
                }
            }
        } else {
            self.emit_opcode(Opcode::Pop);
        }

        self.compile_block(catch.block(), use_expr);

        let env_info = self.pop_compile_environment();
        self.patch_jump_with_target(push_env.0, env_info.num_bindings as u32);
        self.patch_jump_with_target(push_env.1, env_info.index as u32);
        self.emit_opcode(Opcode::PopEnvironment);
        if parent_try.finally().is_some() {
            self.emit_opcode(Opcode::CatchEnd);
        } else {
            self.emit_opcode(Opcode::CatchEnd2);
        }

        self.patch_jump(catch_end);
        self.set_jump_control_in_finally(false);
    }

    pub(crate) fn compile_finally_stmt(&mut self, finally: &Finally, finally_end_label: Label) {
        self.compile_block(finally.block(), false);

        self.pop_finally_control_info();
        self.patch_jump(finally_end_label);
        self.emit_opcode(Opcode::FinallyEnd);
    }
}
