use crate::JsNativeError;
use crate::{
    vm::{call_frame::EnvStackEntry, opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `LoopStart` implements the Opcode Operation for `Opcode::LoopStart`
///
/// Operation:
///  - Push loop start marker.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopStart;

impl Operation for LoopStart {
    const NAME: &'static str = "LoopStart";
    const INSTRUCTION: &'static str = "INST - LoopStart";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let start = context.vm.read::<u32>();
        let exit = context.vm.read::<u32>();

        // Create and push loop evironment entry.
        let entry = EnvStackEntry::new(start, exit).with_loop_flag(1);
        context.vm.frame_mut().env_stack.push(entry);
        Ok(CompletionType::Normal)
    }
}

/// This is a helper function used to clean the loop environment created by the
/// [`LoopStart`] and [`LoopContinue`] opcodes.
fn cleanup_loop_environment(context: &mut Context<'_>) {
    let mut envs_to_pop = 0_usize;
    while let Some(env_entry) = context.vm.frame_mut().env_stack.pop() {
        envs_to_pop += env_entry.env_num();

        if env_entry.is_loop_env() {
            break;
        }
    }

    let env_truncation_len = context.vm.environments.len().saturating_sub(envs_to_pop);
    context.vm.environments.truncate(env_truncation_len);
}

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
        let start = context.vm.read::<u32>();
        let exit = context.vm.read::<u32>();

        let mut iteration_count = 0;

        // 1. Clean up the previous environment.
        if let Some(entry) = context
            .vm
            .frame()
            .env_stack
            .last()
            .filter(|entry| entry.exit_address() == exit)
        {
            let env_truncation_len = context
                .vm
                .environments
                .len()
                .saturating_sub(entry.env_num());
            context.vm.environments.truncate(env_truncation_len);

            // Pop loop environment and get it's iteration count.
            let previous_entry = context.vm.frame_mut().env_stack.pop();
            if let Some(previous_iteration_count) =
                previous_entry.and_then(EnvStackEntry::as_loop_iteration_count)
            {
                iteration_count = previous_iteration_count.wrapping_add(1);

                let max = context.vm.runtime_limits.loop_iteration_limit();
                if previous_iteration_count > max {
                    cleanup_loop_environment(context);

                    return Err(JsNativeError::runtime_limit()
                        .with_message(format!("Maximum loop iteration limit {max} exceeded"))
                        .into());
                }
            }
        }

        // 2. Push a new clean EnvStack.
        let entry = EnvStackEntry::new(start, exit).with_loop_flag(iteration_count);

        context.vm.frame_mut().env_stack.push(entry);

        Ok(CompletionType::Normal)
    }
}

/// `LoopEnd` implements the Opcode Operation for `Opcode::LoopEnd`
///
/// Operation:
///  - Clean up enviroments at the end of a lopp.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopEnd;

impl Operation for LoopEnd {
    const NAME: &'static str = "LoopEnd";
    const INSTRUCTION: &'static str = "INST - LoopEnd";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        cleanup_loop_environment(context);
        Ok(CompletionType::Normal)
    }
}
