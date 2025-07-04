use super::{Operation, VaryingOperand};
use crate::{
    builtins::function::arguments::{MappedArguments, UnmappedArguments},
    Context,
};

/// `CreateMappedArgumentsObject` implements the Opcode Operation for `Opcode::CreateMappedArgumentsObject`
///
/// Operation:
///  - TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateMappedArgumentsObject;

impl CreateMappedArgumentsObject {
    #[inline(always)]
    pub(super) fn operation(value: VaryingOperand, context: &mut Context) {
        let frame = context.vm.frame();
        let function_object = context
            .vm
            .stack
            .get_function(context.vm.frame())
            .expect("there should be a function object");
        let code = frame.code_block().clone();
        let args = context.vm.stack.get_arguments(context.vm.frame());
        let env = context
            .vm
            .environments
            .current_declarative_ref()
            .expect("must be declarative");
        let arguments = MappedArguments::new(
            &function_object,
            &code.mapped_arguments_binding_indices,
            args,
            env,
            context,
        );
        context.vm.set_register(value.into(), arguments.into());
    }
}

impl Operation for CreateMappedArgumentsObject {
    const NAME: &'static str = "CreateMappedArgumentsObject";
    const INSTRUCTION: &'static str = "INST - CreateMappedArgumentsObject";
    const COST: u8 = 8;
}

/// `CreateUnmappedArgumentsObject` implements the Opcode Operation for `Opcode::CreateUnmappedArgumentsObject`
///
/// Operation:
///  - TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateUnmappedArgumentsObject;

impl CreateUnmappedArgumentsObject {
    #[inline(always)]
    pub(super) fn operation(dst: VaryingOperand, context: &mut Context) {
        let args = context.vm.stack.get_arguments(context.vm.frame()).to_vec();
        let arguments = UnmappedArguments::new(&args, context);
        context.vm.set_register(dst.into(), arguments.into());
    }
}

impl Operation for CreateUnmappedArgumentsObject {
    const NAME: &'static str = "CreateUnmappedArgumentsObject";
    const INSTRUCTION: &'static str = "INST - CreateUnmappedArgumentsObject";
    const COST: u8 = 4;
}
