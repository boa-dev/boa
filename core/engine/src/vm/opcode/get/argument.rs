use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `GetArgument` implements the Opcode Operation for `Opcode::GetArgument`
///
/// Operation:
///  - Get i-th argument of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetArgument;

impl GetArgument {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        index: usize,
        dst: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = context
            .vm
            .frame()
            .argument(index, &context.vm)
            .cloned()
            .unwrap_or_default();
        registers.set(dst, value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetArgument {
    const NAME: &'static str = "GetArgument";
    const INSTRUCTION: &'static str = "INST - GetArgument";
    const COST: u8 = 2;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        let dst = context.vm.read::<u8>().into();
        Self::operation(index, dst, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        let dst = context.vm.read::<u16>().into();
        Self::operation(index, dst, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        let dst = context.vm.read::<u32>();
        Self::operation(index, dst, registers, context)
    }
}
