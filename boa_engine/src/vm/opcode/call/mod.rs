use crate::{
    builtins::function::Function,
    error::JsNativeError,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsValue,
};

/// `CallEval` implements the Opcode Operation for `Opcode::CallEval`
///
/// Operation:
///  - Call a function named "eval".
#[derive(Debug, Clone, Copy)]
pub(crate) struct CallEval;

impl Operation for CallEval {
    const NAME: &'static str = "CallEval";
    const INSTRUCTION: &'static str = "INST - CallEval";

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
        let this = context.vm.pop();

        let object = match func {
            JsValue::Object(ref object) if object.is_callable() => object.clone(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("not a callable function")
                    .into())
            }
        };

        // A native function with the name "eval" implies, that is this the built-in eval function.
        let eval = matches!(object.borrow().as_function(), Some(Function::Native { .. }));

        let strict = context.vm.frame().code.strict;

        if eval {
            if let Some(x) = arguments.get(0) {
                let result = crate::builtins::eval::Eval::perform_eval(x, true, strict, context)?;
                context.vm.push(result);
            } else {
                context.vm.push(JsValue::Undefined);
            }
        } else {
            let result = object.__call__(&this, &arguments, context)?;
            context.vm.push(result);
        }
        Ok(ShouldExit::False)
    }
}

/// `CallEvalSpread` implements the Opcode Operation for `Opcode::CallEvalSpread`
///
/// Operation:
///  - Call a function named "eval" where the arguments contain spreads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CallEvalSpread;

impl Operation for CallEvalSpread {
    const NAME: &'static str = "CallEvalSpread";
    const INSTRUCTION: &'static str = "INST - CallEvalSpread";

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
        let this = context.vm.pop();

        let object = match func {
            JsValue::Object(ref object) if object.is_callable() => object.clone(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("not a callable function")
                    .into())
            }
        };

        // A native function with the name "eval" implies, that is this the built-in eval function.
        let eval = matches!(object.borrow().as_function(), Some(Function::Native { .. }));

        let strict = context.vm.frame().code.strict;

        if eval {
            if let Some(x) = arguments.get(0) {
                let result = crate::builtins::eval::Eval::perform_eval(x, true, strict, context)?;
                context.vm.push(result);
            } else {
                context.vm.push(JsValue::Undefined);
            }
        } else {
            let result = object.__call__(&this, &arguments, context)?;
            context.vm.push(result);
        }
        Ok(ShouldExit::False)
    }
}

/// `Call` implements the Opcode Operation for `Opcode::Call`
///
/// Operation:
///  - Call a function
#[derive(Debug, Clone, Copy)]
pub(crate) struct Call;

impl Operation for Call {
    const NAME: &'static str = "Call";
    const INSTRUCTION: &'static str = "INST - Call";

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
        let this = context.vm.pop();

        let object = match func {
            JsValue::Object(ref object) if object.is_callable() => object.clone(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("not a callable function")
                    .into())
            }
        };

        let result = object.__call__(&this, &arguments, context)?;

        context.vm.push(result);
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CallSpread;

impl Operation for CallSpread {
    const NAME: &'static str = "CallSpread";
    const INSTRUCTION: &'static str = "INST - CallSpread";

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
        let this = context.vm.pop();

        let object = match func {
            JsValue::Object(ref object) if object.is_callable() => object.clone(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("not a callable function")
                    .into())
            }
        };

        let result = object.__call__(&this, &arguments, context)?;

        context.vm.push(result);
        Ok(ShouldExit::False)
    }
}
