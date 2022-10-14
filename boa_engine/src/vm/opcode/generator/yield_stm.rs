use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Yield;

impl Operation for Yield {
    const NAME: &'static str = "Yield";
    const INSTRUCTION: &'static str = "INST - Yield";

    fn execute(_context: &mut Context) -> JsResult<ShouldExit> {
        Ok(ShouldExit::Yield)
    }
}
