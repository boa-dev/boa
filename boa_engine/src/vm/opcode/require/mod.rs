use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `RequireObjectCoercible` implements the Opcode Operation for `Opcode::RequireObjectCoercible`
///
/// Operation:
///  - Call `RequireObjectCoercible` on the stack value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RequireObjectCoercible;

impl Operation for RequireObjectCoercible {
    const NAME: &'static str = "RequireObjectCoercible";
    const INSTRUCTION: &'static str = "INST - RequireObjectCoercible";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let value = context.vm.pop();
        let value = value.require_object_coercible()?;
        context.vm.push(value.clone());
        Ok(CompletionType::Normal)
    }
}
