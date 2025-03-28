use super::VaryingOperand;
use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `Case` implements the Opcode Operation for `Opcode::Case`
///
/// Operation:
///  - Pop the two values of the stack, strict equal compares the two values,
///    if true jumps to address, otherwise push the second pop'ed value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Case;

impl Case {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn operation(
        (address, value, condition): (VaryingOperand, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value.into());
        let condition = registers.get(condition.into());
        if value.strict_equals(condition) {
            context.vm.frame_mut().pc = address.into();
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for Case {
    const NAME: &'static str = "Case";
    const INSTRUCTION: &'static str = "INST - Case";
    const COST: u8 = 2;
}
