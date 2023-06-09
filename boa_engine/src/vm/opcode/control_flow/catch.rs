use crate::{
    vm::{call_frame::EnvStackEntry, opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `CatchStart` implements the Opcode Operation for `Opcode::CatchStart`
///
/// Operation:
///  - Start of a catch block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CatchStart;

impl Operation for CatchStart {
    const NAME: &'static str = "CatchStart";
    const INSTRUCTION: &'static str = "INST - CatchStart";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let start = context.vm.frame().pc - 1;
        let finally = context.vm.read::<u32>();

        context
            .vm
            .frame_mut()
            .env_stack
            .push(EnvStackEntry::new(start, finally - 1).with_catch_flag());

        context.vm.frame_mut().abrupt_completion = None;
        Ok(CompletionType::Normal)
    }
}

/// `CatchEnd` implements the Opcode Operation for `Opcode::CatchEnd`
///
/// Operation:
///  - End of a catch block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CatchEnd;

impl Operation for CatchEnd {
    const NAME: &'static str = "CatchEnd";
    const INSTRUCTION: &'static str = "INST - CatchEnd";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let mut envs_to_pop = 0_usize;
        while let Some(env_entry) = context.vm.frame_mut().env_stack.pop() {
            envs_to_pop += env_entry.env_num();

            if env_entry.is_catch_env() {
                break;
            }
        }

        let env_truncation_len = context.vm.environments.len().saturating_sub(envs_to_pop);
        context.vm.environments.truncate(env_truncation_len);

        Ok(CompletionType::Normal)
    }
}

/// `CatchEnd2` implements the Opcode Operation for `Opcode::CatchEnd2`
///
/// Operation:
///  - End of a catch block
#[derive(Debug, Clone, Copy)]
pub(crate) struct CatchEnd2;

impl Operation for CatchEnd2 {
    const NAME: &'static str = "CatchEnd2";
    const INSTRUCTION: &'static str = "INST - CatchEnd2";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        if let Some(catch_entry) = context
            .vm
            .frame()
            .env_stack
            .last()
            .filter(|entry| entry.is_catch_env())
        {
            let env_truncation_len = context
                .vm
                .environments
                .len()
                .saturating_sub(catch_entry.env_num());
            context.vm.environments.truncate(env_truncation_len);

            context.vm.frame_mut().env_stack.pop();
        }

        Ok(CompletionType::Normal)
    }
}
