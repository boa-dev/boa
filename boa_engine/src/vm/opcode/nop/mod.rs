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
    const COST: u8 = 1;

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
    const COST: u8 = 0;

    fn execute(_: &mut Context<'_>) -> JsResult<CompletionType> {
        unreachable!("Reserved opcodes are unreachable!")
    }

    fn execute_with_u16_operands(_: &mut Context<'_>) -> JsResult<CompletionType> {
        unreachable!("Reserved.U16 opcodes are unreachable!")
    }

    fn execute_with_u32_operands(_: &mut Context<'_>) -> JsResult<CompletionType> {
        unreachable!("Reserved.U32 opcodes are unreachable!")
    }
}
