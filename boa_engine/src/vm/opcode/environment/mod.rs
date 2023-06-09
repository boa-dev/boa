use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

/// `This` implements the Opcode Operation for `Opcode::This`
///
/// Operation:
///  - Pushes `this` value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct This;

impl Operation for This {
    const NAME: &'static str = "This";
    const INSTRUCTION: &'static str = "INST - This";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let context = context.as_raw_context_mut();
        let this = context.vm.environments.get_this_binding()?;
        context.vm.push(this);
        Ok(CompletionType::Normal)
    }
}

/// `Super` implements the Opcode Operation for `Opcode::Super`
///
/// Operation:
///  - Pushes the current `super` value to the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Super;

impl Operation for Super {
    const NAME: &'static str = "Super";
    const INSTRUCTION: &'static str = "INST - Super";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let home_object = {
            let env = context
                .as_raw_context()
                .vm
                .environments
                .get_this_environment()
                .as_function()
                .expect("super access must be in a function environment");
            let this = env
                .get_this_binding()?
                .expect("`get_this_environment` ensures this returns `Some`");
            let function_object = env.slots().function_object().borrow();
            let function = function_object
                .as_function()
                .expect("must be function object");
            function.get_home_object().or(this.as_object()).cloned()
        };

        let value = home_object
            .map(|o| o.__get_prototype_of__(context))
            .transpose()?
            .flatten()
            .map_or_else(JsValue::null, JsValue::from);

        context.as_raw_context_mut().vm.push(value);
        Ok(CompletionType::Normal)
    }
}

/// `SuperCallPrepare` implements the Opcode Operation for `Opcode::SuperCallPrepare`
///
/// Operation:
///  - Get the super constructor and the new target of the current environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCallPrepare;

impl Operation for SuperCallPrepare {
    const NAME: &'static str = "SuperCallPrepare";
    const INSTRUCTION: &'static str = "INST - SuperCallPrepare";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let this_env = context
            .as_raw_context()
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");
        let new_target = this_env
            .slots()
            .new_target()
            .expect("must have new target")
            .clone();
        let active_function = this_env.slots().function_object().clone();
        let super_constructor = active_function
            .__get_prototype_of__(context)
            .expect("function object must have prototype");

        let context = context.as_raw_context_mut();
        if let Some(constructor) = super_constructor {
            context.vm.push(constructor);
        } else {
            context.vm.push(JsValue::Null);
        }
        context.vm.push(new_target);

        Ok(CompletionType::Normal)
    }
}

/// `SuperCall` implements the Opcode Operation for `Opcode::SuperCall`
///
/// Operation:
///  - Execute the `super()` method.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCall;

impl Operation for SuperCall {
    const NAME: &'static str = "SuperCall";
    const INSTRUCTION: &'static str = "INST - SuperCall";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let argument_count = raw_context.vm.read::<u32>();
        let mut arguments = Vec::with_capacity(argument_count as usize);
        for _ in 0..argument_count {
            arguments.push(raw_context.vm.pop());
        }
        arguments.reverse();

        let new_target_value = raw_context.vm.pop();
        let super_constructor = raw_context.vm.pop();

        let new_target = new_target_value
            .as_object()
            .expect("new target must be object");

        let Some(super_constructor) = super_constructor.as_constructor() else {
            return Err(JsNativeError::typ()
                .with_message("super constructor object must be constructor")
                .into());
        };

        let result = super_constructor.__construct__(&arguments, new_target, context)?;

        let this_env = context
            .as_raw_context()
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");

        this_env.bind_this_value(result.clone())?;
        let function_object = this_env.slots().function_object().clone();

        result.initialize_instance_elements(&function_object, context)?;

        context.as_raw_context_mut().vm.push(result);
        Ok(CompletionType::Normal)
    }
}

/// `SuperCallSpread` implements the Opcode Operation for `Opcode::SuperCallSpread`
///
/// Operation:
///  - Execute the `super()` method where the arguments contain spreads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCallSpread;

impl Operation for SuperCallSpread {
    const NAME: &'static str = "SuperCallWithRest";
    const INSTRUCTION: &'static str = "INST - SuperCallWithRest";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
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

        let new_target_value = raw_context.vm.pop();
        let super_constructor = raw_context.vm.pop();

        let new_target = new_target_value
            .as_object()
            .expect("new target must be object");

        let Some(super_constructor) = super_constructor.as_constructor() else {
            return Err(JsNativeError::typ()
                .with_message("super constructor object must be constructor")
                .into());
        };

        let result = super_constructor.__construct__(&arguments, new_target, context)?;

        let this_env = context
            .as_raw_context()
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");

        this_env.bind_this_value(result.clone())?;
        let function_object = this_env.slots().function_object().clone();

        result.initialize_instance_elements(&function_object, context)?;

        context.as_raw_context_mut().vm.push(result);
        Ok(CompletionType::Normal)
    }
}

/// `SuperCallDerived` implements the Opcode Operation for `Opcode::SuperCallDerived`
///
/// Operation:
///  - Execute the `super()` method when no constructor of the class is defined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCallDerived;

impl Operation for SuperCallDerived {
    const NAME: &'static str = "SuperCallDerived";
    const INSTRUCTION: &'static str = "INST - SuperCallDerived";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let argument_count = raw_context.vm.frame().argument_count;
        let mut arguments = Vec::with_capacity(argument_count as usize);
        for _ in 0..argument_count {
            arguments.push(raw_context.vm.pop());
        }
        arguments.reverse();

        let this_env = raw_context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");
        let new_target = this_env
            .slots()
            .new_target()
            .expect("must have new target")
            .clone();
        let active_function = this_env.slots().function_object().clone();
        let super_constructor = active_function
            .__get_prototype_of__(context)
            .expect("function object must have prototype")
            .expect("function object must have prototype");

        if !super_constructor.is_constructor() {
            return Err(JsNativeError::typ()
                .with_message("super constructor object must be constructor")
                .into());
        }

        let result = super_constructor.__construct__(&arguments, &new_target, context)?;

        let this_env = context
            .as_raw_context()
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");

        this_env.bind_this_value(result.clone())?;

        result.initialize_instance_elements(&active_function, context)?;

        context.as_raw_context_mut().vm.push(result);
        Ok(CompletionType::Normal)
    }
}
