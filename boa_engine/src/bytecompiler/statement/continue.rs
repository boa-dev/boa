use boa_ast::statement::Continue;

use crate::{
    bytecompiler::{ByteCompiler, JumpControlInfoKind},
    vm::Opcode,
    JsNativeError, JsResult,
};

pub(crate) fn compile_continue(
    byte_compiler: &mut ByteCompiler<'_>,
    node: &Continue,
) -> JsResult<()> {
    let next = byte_compiler.next_opcode_location();
    if let Some(info) = byte_compiler
        .jump_info
        .last()
        .filter(|info| info.kind == JumpControlInfoKind::Try)
    {
        let start_address = info.start_address;
        let in_finally = if let Some(finally_start) = info.finally_start {
            next > finally_start.index
        } else {
            false
        };
        let in_catch_no_finally = !info.has_finally && info.in_catch;

        if in_finally {
            byte_compiler.emit_opcode(Opcode::PopIfThrown);
        }
        if in_finally || in_catch_no_finally {
            byte_compiler.emit_opcode(Opcode::CatchEnd2);
            byte_compiler.emit(Opcode::FinallySetJump, &[start_address]);
        } else {
            byte_compiler.emit_opcode(Opcode::TryEnd);
            byte_compiler.emit(Opcode::FinallySetJump, &[start_address]);
        }
        let label = byte_compiler.jump();
        byte_compiler
            .jump_info
            .last_mut()
            .expect("no jump information found")
            .try_continues
            .push(label);
    } else {
        let mut items = byte_compiler
            .jump_info
            .iter()
            .rev()
            .filter(|info| info.kind == JumpControlInfoKind::Loop);
        let address = if let Some(label_name) = node.label() {
            let mut num_loops = 0;
            let mut emit_for_of_in_exit = 0;
            let mut address_info = None;
            for info in items {
                if info.label == node.label() {
                    address_info = Some(info);
                    break;
                }
                num_loops += 1;
                if info.for_of_in_loop {
                    emit_for_of_in_exit += 1;
                }
            }
            // TODO: promote to an early error.
            let address = address_info
                .ok_or_else(|| {
                    JsNativeError::syntax().with_message(format!(
                        "Cannot use the undeclared label '{}'",
                        byte_compiler.context.interner().resolve_expect(label_name)
                    ))
                })?
                .start_address;
            for _ in 0..emit_for_of_in_exit {
                byte_compiler.emit_opcode(Opcode::Pop);
                byte_compiler.emit_opcode(Opcode::Pop);
                byte_compiler.emit_opcode(Opcode::Pop);
            }
            for _ in 0..num_loops {
                byte_compiler.emit_opcode(Opcode::LoopEnd);
            }
            address
        } else {
            items
                .next()
                // TODO: promote to an early error.
                .ok_or_else(|| {
                    JsNativeError::syntax().with_message("continue must be inside loop")
                })?
                .start_address
        };
        byte_compiler.emit_opcode(Opcode::LoopEnd);
        byte_compiler.emit_opcode(Opcode::LoopStart);
        byte_compiler.emit(Opcode::Jump, &[address]);
    }

    Ok(())
}
