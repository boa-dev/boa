use crate::{
    vm::{code_block::create_generator_function_object, opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `GetGenerator` implements the Opcode Operation for `Opcode::GetGenerator`
///
/// Operation:
///  - Get generator function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetGenerator;

impl Operation for GetGenerator {
    const NAME: &'static str = "GetGenerator";
    const INSTRUCTION: &'static str = "INST - GetGenerator";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let index = raw_context.vm.read::<u32>();
        let code = raw_context.vm.frame().code_block.functions[index as usize].clone();
        let function = create_generator_function_object(code, false, None, context);
        context.as_raw_context_mut().vm.push(function);
        Ok(CompletionType::Normal)
    }
}

/// `GetGeneratorAsync` implements the Opcode Operation for `Opcode::GetGeneratorAsync`
///
/// Operation:
///  - Get async generator function from the pre-compiled inner functions.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetGeneratorAsync;

impl Operation for GetGeneratorAsync {
    const NAME: &'static str = "GetGeneratorAsync";
    const INSTRUCTION: &'static str = "INST - GetGeneratorAsync";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let index = raw_context.vm.read::<u32>();
        let code = raw_context.vm.frame().code_block.functions[index as usize].clone();
        let function = create_generator_function_object(code, true, None, context);
        context.as_raw_context_mut().vm.push(function);
        Ok(CompletionType::Normal)
    }
}
