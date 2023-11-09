use crate::{
    vm::{code_block::create_function_object_fast, opcode::Operation, CompletionType},
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
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let code = context.vm.frame().code_block().constant_function(index);
        let function = create_function_object_fast(code, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetFunction {
    const NAME: &'static str = "GetFunction";
    const INSTRUCTION: &'static str = "INST - GetFunction";
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}
