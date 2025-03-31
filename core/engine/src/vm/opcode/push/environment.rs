use crate::{
    builtins::function::OrdinaryFunction,
    environments::PrivateEnvironment,
    vm::{
        opcode::{Operation, VaryingOperand},
        Registers,
    },
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
    #[inline(always)]
    pub(crate) fn operation(index: VaryingOperand, _: &mut Registers, context: &mut Context) {
        let scope = context.vm.frame().code_block().constant_scope(index.into());
        context
            .vm
            .environments
            .push_lexical(scope.num_bindings_non_local());
    }
}

impl Operation for PushScope {
    const NAME: &'static str = "PushScope";
    const INSTRUCTION: &'static str = "INST - PushScope";
    const COST: u8 = 3;
}

/// `PushObjectEnvironment` implements the Opcode Operation for `Opcode::PushObjectEnvironment`
///
/// Operation:
///  - Push an object environment
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushObjectEnvironment;

impl PushObjectEnvironment {
    #[inline(always)]
    pub(crate) fn operation(
        value: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let object = registers.get(value.into());
        let object = object.to_object(context)?;
        context.vm.environments.push_object(object);
        Ok(())
    }
}

impl Operation for PushObjectEnvironment {
    const NAME: &'static str = "PushObjectEnvironment";
    const INSTRUCTION: &'static str = "INST - PushObjectEnvironment";
    const COST: u8 = 3;
}

/// `PushPrivateEnvironment` implements the Opcode Operation for `Opcode::PushPrivateEnvironment`
///
/// Operation:
///  - Push a private environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushPrivateEnvironment;

impl PushPrivateEnvironment {
    #[inline(always)]
    pub(crate) fn operation(
        (class, name_indices): (VaryingOperand, Vec<u32>),
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let class = registers.get(class.into());
        let class = class.as_object().expect("should be a object");
        let mut names = Vec::with_capacity(name_indices.len());
        for index in name_indices {
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
    }
}

impl Operation for PushPrivateEnvironment {
    const NAME: &'static str = "PushPrivateEnvironment";
    const INSTRUCTION: &'static str = "INST - PushPrivateEnvironment";
    const COST: u8 = 5;
}

/// `PopPrivateEnvironment` implements the Opcode Operation for `Opcode::PopPrivateEnvironment`
///
/// Operation:
///  - Pop a private environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopPrivateEnvironment;

impl PopPrivateEnvironment {
    #[inline(always)]
    pub(crate) fn operation((): (), _: &mut Registers, context: &mut Context) {
        context.vm.environments.pop_private();
    }
}

impl Operation for PopPrivateEnvironment {
    const NAME: &'static str = "PopPrivateEnvironment";
    const INSTRUCTION: &'static str = "INST - PopPrivateEnvironment";
    const COST: u8 = 1;
}
