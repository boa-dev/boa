use crate::{
    vm::{opcode::Operation, ShouldExit},
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

    fn execute(_context: &mut Context) -> JsResult<ShouldExit> {
        Ok(ShouldExit::Yield)
    }
}
