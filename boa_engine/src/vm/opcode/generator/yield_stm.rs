use crate::{
    builtins::iterable::create_iter_result_object,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `Yield` implements the Opcode Operation for `Opcode::Yield`
///
/// Operation:
///  - Yield from the current execution.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Yield;

impl Operation for Yield {
    const NAME: &'static str = "Yield";
    const INSTRUCTION: &'static str = "INST - Yield";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        let value = create_iter_result_object(value, false, context);
        context.vm.push(value);
        context.vm.frame_mut().r#yield = true;
        Ok(CompletionType::Return)
    }
}
