use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `Nop` implements the Opcode Operation for `Opcode::Nop`
///
/// Operation:
///  - No-operation instruction, does nothing
#[derive(Debug, Clone, Copy)]
pub(crate) struct Nop;

impl Operation for Nop {
    const NAME: &'static str = "Nop";
    const INSTRUCTION: &'static str = "INST - Nop";

    fn execute(_context: &mut Context) -> JsResult<ShouldExit> {
        Ok(ShouldExit::False)
    }
}
