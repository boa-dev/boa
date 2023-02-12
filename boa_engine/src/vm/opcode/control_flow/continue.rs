use crate::{
    vm::{call_frame::AbruptCompletionRecord, opcode::Operation, FinallyReturn, ShouldExit},
    Context, JsResult,
};

/// `Continue` implements the Opcode Operation for `Opcode::Continue`
///
/// Operands:
///   - Target address
///   - Initial environments to reconcile on continue (will be tracked along with changes to environment stack)
///
/// Operation:
///   - Initializes the `AbruptCompletionRecord` for a delayed continued in a `Opcode::FinallyEnd`
pub(crate) struct Continue;

impl Operation for Continue {
    const NAME: &'static str = "Continue";
    const INSTRUCTION: &'static str = "INST - Continue";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let jump_address = context.vm.read::<u32>();
        let target_address = context.vm.read::<u32>();

        // 1. Iterate through Env stack looking for exit address.
        let mut envs_to_pop = 0;
        while let Some(env_entry) = context.vm.frame_mut().env_stack.last() {
            // We check two conditions here where continue actually jumps to a higher address.
            //   1. When we have reached a finally env that matches the jump address we are moving to.
            //   2. When there is no finally, and we have reached the continue location.
            if (env_entry.is_finally_env() && jump_address == env_entry.start_address())
                || (jump_address == target_address && jump_address == env_entry.start_address())
            {
                break;
            }

            envs_to_pop += env_entry.env_num();
            // The below check determines whether we have continued from inside of a finally block.
            if jump_address > target_address && jump_address == env_entry.exit_address() {
                context.vm.frame_mut().env_stack.pop();
                break;
            }
            context.vm.frame_mut().env_stack.pop();
        }

        let env_truncation_len = context.realm.environments.len().saturating_sub(envs_to_pop);
        context.realm.environments.truncate(env_truncation_len);

        // 2. Register target address in AbruptCompletionRecord.
        let new_record = AbruptCompletionRecord::new_continue().with_initial_target(target_address);
        context.vm.frame_mut().abrupt_completion = Some(new_record);

        // 3. If a value exists on the stack, we store the value in our register.
        if !context.vm.stack.is_empty() {
            let new_value = context.vm.pop();
            context.vm.frame_mut().completion_register = Some(new_value);
        }

        // 4. Set program counter and finally return fields.
        context.vm.frame_mut().pc = jump_address as usize;
        context.vm.frame_mut().finally_return = FinallyReturn::None;
        Ok(ShouldExit::False)
    }
}