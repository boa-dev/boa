use boa_ast::statement::Continue;

use crate::{bytecompiler::ByteCompiler, vm::Opcode, JsNativeError, JsResult};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_continue(&mut self, node: Continue) -> JsResult<()> {
        if let Some(info) = self.jump_info.last().filter(|info| info.is_try_block()) {
            let in_finally = info.in_finally();
            let in_finally_or_has_finally = in_finally || info.has_finally();
            let in_catch_no_finally = !info.has_finally() && info.in_catch();

            if in_finally {
                self.emit_opcode(Opcode::PopIfThrown);
            }
            if in_finally || in_catch_no_finally {
                self.emit_opcode(Opcode::CatchEnd2);
            } else {
                self.emit_opcode(Opcode::TryEnd);
            }

            // 1. Handle if node has a label.
            if let Some(node_label) = node.label() {
                let items = self.jump_info.iter().rev().filter(|info| info.is_loop());
                let mut emit_for_of_in_exit = 0_u32;
                let mut loop_info = None;
                for info in items {
                    if info.label() == Some(node_label) {
                        loop_info = Some(info);
                        break;
                    }

                    if info.for_of_in_loop() {
                        emit_for_of_in_exit += 1;
                    }
                }

                // TODO: promote to an early error.
                loop_info.ok_or_else(|| {
                    JsNativeError::syntax().with_message(format!(
                        "Cannot use the undeclared label '{}'",
                        self.context.interner().resolve_expect(node_label)
                    ))
                })?;

                for _ in 0..emit_for_of_in_exit {
                    self.emit_opcode(Opcode::Pop);
                    self.emit_opcode(Opcode::Pop);
                    self.emit_opcode(Opcode::Pop);
                }

                let (cont_label, set_label) = self.emit_opcode_with_two_operands(Opcode::Continue);

                let loops = self
                    .jump_info
                    .iter_mut()
                    .rev()
                    .filter(|info| info.is_loop());
                let mut set_continue_as_break = false;
                for info in loops {
                    let found_label = info.label() == Some(node_label);
                    if found_label && in_finally_or_has_finally {
                        set_continue_as_break = true;
                        info.push_try_continue_label(set_label);
                        break;
                    } else if found_label && !in_finally_or_has_finally {
                        info.push_try_continue_label(cont_label);
                        info.push_try_continue_label(set_label);
                        break;
                    }
                }
                if set_continue_as_break {
                    self.jump_info
                        .last_mut()
                        .expect("no jump information found")
                        .push_break_label(cont_label);
                }
            } else {
                let (cont_label, set_label) = self.emit_opcode_with_two_operands(Opcode::Continue);
                let mut items = self
                    .jump_info
                    .iter_mut()
                    .rev()
                    .filter(|info| info.is_loop());
                let jump_info = items
                    .next()
                    // TODO: promote to an early error.
                    .ok_or_else(|| {
                        JsNativeError::syntax().with_message("continue must be inside loop")
                    })?;
                jump_info.push_try_continue_label(cont_label);
                jump_info.push_try_continue_label(set_label);
            };
        } else {
            if let Some(node_label) = node.label() {
                let items = self.jump_info.iter().rev().filter(|info| info.is_loop());
                let mut emit_for_of_in_exit = 0_u32;
                let mut loop_info = None;
                for info in items {
                    if info.label() == Some(node_label) {
                        loop_info = Some(info);
                        break;
                    }

                    if info.for_of_in_loop() {
                        emit_for_of_in_exit += 1;
                    }
                }

                // TODO: promote to an early error.
                loop_info.ok_or_else(|| {
                    JsNativeError::syntax().with_message(format!(
                        "Cannot use the undeclared label '{}'",
                        self.context.interner().resolve_expect(node_label)
                    ))
                })?;

                for _ in 0..emit_for_of_in_exit {
                    self.emit_opcode(Opcode::Pop);
                    self.emit_opcode(Opcode::Pop);
                    self.emit_opcode(Opcode::Pop);
                }

                let (cont_label, set_label) = self.emit_opcode_with_two_operands(Opcode::Continue);
                let loops = self
                    .jump_info
                    .iter_mut()
                    .rev()
                    .filter(|info| info.is_loop());

                for info in loops {
                    if info.label() == Some(node_label) {
                        info.push_try_continue_label(cont_label);
                        info.push_try_continue_label(set_label);
                    }
                }
            } else {
                let (cont_label, set_label) = self.emit_opcode_with_two_operands(Opcode::Continue);
                let mut items = self
                    .jump_info
                    .iter_mut()
                    .rev()
                    .filter(|info| info.is_loop());
                let jump_info = items
                    .next()
                    // TODO: promote to an early error.
                    .ok_or_else(|| {
                        JsNativeError::syntax().with_message("continue must be inside loop")
                    })?;
                jump_info.push_try_continue_label(cont_label);
                jump_info.push_try_continue_label(set_label);
            };
        }

        Ok(())
    }
}
