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
        let (set_jump, initial_envs, is_try, has_finally) =
            if let Some(info) = self.jump_info.last().filter(|info| info.is_try_block()) {
                let is_try_block = info.is_try_block();
                let has_finally = info.has_finally();
                let (set_jump_label, init_envs) =
                    self.emit_opcode_with_two_operands(Opcode::SetBreakTarget);
                (
                    Some(set_jump_label),
                    Some(init_envs),
                    is_try_block,
                    has_finally,
                )
            } else {
                (None, None, false, false)
            };

        let is_try_no_finally = is_try && !has_finally;

        if node.label().is_some() && set_jump.is_some() {
            let envs = self.search_jump_info_label(
                set_jump.expect("must exist"),
                node.label().expect("must exist as well"),
            )?;
            // Update the initial envs field of `SetBreakTarget` with the initial envs count
            self.patch_jump_with_target(initial_envs.expect("must exist"), envs);
        } else if set_jump.is_some() {
            self.jump_info
                .last_mut()
                .expect("jump_info must exist to reach this point")
                .push_set_jumps(set_jump.expect("value must exist"));
        }

        // Emit the break opcode -> (Label, Label)
        let (break_label, envs_to_pop) = self.emit_opcode_with_two_operands(Opcode::Break);
        if node.label().is_some() && is_try_no_finally {
            let envs_count = self.search_jump_info_label(
                break_label,
                node.label().expect("must exist in this block"),
            )?;
            self.patch_jump_with_target(envs_to_pop, envs_count);
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
