use crate::{
    vm::{call_frame::AbruptCompletionRecord, opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

/// `Break` implements the Opcode Operation for `Opcode::Break`
///
/// Operation:
///   - Pop required environments and jump to address.
pub(crate) struct Break;

impl Operation for Break {
    const NAME: &'static str = "Break";
    const INSTRUCTION: &'static str = "INST - Break";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let jump_address = context.vm.read::<u32>();
        let target_address = context.vm.read::<u32>();

        let value = context.vm.stack.pop().unwrap_or(JsValue::undefined());

        // 1. Iterate through Env stack looking for exit address.
        let mut envs_to_pop = 0;
        let mut set_loop_result = false;
        let mut found_target = false;
        for i in (0..context.vm.frame().env_stack.len()).rev() {
            if found_target && set_loop_result {
                break;
            }

            let Some(env_entry) = context.vm.frame_mut().env_stack.get_mut(i) else {
                break;
            };

            if found_target {
                set_loop_result = env_entry.set_loop_return_value(&value);
                continue;
            }

            if (jump_address == env_entry.exit_address())
                || (env_entry.is_finally_env() && jump_address == env_entry.start_address())
            {
                found_target = true;
                set_loop_result = env_entry.set_loop_return_value(&value);
                continue;
            }

            // Checks for the break if we have jumped from inside of a finally block
            if jump_address == env_entry.exit_address() {
                found_target = true;
                set_loop_result = env_entry.set_loop_return_value(&value);
                continue;
            }
            envs_to_pop += env_entry.env_num();
            context.vm.frame_mut().env_stack.pop();
        }

        let env_truncation_len = context.vm.environments.len().saturating_sub(envs_to_pop);
        context.vm.environments.truncate(env_truncation_len);

        // 2. Register target address in AbruptCompletionRecord.
        let new_record = AbruptCompletionRecord::new_break().with_initial_target(target_address);
        context.vm.frame_mut().abrupt_completion = Some(new_record);

        // 3. Set program counter and finally return fields.
        context.vm.frame_mut().pc = jump_address as usize;
        Ok(CompletionType::Normal)
    }
}
