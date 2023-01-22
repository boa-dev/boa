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
        let try_target_labels = if self
            .jump_info
            .last()
            .expect("jump_info must exist")
            .is_try_block()
        {
            let (set_jump_label, init_envs) =
                self.emit_opcode_with_two_operands(Opcode::SetBreakTarget);
            (Some(set_jump_label), Some(init_envs))
        } else {
            (None, None)
        };

        if node.label().is_some() && try_target_labels.0.is_some() {
            let envs = self.search_jump_info_label(
                try_target_labels.0.expect("must exist"),
                node.label().expect("must exist as well"),
            )?;
            // Update the initial envs field of `SetBreakTarget` with the initial envs count
            self.patch_jump_with_target(try_target_labels.1.expect("must exist"), envs);
        } else if try_target_labels.0.is_some() {
            self.jump_info
                .last_mut()
                .expect("jump_info must exist")
                .push_set_jumps(try_target_labels.0.expect("value must exist"))
        }

        // Emit the break opcode -> (Label, Label)
        let (break_label, envs_to_pop) = self.emit_opcode_with_two_operands(Opcode::Break);
        if let Some(label_name) = node.label() {
            let envs_count = self.search_jump_info_label(break_label, label_name)?;
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
