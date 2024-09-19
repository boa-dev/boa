use crate::{
    builtins::function::OrdinaryFunction,
    environments::PrivateEnvironment,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};
use boa_gc::Gc;

/// `PushScope` implements the Opcode Operation for `Opcode::PushScope`
///
/// Operation:
///  - Push a declarative environment
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushScope;

impl PushScope {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let scope = context.vm.frame().code_block().constant_scope(index);
        context
            .vm
            .environments
            .push_lexical(scope.num_bindings_non_local());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushScope {
    const NAME: &'static str = "PushScope";
    const INSTRUCTION: &'static str = "INST - PushScope";
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
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
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.pop();
        let object = object.to_object(context)?;

        context.vm.environments.push_object(object);
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
    const COST: u8 = 5;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let class_value = context.vm.pop();
        let class = class_value.to_object(context)?;

        let count = context.vm.read::<u32>();
        let mut names = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let index = context.vm.read::<u32>();
            let name = context
                .vm
                .frame()
                .code_block()
                .constant_string(index as usize);
            names.push(name);
        }

        let ptr: *const _ = class.as_ref();
        let environment = Gc::new(PrivateEnvironment::new(ptr.cast::<()>() as usize, names));

        class
            .downcast_mut::<OrdinaryFunction>()
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
    const COST: u8 = 1;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        context.vm.environments.pop_private();
        Ok(CompletionType::Normal)
    }
}
