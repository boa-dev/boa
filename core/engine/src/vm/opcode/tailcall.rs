use crate::{
    Context, JsValue,
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
                let opcode = Opcode::decode(byte);
                become OPCODE_HANDLERS_TAILCALL[opcode as usize](self, pc)
            }
            None => CompletionRecord::Normal(JsValue::undefined()),
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

        type OpcodeHandlerBudget = fn(&mut Context, usize, &mut u32) -> ControlFlow<CompletionRecord>;

        pub(crate) const OPCODE_HANDLERS_BUDGET: [OpcodeHandlerBudget; 256] = {
            [
                $(
                    paste::paste! { [<handle_ $Variant:snake _budget>] },
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

        $(
            paste::paste! {
                #[inline(always)]
                #[allow(unused_parens)]
                fn [<handle_ $Variant:snake _budget>](context: &mut Context, pc: usize, budget: &mut u32) -> ControlFlow<CompletionRecord> {
                    *budget = budget.saturating_sub(u32::from($Variant::COST));
                    let bytes = &context.vm.frame().code_block.bytecode.bytes;
                    let (args, next_pc) = <($($($FieldType),*)?)>::decode(bytes, pc + 1);
                    context.vm.frame_mut().pc = next_pc as u32;
                    let result = $Variant::operation(args, context);
                    IntoCompletionRecord::into_completion_record(result, context)
                }
            }
        )*
    }
}

pub(crate) use generate_opcode_tailcall_handlers;
