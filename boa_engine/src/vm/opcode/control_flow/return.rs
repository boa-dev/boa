use crate::{
    vm::{call_frame::AbruptCompletionRecord, opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `Return` implements the Opcode Operation for `Opcode::Return`
///
/// Operation:
///  - Return from a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Return;

impl Operation for Return {
    const NAME: &'static str = "Return";
    const INSTRUCTION: &'static str = "INST - Return";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let current_address = context.vm.frame().pc;
        let mut env_to_pop = 0;
        let mut finally_address = None;
        while let Some(env_entry) = context.vm.frame().env_stack.last() {
            if env_entry.is_finally_env() {
                if env_entry.start_address() < current_address {
                    finally_address = Some(env_entry.exit_address());
                } else {
                    finally_address = Some(env_entry.start_address());
                }
                break;
            }

            env_to_pop += env_entry.env_num();
            if env_entry.is_global_env() {
                break;
            }

            context.vm.frame_mut().env_stack.pop();
        }

        let env_truncation_len = context.vm.environments.len().saturating_sub(env_to_pop);
        context.vm.environments.truncate(env_truncation_len);

        let record = AbruptCompletionRecord::new_return();
        context.vm.frame_mut().abrupt_completion = Some(record);

        if let Some(finally) = finally_address {
            context.vm.frame_mut().pc = finally;
            return Ok(CompletionType::Normal);
        }

        Ok(CompletionType::Return)
    }
}
