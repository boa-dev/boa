use crate::{
    builtins::function::OrdinaryFunction,
    environments::PrivateEnvironment,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsString,
};
use boa_gc::Gc;

/// `PushDeclarativeEnvironment` implements the Opcode Operation for `Opcode::PushDeclarativeEnvironment`
///
/// Operation:
///  - Push a declarative environment
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushDeclarativeEnvironment;

impl PushDeclarativeEnvironment {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        context: &mut Context,
        compile_environments_index: usize,
    ) -> JsResult<CompletionType> {
        let compile_environment = context
            .vm
            .frame()
            .code_block()
            .constant_compile_time_environment(compile_environments_index);
        context.vm.environments.push_lexical(compile_environment);
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushDeclarativeEnvironment {
    const NAME: &'static str = "PushDeclarativeEnvironment";
    const INSTRUCTION: &'static str = "INST - PushDeclarativeEnvironment";
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let compile_environments_index = context.vm.read::<u8>() as usize;
        Self::operation(context, compile_environments_index)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let compile_environments_index = context.vm.read::<u16>() as usize;
        Self::operation(context, compile_environments_index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let compile_environments_index = context.vm.read::<u32>() as usize;
        Self::operation(context, compile_environments_index)
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

impl PushPrivateEnvironment {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        class: u32,
        names: Vec<JsString>,
        operand_types: u8,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let class_value = context
            .vm
            .frame()
            .read_value::<0>(operand_types, class, &context.vm);
        let class = class_value.as_object().expect("should be a object");

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

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
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
        Self::operation(class, names, operand_types, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
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
        Self::operation(class, names, operand_types, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
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
        Self::operation(class, names, operand_types, context)
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
