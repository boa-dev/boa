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
        if let Some(info) = self.jump_info.last().filter(|info| info.is_try_block()) {
            let in_finally = info.in_finally();
            let in_catch_no_finally = info.in_catch() && !info.has_finally();
            let has_finally_or_is_finally = info.has_finally() || info.in_finally();

            if in_finally {
                self.emit_opcode(Opcode::PopIfThrown);
            }
            if in_finally || in_catch_no_finally {
                self.emit_opcode(Opcode::CatchEnd2);
            }

            let (break_label, target_jump_label) =
                self.emit_opcode_with_two_operands(Opcode::Break);

            if node.label().is_some() {
                self.search_jump_info_label(
                    target_jump_label,
                    node.label().expect("must exist as well"),
                )?;

                if !has_finally_or_is_finally {
                    self.search_jump_info_label(
                        break_label,
                        node.label().expect("must exist as well"),
                    )?;
                    return Ok(());
                }
            } else {
                self.jump_info
                    .last_mut()
                    .expect("jump_info must exist to reach this point")
                    .push_set_jumps(target_jump_label);
            }

            let info = self
                .jump_info
                .last_mut()
                // TODO: Currently would allow unlabelled breaks in try_blocks with no
                // loops or switch. This can prevented by the early error referenced in line 66.
                .expect("This try block must exist");

            info.push_break_label(break_label);

            return Ok(());
        }

        // Emit the break opcode -> (Label, Label)
        let (break_label, target_label) = self.emit_opcode_with_two_operands(Opcode::Break);
        if node.label().is_some() {
            self.search_jump_info_label(
                break_label,
                node.label().expect("must exist in this block"),
            )?;
            self.search_jump_info_label(target_label, node.label().expect("must exist"))?;
            return Ok(());
        };

        let info = self
            .jump_info
            .last_mut()
            // TODO: promote to an early error.
            .ok_or_else(|| {
                JsNativeError::syntax()
                    .with_message("unlabeled break must be inside loop or switch")
            })?;

        info.push_break_label(break_label);
        info.push_break_label(target_label);

        Ok(())
    }

    fn search_jump_info_label(&mut self, address: Label, node_label: Sym) -> JsResult<()> {
        let mut found = false;
        for info in self.jump_info.iter_mut().rev() {
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

        Ok(())
    }
}
