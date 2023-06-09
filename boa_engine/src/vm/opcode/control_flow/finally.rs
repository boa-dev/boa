use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsError, JsResult,
};

/// `FinallyStart` implements the Opcode Operation for `Opcode::FinallyStart`
///
/// Operation:
///  - Start of a finally block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FinallyStart;

impl Operation for FinallyStart {
    const NAME: &'static str = "FinallyStart";
    const INSTRUCTION: &'static str = "INST - FinallyStart";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let exit = context.vm.read::<u32>();

        let finally_env = context
            .vm
            .frame_mut()
            .env_stack
            .last_mut()
            .expect("EnvStackEntries must exist");

        finally_env.set_exit_address(exit);
        Ok(CompletionType::Normal)
    }
}

/// `FinallyEnd` implements the Opcode Operation for `Opcode::FinallyEnd`
///
/// Operation:
///  - End of a finally block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FinallyEnd;

impl Operation for FinallyEnd {
    const NAME: &'static str = "FinallyEnd";
    const INSTRUCTION: &'static str = "INST - FinallyEnd";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let finally_candidates = context
            .vm
            .frame()
            .env_stack
            .iter()
            .filter(|env| env.is_finally_env() && context.vm.frame().pc < env.start_address());

        let next_finally = match finally_candidates.last() {
            Some(env) => env.start_address(),
            _ => u32::MAX,
        };

        let mut envs_to_pop = 0;
        match context.vm.frame().abrupt_completion {
            Some(record) if next_finally < record.target() => {
                context.vm.frame_mut().pc = next_finally;

                while let Some(env_entry) = context.vm.frame().env_stack.last() {
                    if next_finally <= env_entry.exit_address() {
                        break;
                    }

                    envs_to_pop += env_entry.env_num();
                    context.vm.frame_mut().env_stack.pop();
                }

                let env_truncation_len = context.vm.environments.len().saturating_sub(envs_to_pop);
                context.vm.environments.truncate(env_truncation_len);
            }
            Some(record) if record.is_break() && context.vm.frame().pc < record.target() => {
                // handle the continuation of an abrupt break.
                context.vm.frame_mut().pc = record.target();
                while let Some(env_entry) = context.vm.frame().env_stack.last() {
                    if record.target() == env_entry.exit_address() {
                        break;
                    }

                    envs_to_pop += env_entry.env_num();
                    context.vm.frame_mut().env_stack.pop();
                }

                context.vm.frame_mut().abrupt_completion = None;

                let env_truncation_len = context.vm.environments.len().saturating_sub(envs_to_pop);
                context.vm.environments.truncate(env_truncation_len);
            }
            Some(record) if record.is_continue() && context.vm.frame().pc > record.target() => {
                // Handle the continuation of an abrupt continue
                context.vm.frame_mut().pc = record.target();
                while let Some(env_entry) = context.vm.frame().env_stack.last() {
                    if env_entry.start_address() == record.target() {
                        break;
                    }
                    envs_to_pop += env_entry.env_num();
                    context.vm.frame_mut().env_stack.pop();
                }

                context.vm.frame_mut().abrupt_completion = None;
                let env_truncation_len = context.vm.environments.len().saturating_sub(envs_to_pop);
                context.vm.environments.truncate(env_truncation_len);
            }
            Some(record) if record.is_return() => {
                return Ok(CompletionType::Return);
            }
            Some(record)
                if record.is_throw_with_target() && context.vm.frame().pc < record.target() =>
            {
                context.vm.frame_mut().pc = record.target();
                while let Some(env_entry) = context.vm.frame_mut().env_stack.pop() {
                    envs_to_pop += env_entry.env_num();
                    if env_entry.start_address() == record.target() {
                        break;
                    }
                }
                context.vm.frame_mut().abrupt_completion = None;
                let env_truncation_len = context.vm.environments.len().saturating_sub(envs_to_pop);
                context.vm.environments.truncate(env_truncation_len);
            }
            Some(record) if !record.is_throw_with_target() => {
                let current_stack = context
                    .vm
                    .frame_mut()
                    .env_stack
                    .pop()
                    .expect("Popping current finally stack.");

                let env_truncation_len = context
                    .vm
                    .environments
                    .len()
                    .saturating_sub(current_stack.env_num());
                context.vm.environments.truncate(env_truncation_len);

                let err = JsError::from_opaque(context.vm.pop());
                context.vm.err = Some(err);
                return Ok(CompletionType::Throw);
            }
            _ => {
                context.vm.frame_mut().env_stack.pop();
            }
        }

        Ok(CompletionType::Normal)
    }
}
