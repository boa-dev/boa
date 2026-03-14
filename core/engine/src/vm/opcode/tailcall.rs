use crate::{
    Context, JsError, JsNativeError,
    vm::{Opcode, completion_record::CompletionRecord},
};

use super::OPCODE_HANDLERS_TAILCALL;

impl Context {
    pub(crate) extern "rust-preserve-none" fn dispatch_next(
        &mut self,
        pc: usize,
    ) -> CompletionRecord {
        match self.vm.frame().code_block.bytecode.bytes.get(pc) {
            Some(&byte) => {
                #[cfg(all(feature = "trace", not(all(feature = "tailcall", boa_nightly))))]
                unreachable!();
                let opcode = Opcode::decode(byte);

                #[cfg(feature = "trace")]
                if self.vm.trace || self.vm.frame().code_block.traceable() {
                    use crate::sys::time::Instant;

                    if self.vm.current_frame != Some(self.vm.frame()) {
                        println!();
                        self.trace_call_frame();
                        self.vm.current_frame = Some(self.vm.frame());
                    }

                    let frame = self.vm.frame();
                    let (instruction, _) = frame.code_block.bytecode.next_instruction(pc);

                    let operands = frame.code_block().instruction_operands(&instruction);

                    // measure time since last dispatch
                    let now = Instant::now();
                    let duration = now - frame.code_block().last_trace_time.get().unwrap_or(now);
                    frame.code_block().last_trace_time.set(Some(now));

                    let stack = self.vm.stack.display_trace(frame, self.vm.frames.len() - 1);

                    println!(
                        "{:<TIME_COLUMN_WIDTH$} {:<OPCODE_COLUMN_WIDTH$} {operands:<OPERAND_COLUMN_WIDTH$} {stack}",
                        format!("{}μs", duration.as_micros()),
                        opcode.as_str(),
                        TIME_COLUMN_WIDTH = Context::TIME_COLUMN_WIDTH,
                        OPCODE_COLUMN_WIDTH = Context::OPCODE_COLUMN_WIDTH,
                        OPERAND_COLUMN_WIDTH = Context::OPERAND_COLUMN_WIDTH,
                    );
                }

                become OPCODE_HANDLERS_TAILCALL[opcode as usize](self, pc);
            }
            None => CompletionRecord::Throw(JsError::from_native(JsNativeError::error())), // program ended without a return
        }
    }
}

macro_rules! generate_opcode_tailcall_handlers {
        (
        $(
            $(#[$comment:ident $($args:tt)*])*
            $Variant:ident $({
                $(
                    $(#[$fieldinner:ident $($fieldargs:tt)*])*
                    $FieldName:ident : $FieldType:ty
                ),*
                $(,)?
            })? $(=> $mapping:ident)?
        ),*
        $(,)?
    ) => {
        type OpcodeHandlerTailCall = extern "rust-preserve-none" fn(&mut Context, usize) -> CompletionRecord;

        const OPCODE_HANDLERS_TAILCALL: [OpcodeHandlerTailCall; 256] = {
            [
                $(
                    paste::paste! { [<handle_ $Variant:snake _tailcall>] },
                )*
            ]
        };

        $(
            paste::paste! {
                #[allow(unused_parens)]
                extern "rust-preserve-none" fn [<handle_ $Variant:snake _tailcall>](
                    context: &mut Context,
                    pc: usize,
                ) -> CompletionRecord {
                    let bytes = &context.vm.frame().code_block.bytecode.bytes;
                    let (args, next_pc) = <($($($FieldType),*)?)>::decode(bytes, pc + 1);
                    context.vm.frame_mut().pc = next_pc as u32;
                    let result = $Variant::operation(args, context);

                    let cr = IntoCompletionRecord::into_completion_record(result, context);

                    // This match MUST be the last expression — both arms in tail position
                    match cr {
                        ControlFlow::Continue(()) => become context.dispatch_next(context.vm.frame().pc as usize),
                        ControlFlow::Break(value) => value,
                    }
                }
            }
        )*
    }
}

pub(crate) use generate_opcode_tailcall_handlers;
