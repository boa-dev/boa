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
    fn operation(
        value: u32,
        dst: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        let key = value.to_property_key(context)?;
        registers.set(dst, key.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for ToPropertyKey {
    const NAME: &'static str = "ToPropertyKey";
    const INSTRUCTION: &'static str = "INST - ToPropertyKey";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        let dst = context.vm.read::<u8>().into();
        Self::operation(value, dst, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        let dst = context.vm.read::<u16>().into();
        Self::operation(value, dst, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        let dst = context.vm.read::<u32>();
        Self::operation(value, dst, registers, context)
    }
}
