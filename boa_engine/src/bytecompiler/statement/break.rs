use boa_ast::statement::Break;

use crate::{
    bytecompiler::{ByteCompiler, Label},
    vm::Opcode,
    JsNativeError, JsResult,
};

use boa_interner::Sym;

impl ByteCompiler<'_, '_> {
    /// Compile a [`Break`] `boa_ast` node
    pub(crate) fn compile_break(&mut self, node: Break) -> JsResult<()> {
        let try_with_finally_or_finally = if let Some(info) = self.jump_info.last().filter(|info| info.is_try_block()) {
                let in_finally = info.in_finally();
                let in_catch_no_finally = info.in_catch() && !info.has_finally();
                let has_finally_or_finally = info.has_finally() || info.in_finally();

                if in_finally {
                    self.emit_opcode(Opcode::PopIfThrown);
                }
                if in_finally || in_catch_no_finally {
                    self.emit_opcode(Opcode::CatchEnd2);
                }
                let (set_jump_label, init_envs) =
                    self.emit_opcode_with_two_operands(Opcode::SetBreakTarget);

                if node.label().is_some() {
                    let envs = self.search_jump_info_label(
                        set_jump_label,
                        node.label().expect("must exist as well"),
                    )?;
                    // Update the initial envs field of `SetBreakTarget` with the initial envs count
                    self.patch_jump_with_target(init_envs, envs);
                } else {
                    self.jump_info
                        .last_mut()
                        .expect("jump_info must exist to reach this point")
                        .push_set_jumps(set_jump_label);
                }
                has_finally_or_finally
            } else {
                false
            };

        // Emit the break opcode -> (Label, Label)
        let (break_label, envs_to_pop) = self.emit_opcode_with_two_operands(Opcode::Break);
        if node.label().is_some() && !try_with_finally_or_finally {
            let envs_count = self.search_jump_info_label(
                break_label,
                node.label().expect("must exist in this block"),
            )?;
            self.patch_jump_with_target(envs_to_pop, envs_count);
            return Ok(())
        };
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
        Ok(())
    }

    fn search_jump_info_label(&mut self, address: Label, node_label: Sym) -> JsResult<u32> {
        let mut found = false;
        let mut total_envs: u32 = 0;
        for info in self.jump_info.iter_mut().rev() {
            total_envs += info.decl_envs();
            if info.label() == Some(node_label) {
                info.push_break_label(address);
                found = true;
                break;
            }
        }
        // TODO: promote to an early error.
        if !found {
            return Err(JsNativeError::syntax()
                .with_message(format!(
                    "Cannot use the undeclared label '{}'",
                    self.interner().resolve_expect(node_label)
                ))
                .into());
        }

        Ok(total_envs)
    }
}
