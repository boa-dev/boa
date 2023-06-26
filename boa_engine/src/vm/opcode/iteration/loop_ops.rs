use crate::JsNativeError;
use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `LoopContinue` implements the Opcode Operation for `Opcode::LoopContinue`.
///
/// Operation:
///  - Pushes a clean environment onto the frame's `EnvEntryStack`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopContinue;

impl Operation for LoopContinue {
    const NAME: &'static str = "LoopContinue";
    const INSTRUCTION: &'static str = "INST - LoopContinue";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let previous_iteration_count = context.vm.frame_mut().loop_iteration_count;

        let max = context.vm.runtime_limits.loop_iteration_limit();
        if previous_iteration_count > max {
            return Err(JsNativeError::runtime_limit()
                .with_message(format!("Maximum loop iteration limit {max} exceeded"))
                .into());
        }

        context.vm.frame_mut().loop_iteration_count = previous_iteration_count.wrapping_add(1);
        Ok(CompletionType::Normal)
    }
}
