use crate::{
    vm::{ShouldExit, opcode::Operation},
    Context, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Swap;

impl Operation for Swap {
    const NAME: &'static str = "Swap";
    const INSTRUCTION: &'static str = "INST - Swap";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let first = context.vm.pop();
        let second = context.vm.pop();

        context.vm.push(first);
        context.vm.push(second);
        Ok(ShouldExit::False)
    }
}