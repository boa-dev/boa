use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PopIfThrown;

impl Operation for PopIfThrown {
    const NAME: &'static str = "PopIfThrown";
    const INSTRUCTION: &'static str = "INST - PopIfThrown";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let frame = context.vm.frame_mut();
        if frame.thrown {
            frame.thrown = false;
            context.vm.pop();
        }
        Ok(ShouldExit::False)
    }
}
