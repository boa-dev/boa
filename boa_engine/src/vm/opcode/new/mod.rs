use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType},
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        if raw_context.vm.runtime_limits.recursion_limit() <= raw_context.vm.frames.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message(format!(
                    "Maximum recursion limit {} exceeded",
                    raw_context.vm.runtime_limits.recursion_limit()
                ))
                .into());
        }
        if raw_context.vm.runtime_limits.stack_size_limit() <= raw_context.vm.stack.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message("Maximum call stack size exceeded")
                .into());
        }
        let argument_count = raw_context.vm.read::<u32>();
        let mut arguments = Vec::with_capacity(argument_count as usize);
        for _ in 0..argument_count {
            arguments.push(raw_context.vm.pop());
        }
        arguments.reverse();
        let func = raw_context.vm.pop();

        let result = func
            .as_constructor()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("not a constructor")
                    .into()
            })
            .and_then(|cons| cons.__construct__(&arguments, cons, context))?;

        context.as_raw_context_mut().vm.push(result);
        Ok(CompletionType::Normal)
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        if raw_context.vm.runtime_limits.recursion_limit() <= raw_context.vm.frames.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message(format!(
                    "Maximum recursion limit {} exceeded",
                    raw_context.vm.runtime_limits.recursion_limit()
                ))
                .into());
        }
        if raw_context.vm.runtime_limits.stack_size_limit() <= raw_context.vm.stack.len() {
            return Err(JsNativeError::runtime_limit()
                .with_message("Maximum call stack size exceeded")
                .into());
        }
        // Get the arguments that are stored as an array object on the stack.
        let arguments_array = raw_context.vm.pop();
        let arguments_array_object = arguments_array
            .as_object()
            .expect("arguments array in call spread function must be an object");
        let arguments = arguments_array_object
            .borrow()
            .properties()
            .dense_indexed_properties()
            .expect("arguments array in call spread function must be dense")
            .clone();

        let func = raw_context.vm.pop();

        let result = func
            .as_constructor()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("not a constructor")
                    .into()
            })
            .and_then(|cons| cons.__construct__(&arguments, cons, context))?;

        context.as_raw_context_mut().vm.push(result);
        Ok(CompletionType::Normal)
    }
}
