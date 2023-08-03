use crate::{
    vm::{opcode::Operation, CompletionType},
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

    fn execute(_: &mut Context<'_>) -> JsResult<CompletionType> {
        Ok(CompletionType::Normal)
    }
}

/// `Reserved` implements the Opcode Operation for `Opcode::Reserved`
///
/// Operation:
///  - Panics, this should be unreachable.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Reserved;

impl Operation for Reserved {
    const NAME: &'static str = "Reserved";
    const INSTRUCTION: &'static str = "INST - Reserved";

    fn execute(_: &mut Context<'_>) -> JsResult<CompletionType> {
        unreachable!("Reserved opcodes are unreachable!")
    }

    fn half_execute(_: &mut Context<'_>) -> JsResult<CompletionType> {
        unreachable!("Half.Reserved opcodes are unreachable!")
    }

    fn wide_execute(_: &mut Context<'_>) -> JsResult<CompletionType> {
        unreachable!("Wide.Reserved opcodes are unreachable!")
    }
}
