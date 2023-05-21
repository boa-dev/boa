use crate::{
    environments::PrivateEnvironment,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};
use boa_gc::Gc;

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
            .push_lexical(num_bindings, compile_environment);
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
            .push_function_inherit(num_bindings, compile_environment);
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

/// `PushPrivateEnvironment` implements the Opcode Operation for `Opcode::PushPrivateEnvironment`
///
/// Operation:
///  - Push a private environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushPrivateEnvironment;

impl Operation for PushPrivateEnvironment {
    const NAME: &'static str = "PushPrivateEnvironment";
    const INSTRUCTION: &'static str = "INST - PushPrivateEnvironment";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let class_value = context.vm.pop();
        let class = class_value.to_object(context)?;

        let count = context.vm.read::<u32>();
        let mut names = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let index = context.vm.read::<u32>();
            let name = context.vm.frame().code_block.private_names[index as usize];
            names.push(name.description());
        }

        let ptr: *const _ = class.as_ref();
        let environment = Gc::new(PrivateEnvironment::new(ptr as usize, names));

        class
            .borrow_mut()
            .as_function_mut()
            .expect("class object must be function")
            .push_private_environment(environment.clone());
        context.vm.environments.push_private(environment);

        context.vm.push(class_value);

        Ok(CompletionType::Normal)
    }
}

/// `PopPrivateEnvironment` implements the Opcode Operation for `Opcode::PopPrivateEnvironment`
///
/// Operation:
///  - Pop a private environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopPrivateEnvironment;

impl Operation for PopPrivateEnvironment {
    const NAME: &'static str = "PopPrivateEnvironment";
    const INSTRUCTION: &'static str = "INST - PopPrivateEnvironment";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        context.vm.environments.pop_private();

        Ok(CompletionType::Normal)
    }
}
