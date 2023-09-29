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
        let code = context.vm.frame().code_block.functions[index].clone();
        let function = create_function_object_fast(code, false, true, false, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetArrowFunction {
    const NAME: &'static str = "GetArrowFunction";
    const INSTRUCTION: &'static str = "INST - GetArrowFunction";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn u16_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn u32_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `GetAsyncArrowFunction` implements the Opcode Operation for `Opcode::GetAsyncArrowFunction`
///
/// Operation:
///  - Get async arrow function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetAsyncArrowFunction;

impl GetAsyncArrowFunction {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let code = context.vm.frame().code_block.functions[index].clone();
        let function = create_function_object_fast(code, true, true, false, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetAsyncArrowFunction {
    const NAME: &'static str = "GetAsyncArrowFunction";
    const INSTRUCTION: &'static str = "INST - GetAsyncArrowFunction";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn u16_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn u32_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
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
        let code = context.vm.frame().code_block.functions[index].clone();
        let function = create_function_object_fast(code, false, false, method, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetFunction {
    const NAME: &'static str = "GetFunction";
    const INSTRUCTION: &'static str = "INST - GetFunction";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        let method = context.vm.read::<u8>() != 0;
        Self::operation(context, index, method)
    }

    fn u16_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        let method = context.vm.read::<u8>() != 0;
        Self::operation(context, index, method)
    }

    fn u32_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        let method = context.vm.read::<u8>() != 0;
        Self::operation(context, index, method)
    }
}

/// `GetFunctionAsync` implements the Opcode Operation for `Opcode::GetFunctionAsync`
///
/// Operation:
///  - Get async function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetFunctionAsync;

impl GetFunctionAsync {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        context: &mut Context<'_>,
        index: usize,
        method: bool,
    ) -> JsResult<CompletionType> {
        let code = context.vm.frame().code_block.functions[index].clone();
        let function = create_function_object_fast(code, true, false, method, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetFunctionAsync {
    const NAME: &'static str = "GetFunctionAsync";
    const INSTRUCTION: &'static str = "INST - GetFunctionAsync";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        let method = context.vm.read::<u8>() != 0;
        Self::operation(context, index, method)
    }

    fn u16_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        let method = context.vm.read::<u8>() != 0;
        Self::operation(context, index, method)
    }

    fn u32_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        let method = context.vm.read::<u8>() != 0;
        Self::operation(context, index, method)
    }
}
