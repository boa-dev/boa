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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
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

/// `GetReturnValue` implements the Opcode Operation for `Opcode::GetReturnValue`
///
/// Operation:
///  - Gets the return value of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetReturnValue;

impl Operation for GetReturnValue {
    const NAME: &'static str = "GetReturnValue";
    const INSTRUCTION: &'static str = "INST - GetReturnValue";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.frame().return_value.clone();
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

/// `SetReturnValue` implements the Opcode Operation for `Opcode::SetReturnValue`
///
/// Operation:
///  - Sets the return value of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetReturnValue;

impl Operation for SetReturnValue {
    const NAME: &'static str = "SetReturnValue";
    const INSTRUCTION: &'static str = "INST - SetReturnValue";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        context.vm.frame_mut().return_value = value;
        Ok(CompletionType::Normal)
    }
}
