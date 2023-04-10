use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `PushDeclarativeEnvironment` implements the Opcode Operation for `Opcode::PushDeclarativeEnvironment`
///
/// Operation:
///  - Push a declarative environment
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushDeclarativeEnvironment;

impl Operation for PushDeclarativeEnvironment {
    const NAME: &'static str = "PushDeclarativeEnvironment";
    const INSTRUCTION: &'static str = "INST - PushDeclarativeEnvironment";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let num_bindings = context.vm.read::<u32>();
        let compile_environments_index = context.vm.read::<u32>();
        let compile_environment = context.vm.frame().code_block.compile_environments
            [compile_environments_index as usize]
            .clone();
        context
            .vm
            .environments
            .push_declarative(num_bindings as usize, compile_environment);
        context.vm.frame_mut().inc_frame_env_stack();
        Ok(CompletionType::Normal)
    }
}

/// `PushFunctionEnvironment` implements the Opcode Operation for `Opcode::PushFunctionEnvironment`
///
/// Operation:
///  - Push a function environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushFunctionEnvironment;

impl Operation for PushFunctionEnvironment {
    const NAME: &'static str = "PushFunctionEnvironment";
    const INSTRUCTION: &'static str = "INST - PushFunctionEnvironment";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let num_bindings = context.vm.read::<u32>();
        let compile_environments_index = context.vm.read::<u32>();
        let compile_environment = context.vm.frame().code_block.compile_environments
            [compile_environments_index as usize]
            .clone();
        context
            .vm
            .environments
            .push_function_inherit(num_bindings as usize, compile_environment);
        Ok(CompletionType::Normal)
    }
}

/// `PushObjectEnvironment` implements the Opcode Operation for `Opcode::PushObjectEnvironment`
///
/// Operation:
///  - Push an object environment
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushObjectEnvironment;

impl Operation for PushObjectEnvironment {
    const NAME: &'static str = "PushObjectEnvironment";
    const INSTRUCTION: &'static str = "INST - PushObjectEnvironment";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let object = context.vm.pop();
        let object = object.to_object(context)?;

        context.vm.environments.push_object(object);
        context.vm.frame_mut().inc_frame_env_stack();
        Ok(CompletionType::Normal)
    }
}
