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

impl Operation for Reserved {
    const NAME: &'static str = "Reserved";
    const INSTRUCTION: &'static str = "INST - Reserved";
    const COST: u8 = 0;

    fn execute(_: &mut Registers, _: &mut Context) -> JsResult<CompletionType> {
        unreachable!("Reserved opcodes are unreachable!")
    }

    fn execute_u16(_: &mut Registers, _: &mut Context) -> JsResult<CompletionType> {
        unreachable!("Reserved.U16 opcodes are unreachable!")
    }

    fn execute_u32(_: &mut Registers, _: &mut Context) -> JsResult<CompletionType> {
        unreachable!("Reserved.U32 opcodes are unreachable!")
    }
}
