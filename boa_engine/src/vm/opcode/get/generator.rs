use crate::{
    vm::{code_block::create_generator_function_object, opcode::Operation, ShouldExit},
    Context, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GetGenerator;

impl Operation for GetGenerator {
    const NAME: &'static str = "GetGenerator";
    const INSTRUCTION: &'static str = "INST - GetGenerator";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let code = context.vm.frame().code.functions[index as usize].clone();
        let function = create_generator_function_object(code, false, context);
        context.vm.push(function);
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GetGeneratorAsync;

impl Operation for GetGeneratorAsync {
    const NAME: &'static str = "GetGeneratorAsync";
    const INSTRUCTION: &'static str = "INST - GetGeneratorAsync";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let code = context.vm.frame().code.functions[index as usize].clone();
        let function = create_generator_function_object(code, true, context);
        context.vm.push(function);
        Ok(ShouldExit::False)
    }
}
