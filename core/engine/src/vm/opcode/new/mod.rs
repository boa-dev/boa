use super::VaryingOperand;
use crate::{Context, JsResult, error::JsNativeError, vm::opcode::Operation};

/// `New` implements the Opcode Operation for `Opcode::New`
///
/// Operation:
///  - Call construct on a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct New;

impl New {
    #[inline(always)]
    pub(super) fn operation(argument_count: VaryingOperand, context: &Context) -> JsResult<()> {
        let func = context.with_vm(|vm| {
            vm.stack
                .calling_convention_get_function(argument_count.into())
                .clone()
        });

        let cons = func
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("not a constructor"))?
            .clone();

        context.stack_push(cons.clone()); // Push new.target

        cons.__construct__(argument_count.into()).resolve(context)?;
        Ok(())
    }
}

impl Operation for New {
    const NAME: &'static str = "New";
    const INSTRUCTION: &'static str = "INST - New";
    const COST: u8 = 3;
}

/// `NewSpread` implements the Opcode Operation for `Opcode::NewSpread`
///
/// Operation:
///  - Call construct on a function where the arguments contain spreads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NewSpread;

impl NewSpread {
    #[inline(always)]
    pub(super) fn operation((): (), context: &Context) -> JsResult<()> {
        // Get the arguments that are stored as an array object on the stack.
        let arguments_array = context.stack_pop();
        let arguments_array_object = arguments_array
            .as_object()
            .expect("arguments array in call spread function must be an object");
        let arguments = arguments_array_object
            .borrow()
            .properties()
            .to_dense_indexed_properties()
            .expect("arguments array in call spread function must be dense");

        let func = context.stack_pop();

        let cons = func
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("not a constructor"))?
            .clone();

        let argument_count = arguments.len();
        context.stack_push(func);
        context.with_vm_mut(|vm| vm.stack.calling_convention_push_arguments(&arguments));
        context.stack_push(cons.clone()); // Push new.target

        cons.__construct__(argument_count).resolve(context)?;
        Ok(())
    }
}

impl Operation for NewSpread {
    const NAME: &'static str = "NewSpread";
    const INSTRUCTION: &'static str = "INST - NewSpread";
    const COST: u8 = 3;
}
