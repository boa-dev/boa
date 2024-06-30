use crate::{
    builtins::function::arguments::{MappedArguments, UnmappedArguments},
    vm::CompletionType,
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

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let function_object = context
            .vm
            .frame()
            .function(&context.vm)
            .clone()
            .expect("there should be a function object");
        let code = context.vm.frame().code_block().clone();
        let args = context.vm.frame().arguments(&context.vm).to_vec();

        let env = context.vm.environments.current_ref();
        let arguments = MappedArguments::new(
            &function_object,
            &code.mapped_arguments_binding_indices,
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

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let args = context.vm.frame().arguments(&context.vm).to_vec();
        let arguments = UnmappedArguments::new(&args, context);
        context.vm.push(arguments);
        Ok(CompletionType::Normal)
    }
}
