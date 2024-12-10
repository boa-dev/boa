use crate::{
    vm::{code_block::create_function_object_fast, opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `GetFunction` implements the Opcode Operation for `Opcode::GetFunction`
///
/// Operation:
///  - Get function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetFunction;

impl GetFunction {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        dst: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let code = context.vm.frame().code_block().constant_function(index);
        let function = create_function_object_fast(code, context);
        registers.set(dst, function.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetFunction {
    const NAME: &'static str = "GetFunction";
    const INSTRUCTION: &'static str = "INST - GetFunction";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u8>());
        let index = context.vm.read::<u8>() as usize;
        Self::operation(dst, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u16>());
        let index = context.vm.read::<u16>() as usize;
        Self::operation(dst, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(dst, index, registers, context)
    }
}
