use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `Reserved` implements the Opcode Operation for `Opcode::Reserved`
///
/// Operation:
///  - Panics, this should be unreachable.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Reserved;

impl Reserved {
    pub(crate) fn operation(_: (), _: &mut Registers, _: &mut Context) -> JsResult<CompletionType> {
        unreachable!("Reserved opcodes are unreachable!")
    }
}

impl Operation for Reserved {
    const NAME: &'static str = "Reserved";
    const INSTRUCTION: &'static str = "INST - Reserved";
    const COST: u8 = 0;
}
