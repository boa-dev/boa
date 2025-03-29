use super::VaryingOperand;
use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `ToPropertyKey` implements the Opcode Operation for `Opcode::ToPropertyKey`
///
/// Operation:
///  - Call `ToPropertyKey` on the value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToPropertyKey;

impl ToPropertyKey {
    #[inline(always)]
    pub(super) fn operation(
        (value, dst): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value.into());
        let key = value.to_property_key(context)?;
        registers.set(dst.into(), key.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for ToPropertyKey {
    const NAME: &'static str = "ToPropertyKey";
    const INSTRUCTION: &'static str = "INST - ToPropertyKey";
    const COST: u8 = 2;
}
