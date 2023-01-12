use boa_ast::statement::Break;

use crate::{bytecompiler::ByteCompiler, vm::Opcode, JsNativeError, JsResult};

impl ByteCompiler<'_, '_> {
    /// Compile a [`Break`] `boa_ast` node
    pub(crate) fn compile_break(&mut self, node: Break) -> JsResult<()> {
        let next = self.next_opcode_location();
        if let Some(info) = self.jump_info.last().filter(|info| info.is_try_block()) {
            let in_finally = if let Some(finally_start) = info.finally_start() {
                next >= finally_start.index
            } else {
                false
            };
            let in_catch_no_finally = !info.has_finally() && info.in_catch();

            if in_finally {
                self.emit_opcode(Opcode::PopIfThrown);
            }
            if in_finally || in_catch_no_finally {
                self.emit_opcode(Opcode::CatchEnd2);
            } else {
                self.emit_opcode(Opcode::TryEnd);
            }
            self.emit(Opcode::FinallySetJump, &[u32::MAX]);
        }
        let (break_label, envs_to_pop) = self.emit_opcode_with_two_operands(Opcode::Break);
        if let Some(label_name) = node.label() {
            let mut found = false;
            let mut total_envs: u32 = 0;
            for info in self.jump_info.iter_mut().rev() {
                total_envs += info.decl_envs();
                if info.label() == Some(label_name) {
                    info.push_break_label(break_label);
                    found = true;
                    break;
                }
            }
            // TODO: promote to an early error.
            if !found {
                return Err(JsNativeError::syntax()
                    .with_message(format!(
                        "Cannot use the undeclared label '{}'",
                        self.interner().resolve_expect(label_name)
                    ))
                    .into());
            }
            self.patch_jump_with_target(envs_to_pop, total_envs);
        } else {
            let envs = self
                .jump_info
                .last()
                // TODO: promote to an early error.
                .ok_or_else(|| {
                    JsNativeError::syntax()
                        .with_message("unlabeled break must be inside loop or switch")
                })?
                .decl_envs();

            self.patch_jump_with_target(envs_to_pop, envs);

            self.jump_info
                .last_mut()
                .expect("cannot throw error as last access would have thrown")
                .push_break_label(break_label);
        }

        Ok(())
    }
}
