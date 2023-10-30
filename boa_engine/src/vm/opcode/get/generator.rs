use crate::{
    vm::{create_function_object_fast, opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `GetGenerator` implements the Opcode Operation for `Opcode::GetGenerator`
///
/// Operation:
///  - Get generator function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetGenerator;

impl GetGenerator {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let code = context.vm.frame().code_block().constant_function(index);
        let function = create_function_object_fast(code, false, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetGenerator {
    const NAME: &'static str = "GetGenerator";
    const INSTRUCTION: &'static str = "INST - GetGenerator";
    const COST: u8 = 3;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}
