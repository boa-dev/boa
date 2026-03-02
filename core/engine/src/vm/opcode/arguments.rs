use super::{Operation, VaryingOperand};
use crate::{
    Context,
    builtins::function::arguments::{MappedArguments, UnmappedArguments},
};

/// `CreateMappedArgumentsObject` implements the Opcode Operation for `Opcode::CreateMappedArgumentsObject`
///
/// Operation:
///  - Create a mapped arguments object and store it in a register.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateMappedArgumentsObject;

impl CreateMappedArgumentsObject {
    #[inline(always)]
    pub(super) fn operation(value: VaryingOperand, context: &Context) {
        let (function_object, code, args, env) = context.with_vm(|vm| {
            let function_object = vm
                .stack
                .get_function(&vm.frame)
                .expect("there should be a function object");
            let code = vm.frame.code_block().clone();
            let args = vm.stack.get_arguments(&vm.frame).to_vec();
            let env = vm
                .frame
                .environments
                .current_declarative_ref()
                .expect("must be declarative")
                .clone();
            (function_object, code, args, env)
        });
        let arguments = MappedArguments::new(
            &function_object,
            &code.mapped_arguments_binding_indices,
            &args,
            &env,
            context,
        );
        context.set_register(value.into(), arguments.into());
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
///  - Create an unmapped arguments object and store it in a register.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateUnmappedArgumentsObject;

impl CreateUnmappedArgumentsObject {
    #[inline(always)]
    pub(super) fn operation(dst: VaryingOperand, context: &Context) {
        let args = context.with_vm(|vm| vm.stack.get_arguments(&vm.frame).to_vec());
        let arguments = UnmappedArguments::new(&args, context);
        context.set_register(dst.into(), arguments.into());
    }
}

impl Operation for CreateUnmappedArgumentsObject {
    const NAME: &'static str = "CreateUnmappedArgumentsObject";
    const INSTRUCTION: &'static str = "INST - CreateUnmappedArgumentsObject";
    const COST: u8 = 4;
}
