use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsValue,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
