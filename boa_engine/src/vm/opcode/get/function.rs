use crate::{
    vm::{code_block::create_function_object, opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `GetArrowFunction` implements the Opcode Operation for `Opcode::GetArrowFunction`
///
/// Operation:
///  - Get arrow function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetArrowFunction;

impl Operation for GetArrowFunction {
    const NAME: &'static str = "GetArrowFunction";
    const INSTRUCTION: &'static str = "INST - GetArrowFunction";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        context.vm.read::<u8>();
        let code = context.vm.frame().code_block.functions[index as usize].clone();
        let function = create_function_object(code, false, true, None, false, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

/// `GetAsyncArrowFunction` implements the Opcode Operation for `Opcode::GetAsyncArrowFunction`
///
/// Operation:
///  - Get async arrow function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetAsyncArrowFunction;

impl Operation for GetAsyncArrowFunction {
    const NAME: &'static str = "GetAsyncArrowFunction";
    const INSTRUCTION: &'static str = "INST - GetAsyncArrowFunction";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        context.vm.read::<u8>();
        let code = context.vm.frame().code_block.functions[index as usize].clone();
        let function = create_function_object(code, true, true, None, false, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

/// `GetFunction` implements the Opcode Operation for `Opcode::GetFunction`
///
/// Operation:
///  - Get function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetFunction;

impl Operation for GetFunction {
    const NAME: &'static str = "GetFunction";
    const INSTRUCTION: &'static str = "INST - GetFunction";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let method = context.vm.read::<u8>() != 0;
        let code = context.vm.frame().code_block.functions[index as usize].clone();
        let function = create_function_object(code, false, false, None, method, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}

/// `GetFunctionAsync` implements the Opcode Operation for `Opcode::GetFunctionAsync`
///
/// Operation:
///  - Get async function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetFunctionAsync;

impl Operation for GetFunctionAsync {
    const NAME: &'static str = "GetFunctionAsync";
    const INSTRUCTION: &'static str = "INST - GetFunctionAsync";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let method = context.vm.read::<u8>() != 0;
        let code = context.vm.frame().code_block.functions[index as usize].clone();
        let function = create_function_object(code, true, false, None, method, context);
        context.vm.push(function);
        Ok(CompletionType::Normal)
    }
}
