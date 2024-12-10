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
    fn operation(
        address: u32,
        value: u32,
        condition: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        let condition = registers.get(condition);
        if value.strict_equals(condition) {
            context.vm.frame_mut().pc = address;
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for Case {
    const NAME: &'static str = "Case";
    const INSTRUCTION: &'static str = "INST - Case";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u8>().into();
        let condition = context.vm.read::<u8>().into();
        Self::operation(address, value, condition, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u16>().into();
        let condition = context.vm.read::<u16>().into();
        Self::operation(address, value, condition, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        let condition = context.vm.read::<u32>();
        Self::operation(address, value, condition, registers, context)
    }
}
