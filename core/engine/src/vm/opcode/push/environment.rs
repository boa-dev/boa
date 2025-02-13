use crate::{
    builtins::function::OrdinaryFunction,
    environments::PrivateEnvironment,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult, JsString,
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
    fn operation(index: usize, context: &mut Context) -> JsResult<CompletionType> {
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

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(index, context)
    }

    fn execute_u16(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(index, context)
    }

    fn execute_u32(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(index, context)
    }
}

/// `PushObjectEnvironment` implements the Opcode Operation for `Opcode::PushObjectEnvironment`
///
/// Operation:
///  - Push an object environment
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushObjectEnvironment;

impl PushObjectEnvironment {
    fn operation(
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(value);
        let object = object.to_object(context)?;
        context.vm.environments.push_object(object);
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushObjectEnvironment {
    const NAME: &'static str = "PushObjectEnvironment";
    const INSTRUCTION: &'static str = "INST - PushObjectEnvironment";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        Self::operation(value, registers, context)
    }
}

/// `PushPrivateEnvironment` implements the Opcode Operation for `Opcode::PushPrivateEnvironment`
///
/// Operation:
///  - Push a private environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushPrivateEnvironment;

impl PushPrivateEnvironment {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        class: u32,
        names: Vec<JsString>,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let class = registers.get(class);
        let class = class.as_object().expect("should be a object");

        let ptr: *const _ = class.as_ref();
        let environment = Gc::new(PrivateEnvironment::new(ptr.cast::<()>() as usize, names));

        class
            .downcast_mut::<OrdinaryFunction>()
            .expect("class object must be function")
            .push_private_environment(environment.clone());
        context.vm.environments.push_private(environment);

        Ok(CompletionType::Normal)
    }
}

impl Operation for PushPrivateEnvironment {
    const NAME: &'static str = "PushPrivateEnvironment";
    const INSTRUCTION: &'static str = "INST - PushPrivateEnvironment";
    const COST: u8 = 5;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let class = u32::from(context.vm.read::<u8>());
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
        Self::operation(class, names, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let class = u32::from(context.vm.read::<u16>());
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
        Self::operation(class, names, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let class = context.vm.read::<u32>();
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
        Self::operation(class, names, registers, context)
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

    fn execute(_: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        context.vm.environments.pop_private();
        Ok(CompletionType::Normal)
    }
}
