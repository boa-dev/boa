use crate::{vm::CompletionType, Context, JsResult};

use super::{Opcode, Operation};

/// `ModifierU16` implements the Opcode Operation for `Opcode::ModifierU16`
///
/// Operation:
///  - [`Opcode`] prefix operand modifier, makes all varying operands of an instruction [`u16`] sized.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ModifierU16;

impl Operation for ModifierU16 {
    const NAME: &'static str = "ModifierU16";
    const INSTRUCTION: &'static str = "INST - ModifierU16";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let opcode = context.vm.read::<u8>() as usize;

        Opcode::EXECUTE_FNS[256 + opcode](context)
    }
}

/// `ModifierU32` implements the Opcode Operation for `Opcode::ModifierU32`
///
/// Operation:
///  - [`Opcode`] prefix operand modifier, makes all varying operands of an instruction [`u32`] sized.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ModifierU32;

impl Operation for ModifierU32 {
    const NAME: &'static str = "ModifierU32";
    const INSTRUCTION: &'static str = "INST - ModifierU32";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let opcode = context.vm.read::<u8>() as usize;

        Opcode::EXECUTE_FNS[256 * 2 + opcode](context)
    }
}
