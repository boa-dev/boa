use crate::{
    environments::EnvironmentSlots,
    error::JsNativeError,
    vm::{ok_or_throw_completion, opcode::Operation, throw_completion, CompletionType},
    Context, JsError, JsValue,
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let env = context.realm.environments.get_this_environment();
        match env {
            EnvironmentSlots::Function(env) => {
                let binding_result = match env.borrow().get_this_binding() {
                    Ok(binding) => Ok(binding.clone()),
                    Err(e) => Err(e),
                };
                let function_binding = ok_or_throw_completion!(binding_result, context);
                context.vm.push(function_binding);
            }
            EnvironmentSlots::Global => {
                let this = context.realm.global_this();
                context.vm.push(this.clone());
            }
        }
        CompletionType::Normal
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let home_result = {
            let env = context
                .realm
                .environments
                .get_this_environment()
                .as_function_slots()
                .expect("super access must be in a function environment");
            let env = env.borrow();
            match env.get_this_binding() {
                Ok(binding) => {
                    let function_object = env.function_object().borrow();
                    let function = function_object
                        .as_function()
                        .expect("must be function object");

                    Ok(function.get_home_object().or(binding.as_object()).cloned())
                }
                Err(e) => Err(e),
            }
        };

        let home = ok_or_throw_completion!(home_result, context);

        if let Some(home) = home {
            if let Some(proto) =
                ok_or_throw_completion!(home.__get_prototype_of__(context), context)
            {
                context.vm.push(JsValue::from(proto));
            } else {
                context.vm.push(JsValue::Null);
            }
        } else {
            context.vm.push(JsValue::Null);
        };
        CompletionType::Normal
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let argument_count = context.vm.read::<u32>();
        let mut arguments = Vec::with_capacity(argument_count as usize);
        for _ in 0..argument_count {
            arguments.push(context.vm.pop());
        }
        arguments.reverse();

        let (new_target, active_function) = {
            let this_env = context
                .realm
                .environments
                .get_this_environment()
                .as_function_slots()
                .expect("super call must be in function environment");
            let this_env_borrow = this_env.borrow();
            let new_target = this_env_borrow
                .new_target()
                .expect("must have new target")
                .clone();
            let active_function = this_env.borrow().function_object().clone();
            (new_target, active_function)
        };
        let super_constructor = active_function
            .__get_prototype_of__(context)
            .expect("function object must have prototype")
            .expect("function object must have prototype");

        if !super_constructor.is_constructor() {
            throw_completion!(
                JsNativeError::typ()
                    .with_message("super constructor object must be constructor")
                    .into(),
                JsError,
                context
            );
        }

        let result = ok_or_throw_completion!(
            super_constructor.__construct__(&arguments, &new_target, context),
            context
        );

        let this_env = context
            .realm
            .environments
            .get_this_environment()
            .as_function_slots()
            .expect("super call must be in function environment");

        if !this_env.borrow_mut().bind_this_value(&result) {
            throw_completion!(
                JsNativeError::reference()
                    .with_message("this already initialized")
                    .into(),
                JsError,
                context
            );
        }

        ok_or_throw_completion!(
            result.initialize_instance_elements(&active_function, context),
            context
        );

        context.vm.push(result);
        CompletionType::Normal
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
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

        let (new_target, active_function) = {
            let this_env = context
                .realm
                .environments
                .get_this_environment()
                .as_function_slots()
                .expect("super call must be in function environment");
            let this_env_borrow = this_env.borrow();
            let new_target = this_env_borrow
                .new_target()
                .expect("must have new target")
                .clone();
            let active_function = this_env.borrow().function_object().clone();
            (new_target, active_function)
        };
        let super_constructor = active_function
            .__get_prototype_of__(context)
            .expect("function object must have prototype")
            .expect("function object must have prototype");

        if !super_constructor.is_constructor() {
            throw_completion!(
                JsNativeError::typ()
                    .with_message("super constructor object must be constructor")
                    .into(),
                JsError,
                context
            );
        }

        let result = ok_or_throw_completion!(
            super_constructor.__construct__(&arguments, &new_target, context),
            context
        );

        let this_env = context
            .realm
            .environments
            .get_this_environment()
            .as_function_slots()
            .expect("super call must be in function environment");

        if !this_env.borrow_mut().bind_this_value(&result) {
            throw_completion!(
                JsNativeError::reference()
                    .with_message("this already initialized")
                    .into(),
                JsError,
                context
            );
        }

        ok_or_throw_completion!(
            result.initialize_instance_elements(&active_function, context),
            context
        );

        context.vm.push(result);
        CompletionType::Normal
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let argument_count = context.vm.frame().arg_count;
        let mut arguments = Vec::with_capacity(argument_count);
        for _ in 0..argument_count {
            arguments.push(context.vm.pop());
        }
        arguments.reverse();

        let (new_target, active_function) = {
            let this_env = context
                .realm
                .environments
                .get_this_environment()
                .as_function_slots()
                .expect("super call must be in function environment");
            let this_env_borrow = this_env.borrow();
            let new_target = this_env_borrow
                .new_target()
                .expect("must have new target")
                .clone();
            let active_function = this_env.borrow().function_object().clone();
            (new_target, active_function)
        };
        let super_constructor = active_function
            .__get_prototype_of__(context)
            .expect("function object must have prototype")
            .expect("function object must have prototype");

        if !super_constructor.is_constructor() {
            throw_completion!(
                JsNativeError::typ()
                    .with_message("super constructor object must be constructor")
                    .into(),
                JsError,
                context
            );
        }

        let result = ok_or_throw_completion!(
            super_constructor.__construct__(&arguments, &new_target, context),
            context
        );

        let this_env = context
            .realm
            .environments
            .get_this_environment()
            .as_function_slots()
            .expect("super call must be in function environment");

        if !this_env.borrow_mut().bind_this_value(&result) {
            throw_completion!(
                JsNativeError::reference()
                    .with_message("this already initialized")
                    .into(),
                JsError,
                context
            );
        }

        ok_or_throw_completion!(
            result.initialize_instance_elements(&active_function, context),
            context
        );

        context.vm.push(result);
        CompletionType::Normal
    }
}
