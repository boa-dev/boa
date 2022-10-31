use crate::{
    vm::{opcode::Operation, ShouldExit},
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        if let Some(env) = context
            .realm
            .environments
            .get_this_environment()
            .as_function_slots()
        {
            if let Some(new_target) = env.borrow().new_target() {
                context.vm.push(new_target.clone());
            } else {
                context.vm.push(JsValue::undefined());
            }
        } else {
            context.vm.push(JsValue::undefined());
        }
        Ok(ShouldExit::False)
    }
}
