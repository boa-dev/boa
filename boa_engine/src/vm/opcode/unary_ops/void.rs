use crate::{
    vm::{opcode::Operation, ShouldExit},
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let _old = context.vm.pop();
        context.vm.push(JsValue::undefined());
        Ok(ShouldExit::False)
    }
}
