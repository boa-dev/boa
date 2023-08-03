use crate::{vm::CompletionType, Context, JsResult};

use super::{Opcode, Operation};

/// `Half` implements the Opcode Operation for `Opcode::Half`
///
/// Operation:
///  - [`Opcode`] prefix operand modifier, makes all varying operands of an instruction [`u16`] sized.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Half;

impl Operation for Half {
    const NAME: &'static str = "Half";
    const INSTRUCTION: &'static str = "INST - Half";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let opcode = context.vm.read::<u8>() as usize;

        Opcode::EXECUTE_FNS[256 + opcode](context)
    }
}

/// `Wide` implements the Opcode Operation for `Opcode::Wide`
///
/// Operation:
///  - [`Opcode`] prefix operand modifier, makes all varying operands of an instruction [`u16`] sized.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Wide;

impl Operation for Wide {
    const NAME: &'static str = "Wide";
    const INSTRUCTION: &'static str = "INST - Wide";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let opcode = context.vm.read::<u8>() as usize;

        Opcode::EXECUTE_FNS[256 * 2 + opcode](context)
    }
}
