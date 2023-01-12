use boa_ast::statement::Continue;

use crate::{bytecompiler::ByteCompiler, vm::Opcode, JsNativeError, JsResult};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_continue(&mut self, node: Continue) -> JsResult<()> {
        let next = self.next_opcode_location();
        if let Some(info) = self.jump_info.last().filter(|info| info.is_try_block()) {
            let start_address = info.start_address();
            let in_finally = if let Some(finally_start) = info.finally_start() {
                next > finally_start.index
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

            self.emit(Opcode::FinallySetJump, &[start_address]);

            let label = self.jump();
            self.jump_info
                .last_mut()
                .expect("no jump information found")
                .push_try_continue_label(label);
        } else {
            let mut items = self.jump_info.iter().rev().filter(|info| info.is_loop());
            let address = if let Some(label_name) = node.label() {
                let mut num_loops = 0;
                let mut emit_for_of_in_exit = 0;
                let mut address_info = None;
                for info in items {
                    if info.label() == node.label() {
                        address_info = Some(info);
                        break;
                    }
                    num_loops += 1;
                    if info.for_of_in_loop() {
                        emit_for_of_in_exit += 1;
                    }
                }
                // TODO: promote to an early error.
                let address = address_info
                    .ok_or_else(|| {
                        JsNativeError::syntax().with_message(format!(
                            "Cannot use the undeclared label '{}'",
                            self.context.interner().resolve_expect(label_name)
                        ))
                    })?
                    .start_address();
                for _ in 0..emit_for_of_in_exit {
                    self.emit_opcode(Opcode::Pop);
                    self.emit_opcode(Opcode::Pop);
                    self.emit_opcode(Opcode::Pop);
                }
                for _ in 0..num_loops {
                    self.emit_opcode(Opcode::LoopEnd);
                }
                address
            } else {
                items
                    .next()
                    // TODO: promote to an early error.
                    .ok_or_else(|| {
                        JsNativeError::syntax().with_message("continue must be inside loop")
                    })?
                    .start_address()
            };
            self.emit_opcode(Opcode::LoopEnd);
            self.emit_opcode(Opcode::LoopStart);
            self.emit(Opcode::Jump, &[address]);
        }

        Ok(())
    }
}
