use super::VaryingOperand;
use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, Registers},
    Context, JsResult,
};

/// `New` implements the Opcode Operation for `Opcode::New`
///
/// Operation:
///  - Call construct on a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct New;

impl New {
    #[inline(always)]
    pub(super) fn operation(
        argument_count: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let argument_count = usize::from(argument_count);
        let at = context.vm.stack.len() - argument_count;
        let func = &context.vm.stack[at - 1];

        let cons = func
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("not a constructor"))?
            .clone();

        context.vm.push(cons.clone()); // Push new.target

        if let Some(register_count) = cons.__construct__(argument_count).resolve(context)? {
            registers.push_function(register_count);
        }
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
    pub(super) fn operation(
        (): (),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        // Get the arguments that are stored as an array object on the stack.
        let arguments_array = context.vm.pop();
        let arguments_array_object = arguments_array
            .as_object()
            .expect("arguments array in call spread function must be an object");
        let arguments = arguments_array_object
            .borrow()
            .properties()
            .to_dense_indexed_properties()
            .expect("arguments array in call spread function must be dense");

        let func = context.vm.pop();

        let cons = func
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("not a constructor"))?
            .clone();

        let argument_count = arguments.len();
        context.vm.push(func);
        context.vm.push_values(&arguments);
        context.vm.push(cons.clone()); // Push new.target

        if let Some(register_count) = cons.__construct__(argument_count).resolve(context)? {
            registers.push_function(register_count);
        }
        Ok(())
    }
}

impl Operation for NewSpread {
    const NAME: &'static str = "NewSpread";
    const INSTRUCTION: &'static str = "INST - NewSpread";
    const COST: u8 = 3;
}
