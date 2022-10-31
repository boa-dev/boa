use crate::{
    vm::{opcode::Operation, ShouldExit},
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let num_bindings = context.vm.read::<u32>();
        let compile_environments_index = context.vm.read::<u32>();
        let compile_environment = context.vm.frame().code.compile_environments
            [compile_environments_index as usize]
            .clone();
        context
            .realm
            .environments
            .push_declarative(num_bindings as usize, compile_environment);
        context.vm.frame_mut().loop_env_stack_inc();
        context.vm.frame_mut().try_env_stack_inc();
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let num_bindings = context.vm.read::<u32>();
        let compile_environments_index = context.vm.read::<u32>();
        let compile_environment = context.vm.frame().code.compile_environments
            [compile_environments_index as usize]
            .clone();
        context
            .realm
            .environments
            .push_function_inherit(num_bindings as usize, compile_environment);
        Ok(ShouldExit::False)
    }
}
