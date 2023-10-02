use crate::{bytecompiler::ByteCompiler, vm::Opcode};
use boa_ast::statement::Switch;

impl ByteCompiler<'_, '_> {
    /// Compile a [`Switch`] `boa_ast` node
    pub(crate) fn compile_switch(&mut self, switch: &Switch, use_expr: bool) {
        self.compile_expr(switch.val(), true);

        let env_index = self.push_compile_environment(false);
        self.emit_with_varying_operand(Opcode::PushDeclarativeEnvironment, env_index);

        self.block_declaration_instantiation(switch);

        let start_address = self.next_opcode_location();
        self.push_switch_control_info(None, start_address, use_expr);

        let mut labels = Vec::with_capacity(switch.cases().len());
        for case in switch.cases() {
            // If it does not have a condition it is the default case.
            let label = if let Some(condition) = case.condition() {
                self.compile_expr(condition, true);

                self.emit_opcode_with_operand(Opcode::Case)
            } else {
                Self::DUMMY_LABEL
            };

            labels.push(label);
        }

        let default_label = self.emit_opcode_with_operand(Opcode::Default);
        let mut default_label_set = false;

        for (label, case) in labels.into_iter().zip(switch.cases()) {
            // Check if it's the default case.
            let label = if label == Self::DUMMY_LABEL {
                default_label_set = true;
                default_label
            } else {
                label
            };
            self.patch_jump(label);

            self.compile_statement_list(case.body(), use_expr, true);
        }

        if !default_label_set {
            self.patch_jump(default_label);
        }

        self.pop_switch_control_info();

        self.pop_compile_environment();
        self.emit_opcode(Opcode::PopEnvironment);
    }
}
