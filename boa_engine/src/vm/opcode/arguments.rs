use crate::{
    builtins::function::arguments::Arguments,
    vm::{CallFrame, CompletionType},
    Context, JsResult,
};

use super::Operation;

/// `CreateMappedArgumentsObject` implements the Opcode Operation for `Opcode::CreateMappedArgumentsObject`
///
/// Operation:
///  - TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateMappedArgumentsObject;

impl Operation for CreateMappedArgumentsObject {
    const NAME: &'static str = "CreateMappedArgumentsObject";
    const INSTRUCTION: &'static str = "INST - CreateMappedArgumentsObject";
    const COST: u8 = 8;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let arguments_start = context.vm.frame().fp as usize + CallFrame::FIRST_ARGUMENT_POSITION;
        let function_object = context
            .vm
            .frame()
            .function(&context.vm)
            .clone()
            .expect("there should be a function object");
        let code = context.vm.frame().code_block().clone();
        let args = context.vm.stack[arguments_start..].to_vec();

        let env = context.vm.environments.current();
        let arguments = Arguments::create_mapped_arguments_object(
            &function_object,
            &code.params,
            &args,
            env.declarative_expect(),
            context,
        );
        context.vm.push(arguments);
        Ok(CompletionType::Normal)
    }
}

/// `CreateUnmappedArgumentsObject` implements the Opcode Operation for `Opcode::CreateUnmappedArgumentsObject`
///
/// Operation:
///  - TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateUnmappedArgumentsObject;

impl Operation for CreateUnmappedArgumentsObject {
    const NAME: &'static str = "CreateUnmappedArgumentsObject";
    const INSTRUCTION: &'static str = "INST - CreateUnmappedArgumentsObject";
    const COST: u8 = 4;

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let arguments_start = context.vm.frame().fp as usize + CallFrame::FIRST_ARGUMENT_POSITION;
        let args = context.vm.stack[arguments_start..].to_vec();
        let arguments = Arguments::create_unmapped_arguments_object(&args, context);
        context.vm.push(arguments);
        Ok(CompletionType::Normal)
    }
}
