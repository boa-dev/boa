use crate::{
    vm::{code_block::create_function_object_fast, opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `GetArrowFunction` implements the Opcode Operation for `Opcode::GetArrowFunction`
///
/// Operation:
///  - Get arrow function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetArrowFunction;

impl GetArrowFunction {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let code = context.vm.frame().code_block().constant_function(index);
        let function = create_function_object_fast(code, false, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetArrowFunction {
    const NAME: &'static str = "GetArrowFunction";
    const INSTRUCTION: &'static str = "INST - GetArrowFunction";
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

/// `GetFunction` implements the Opcode Operation for `Opcode::GetFunction`
///
/// Operation:
///  - Get function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetFunction;

impl GetFunction {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        context: &mut Context<'_>,
        index: usize,
        method: bool,
    ) -> JsResult<CompletionType> {
        let code = context.vm.frame().code_block().constant_function(index);
        let function = create_function_object_fast(code, method, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetFunction {
    const NAME: &'static str = "GetFunction";
    const INSTRUCTION: &'static str = "INST - GetFunction";
    const COST: u8 = 3;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        let method = context.vm.read::<u8>() != 0;
        Self::operation(context, index, method)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        let method = context.vm.read::<u8>() != 0;
        Self::operation(context, index, method)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        let method = context.vm.read::<u8>() != 0;
        Self::operation(context, index, method)
    }
}
