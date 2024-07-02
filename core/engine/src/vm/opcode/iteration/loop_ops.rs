use crate::JsNativeError;
use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `IncrementLoopIteration` implements the Opcode Operation for `Opcode::IncrementLoopIteration`.
///
/// Operation:
///  - Increment loop itearation count.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IncrementLoopIteration;

impl Operation for IncrementLoopIteration {
    const NAME: &'static str = "IncrementLoopIteration";
    const INSTRUCTION: &'static str = "INST - IncrementLoopIteration";
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let max = context.vm.runtime_limits.loop_iteration_limit();
        let frame = context.vm.frame_mut();
        let previous_iteration_count = frame.loop_iteration_count;

        if previous_iteration_count > max {
            return Err(JsNativeError::runtime_limit()
                .with_message(format!("Maximum loop iteration limit {max} exceeded"))
                .into());
        }

        frame.loop_iteration_count = previous_iteration_count.wrapping_add(1);
        Ok(CompletionType::Normal)
    }
}
