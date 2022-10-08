use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PushEmptyObject;

impl Operation for PushEmptyObject {
    const NAME: &'static str = "PushEmptyObject";
    const INSTRUCTION: &'static str = "INST - PushEmptyObject";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        context.vm.push(context.construct_object());
        Ok(ShouldExit::False)
    }
}