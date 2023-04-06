use crate::{
    vm::{call_frame::EnvStackEntry, opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `LabelledStart` implements the Opcode Operation for `Opcode::LabelledStart`
///
/// Operation:
///  - Start of a labelled block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LabelledStart;

impl Operation for LabelledStart {
    const NAME: &'static str = "LabelledStart";
    const INSTRUCTION: &'static str = "INST - LabelledStart";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let start = context.vm.frame().pc as u32 - 1;
        let end = context.vm.read::<u32>();
        context
            .vm
            .frame_mut()
            .env_stack
            .push(EnvStackEntry::new(start, end).with_labelled_flag());
        Ok(CompletionType::Normal)
    }
}

/// `LabelledEnd` implements the Opcode Operation for `Opcode::LabelledEnd`
///
/// Operation:
///  - Clean up environments at the end of labelled block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LabelledEnd;

impl Operation for LabelledEnd {
    const NAME: &'static str = "LabelledEnd";
    const INSTRUCTION: &'static str = "INST - LabelledEnd";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let mut envs_to_pop = 0_usize;
        while let Some(env_entry) = context.vm.frame_mut().env_stack.pop() {
            envs_to_pop += env_entry.env_num();

            if env_entry.is_labelled_env() {
                break;
            }
        }

        let env_truncation_len = context.vm.environments.len().saturating_sub(envs_to_pop);
        context.vm.environments.truncate(env_truncation_len);

        Ok(CompletionType::Normal)
    }
}
