use crate::{
    vm::{call_frame::EarlyReturnType, opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `Yield` implements the Opcode Operation for `Opcode::Yield`
///
/// Operation:
///  - Yield from the current execution.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Yield;

impl Operation for Yield {
    const NAME: &'static str = "Yield";
    const INSTRUCTION: &'static str = "INST - Yield";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        context.vm.frame_mut().early_return = Some(EarlyReturnType::Yield);
        Ok(CompletionType::Return)
    }
}
