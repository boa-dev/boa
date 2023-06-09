use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `Dup` implements the Opcode Operation for `Opcode::Dup`
///
/// Operation:
///  - Push a copy of the top value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Dup;

impl Operation for Dup {
    const NAME: &'static str = "Dup";
    const INSTRUCTION: &'static str = "INST - Dup";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let value = context.vm.pop();
        context.vm.push(value.clone());
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}
