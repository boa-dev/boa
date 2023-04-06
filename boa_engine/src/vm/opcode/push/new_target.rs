use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

/// `PushNewTarget` implements the Opcode Operation for `Opcode::PushNewTarget`
///
/// Operation:
///  - Push the current new target to the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushNewTarget;

impl Operation for PushNewTarget {
    const NAME: &'static str = "PushNewTarget";
    const INSTRUCTION: &'static str = "INST - PushNewTarget";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let new_target = if let Some(new_target) = context
            .vm
            .environments
            .get_this_environment()
            .as_function_slots()
            .and_then(|env| env.borrow().new_target().cloned())
        {
            new_target.into()
        } else {
            JsValue::undefined()
        };
        context.vm.push(new_target);
        Ok(CompletionType::Normal)
    }
}
