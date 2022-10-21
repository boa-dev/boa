use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `Swap` implements the Opcode Operation for `Opcode::Swap`
///
/// Operation:
///  - Swap the top two values on the stack.
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
