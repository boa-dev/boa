use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `New` implements the Opcode Operation for `Opcode::New`
///
/// Operation:
///  - Call construct on a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct New;

impl Operation for New {
    const NAME: &'static str = "New";
    const INSTRUCTION: &'static str = "INST - New";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        if context.vm.stack_size_limit <= context.vm.stack.len() {
            return Err(JsNativeError::range()
                .with_message("Maximum call stack size exceeded")
                .into());
        }
        let argument_count = context.vm.read::<u32>();
        let mut arguments = Vec::with_capacity(argument_count as usize);
        for _ in 0..argument_count {
            arguments.push(context.vm.pop());
        }
        arguments.reverse();
        let func = context.vm.pop();

        let result = func
            .as_constructor()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("not a constructor")
                    .into()
            })
            .and_then(|cons| cons.__construct__(&arguments, cons, context))?;

        context.vm.push(result);
        Ok(ShouldExit::False)
    }
}

/// `NewSpread` implements the Opcode Operation for `Opcode::NewSpread`
///
/// Operation:
///  - Call construct on a function where the arguments contain spreads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NewSpread;

impl Operation for NewSpread {
    const NAME: &'static str = "NewSpread";
    const INSTRUCTION: &'static str = "INST - NewSpread";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        if context.vm.stack_size_limit <= context.vm.stack.len() {
            return Err(JsNativeError::range()
                .with_message("Maximum call stack size exceeded")
                .into());
        }
        // Get the arguments that are stored as an array object on the stack.
        let arguments_array = context.vm.pop();
        let arguments_array_object = arguments_array
            .as_object()
            .expect("arguments array in call spread function must be an object");
        let arguments = arguments_array_object
            .borrow()
            .properties()
            .dense_indexed_properties()
            .expect("arguments array in call spread function must be dense")
            .clone();

        let func = context.vm.pop();

        let result = func
            .as_constructor()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("not a constructor")
                    .into()
            })
            .and_then(|cons| cons.__construct__(&arguments, cons, context))?;

        context.vm.push(result);
        Ok(ShouldExit::False)
    }
}
