use super::VaryingOperand;
use crate::{Context, JsResult, vm::opcode::Operation};

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
        context: &Context,
    ) -> JsResult<()> {
        let value = context.vm_mut().get_register(value.into()).clone();
        let key = value.to_property_key(context)?;
        context.vm_mut().set_register(dst.into(), key.into());
        Ok(())
    }
}

impl Operation for ToPropertyKey {
    const NAME: &'static str = "ToPropertyKey";
    const INSTRUCTION: &'static str = "INST - ToPropertyKey";
    const COST: u8 = 2;
}
