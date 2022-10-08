use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult
};

pub(crate) mod if_thrown;

pub(crate) use if_thrown::PopIfThrown;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Pop;

impl Operation for Pop {
    const NAME: &'static str = "Pop";
    const INSTRUCTION: &'static str = "INST - Pop";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let _val = context.vm.pop();
        Ok(ShouldExit::False)
    }
}