use crate::{
    vm::{code_block::create_function_object, opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `GetFunction` implements the Opcode Operation for `Opcode::GetFunction`
///
/// Operation:
///  - Get function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GetFunction;

impl Operation for GetFunction {
    const NAME: &'static str = "GetFunction";
    const INSTRUCTION: &'static str = "INST - GetFunction";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let code = context.vm.frame().code.functions[index as usize].clone();
        let function = create_function_object(code, false, None, context);
        context.vm.push(function);
        Ok(ShouldExit::False)
    }
}

/// `GetFunctionAsync` implements the Opcode Operation for `Opcode::GetFunctionAsync`
///
/// Operation:
///  - Get async function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GetFunctionAsync;

impl Operation for GetFunctionAsync {
    const NAME: &'static str = "GetFunctionAsync";
    const INSTRUCTION: &'static str = "INST - GetFunctionAsync";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let code = context.vm.frame().code.functions[index as usize].clone();
        let function = create_function_object(code, true, None, context);
        context.vm.push(function);
        Ok(ShouldExit::False)
    }
}
