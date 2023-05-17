use crate::JsNativeError;
use crate::{
    vm::{call_frame::EnvStackEntry, opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `IteratorLoopStart` implements the Opcode Operation for `Opcode::IteratorLoopStart`
///
/// Operation:
///  - Push iterator loop start marker.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorLoopStart;

impl Operation for IteratorLoopStart {
    const NAME: &'static str = "IteratorLoopStart";
    const INSTRUCTION: &'static str = "INST - IteratorLoopStart";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let start = context.vm.read::<u32>();
        let exit = context.vm.read::<u32>();

        // Create and push loop evironment entry.
        let entry = EnvStackEntry::new(start, exit)
            .with_iterator_loop_flag(1, context.vm.frame().iterators.len() - 1);
        context.vm.frame_mut().env_stack.push(entry);
        Ok(CompletionType::Normal)
    }
}

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
        // 1. Clean up the previous environment.
        let env = context
            .vm
            .frame_mut()
            .env_stack
            .last_mut()
            .expect("loop environment must be present");

        let env_num = env.env_num();
        env.clear_env_num();

        if let Some(previous_iteration_count) = env.as_loop_iteration_count() {
            env.increase_loop_iteration_count();
            let max = context.vm.runtime_limits.loop_iteration_limit();
            if previous_iteration_count > max {
                let env_truncation_len = context.vm.environments.len().saturating_sub(env_num);
                context.vm.environments.truncate(env_truncation_len);
                return Err(JsNativeError::runtime_limit()
                    .with_message(format!("Maximum loop iteration limit {max} exceeded"))
                    .into());
            }
        }

        let env_truncation_len = context.vm.environments.len().saturating_sub(env_num);
        context.vm.environments.truncate(env_truncation_len);

        Ok(CompletionType::Normal)
    }
}

/// `LoopEnd` implements the Opcode Operation for `Opcode::LoopEnd`
///
/// Operation:
///  - Clean up environments at the end of a loop.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopEnd;

impl Operation for LoopEnd {
    const NAME: &'static str = "LoopEnd";
    const INSTRUCTION: &'static str = "INST - LoopEnd";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let mut envs_to_pop = 0_usize;
        while let Some(env_entry) = context.vm.frame_mut().env_stack.pop() {
            envs_to_pop += env_entry.env_num();

            if let Some(value) = env_entry.loop_env_value() {
                context.vm.push(value.clone());

                break;
            }
        }

        let env_truncation_len = context.vm.environments.len().saturating_sub(envs_to_pop);
        context.vm.environments.truncate(env_truncation_len);

        Ok(CompletionType::Normal)
    }
}

/// `LoopUpdateReturnValue` implements the Opcode Operation for `Opcode::LoopUpdateReturnValue`
///
/// Operation:
///  - Update the return value of a loop.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopUpdateReturnValue;

impl Operation for LoopUpdateReturnValue {
    const NAME: &'static str = "LoopUpdateReturnValue";
    const INSTRUCTION: &'static str = "INST - LoopUpdateReturnValue";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        context
            .vm
            .frame_mut()
            .env_stack
            .last_mut()
            .expect("loop environment must be present")
            .set_loop_return_value(value);
        Ok(CompletionType::Normal)
    }
}
