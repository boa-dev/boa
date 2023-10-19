use crate::{vm::CompletionType, Context, JsResult};

use super::{Opcode, Operation};

/// `U16Operands` implements the Opcode Operation for `Opcode::U16Operands`
///
/// Operation:
///  - [`Opcode`] prefix operand modifier, makes all varying operands of an instruction [`u16`] sized.
#[derive(Debug, Clone, Copy)]
pub(crate) struct U16Operands;

impl Operation for U16Operands {
    const NAME: &'static str = "U16Operands";
    const INSTRUCTION: &'static str = "INST - U16Operands";
    const COST: u8 = 1;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let opcode = context.vm.read::<u8>() as usize;

        Opcode::EXECUTE_FNS[Opcode::MAX + opcode](context)
    }

    fn spend_budget_and_execute(
        context: &mut Context<'_>,
        budget: &mut u32,
    ) -> JsResult<CompletionType> {
        let opcode: Opcode = context.vm.read::<u8>().into();

        *budget = budget.saturating_sub(u32::from(opcode.cost()) + u32::from(Self::COST));
        Opcode::EXECUTE_FNS[Opcode::MAX + opcode as usize](context)
    }
}

/// `U32Operands` implements the Opcode Operation for `Opcode::U32Operands`
///
/// Operation:
///  - [`Opcode`] prefix operand modifier, makes all varying operands of an instruction [`u32`] sized.
#[derive(Debug, Clone, Copy)]
pub(crate) struct U32Operands;

impl Operation for U32Operands {
    const NAME: &'static str = "U32Operands";
    const INSTRUCTION: &'static str = "INST - U32Operands";
    const COST: u8 = 1;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let opcode = context.vm.read::<u8>() as usize;

        Opcode::EXECUTE_FNS[Opcode::MAX * 2 + opcode](context)
    }

    fn spend_budget_and_execute(
        context: &mut Context<'_>,
        budget: &mut u32,
    ) -> JsResult<CompletionType> {
        let opcode: Opcode = context.vm.read::<u8>().into();

        *budget = budget.saturating_sub(u32::from(opcode.cost()) + u32::from(Self::COST));
        Opcode::EXECUTE_FNS[Opcode::MAX * 2 + opcode as usize](context)
    }
}
