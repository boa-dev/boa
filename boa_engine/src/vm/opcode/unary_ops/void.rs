use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

/// `Void` implements the Opcode Operation for `Opcode::Void`
///
/// Operation:
///  - Unary `void` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Void;

impl Operation for Void {
    const NAME: &'static str = "Void";
    const INSTRUCTION: &'static str = "INST - Void";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let _old = context.vm.pop();
        context.vm.push(JsValue::undefined());
        Ok(CompletionType::Normal)
    }
}
