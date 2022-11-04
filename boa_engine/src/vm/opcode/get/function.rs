use crate::{
    vm::{code_block::create_function_object, opcode::Operation, ShouldExit},
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let code = context.vm.frame().code.functions[index as usize].clone();
        let function = create_function_object(code, false, true, None, context);
        context.vm.push(function);
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let code = context.vm.frame().code.functions[index as usize].clone();
        let function = create_function_object(code, true, true, None, context);
        context.vm.push(function);
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let code = context.vm.frame().code.functions[index as usize].clone();
        let function = create_function_object(code, false, false, None, context);
        context.vm.push(function);
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let code = context.vm.frame().code.functions[index as usize].clone();
        let function = create_function_object(code, true, false, None, context);
        context.vm.push(function);
        Ok(ShouldExit::False)
    }
}
