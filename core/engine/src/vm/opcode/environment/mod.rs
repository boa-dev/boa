use super::VaryingOperand;
use crate::{
    Context, JsResult, JsValue,
    builtins::function::OrdinaryFunction,
    error::JsNativeError,
    object::internal_methods::InternalMethodPropertyContext,
    vm::{CallFrameFlags, opcode::Operation},
};

/// `This` implements the Opcode Operation for `Opcode::This`
///
/// Operation:
///  - Pushes `this` value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct This;

impl This {
    #[inline(always)]
    pub(super) fn operation(dst: VaryingOperand, context: &mut Context) -> JsResult<()> {
        if context.vm.frame().has_this_value_cached() {
            let this = context.vm.stack.get_this(context.vm.frame());
            context.vm.set_register(dst.into(), this);
            return Ok(());
        }

        let this = context
            .vm
            .environments
            .get_this_binding()?
            .unwrap_or(context.realm().global_this().clone().into());
        context.vm.frame_mut().flags |= CallFrameFlags::THIS_VALUE_CACHED;
        context.vm.stack.set_this(&context.vm.frame, this.clone());
        context.vm.set_register(dst.into(), this);
        Ok(())
    }
}

impl Operation for This {
    const NAME: &'static str = "This";
    const INSTRUCTION: &'static str = "INST - This";
    const COST: u8 = 1;
}

/// `ThisForObjectEnvironmentName` implements the Opcode Operation for `Opcode::ThisForObjectEnvironmentName`
///
/// Operation:
///  - Pushes `this` value that is related to the object environment of the given binding.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThisForObjectEnvironmentName;

impl ThisForObjectEnvironmentName {
    #[inline(always)]
    pub(super) fn operation(
        (dst, index): (VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let binding_locator = context.vm.frame().code_block.bindings[usize::from(index)].clone();
        let this = context
            .this_from_object_environment_binding(&binding_locator)?
            .map_or(JsValue::undefined(), Into::into);
        context.vm.set_register(dst.into(), this);
        Ok(())
    }
}

impl Operation for ThisForObjectEnvironmentName {
    const NAME: &'static str = "ThisForObjectEnvironmentName";
    const INSTRUCTION: &'static str = "INST - ThisForObjectEnvironmentName";
    const COST: u8 = 1;
}

/// `Super` implements the Opcode Operation for `Opcode::Super`
///
/// Operation:
///  - Pushes the current `super` value to the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Super;

impl Super {
    #[inline(always)]
    pub(super) fn operation(dst: VaryingOperand, context: &mut Context) -> JsResult<()> {
        let home_object = {
            let env = context
                .vm
                .environments
                .get_this_environment()
                .as_function()
                .expect("super access must be in a function environment");
            let this = env
                .get_this_binding()?
                .expect("`get_this_environment` ensures this returns `Some`");

            env.slots()
                .function_object()
                .downcast_ref::<OrdinaryFunction>()
                .expect("must be function object")
                .get_home_object()
                .cloned()
                .or(this.as_object())
        };

        let value = home_object
            .map(|o| o.__get_prototype_of__(&mut InternalMethodPropertyContext::new(context)))
            .transpose()?
            .flatten()
            .map_or_else(JsValue::null, JsValue::from);

        context.vm.set_register(dst.into(), value);
        Ok(())
    }
}

impl Operation for Super {
    const NAME: &'static str = "Super";
    const INSTRUCTION: &'static str = "INST - Super";
    const COST: u8 = 3;
}

/// `SuperCallPrepare` implements the Opcode Operation for `Opcode::SuperCallPrepare`
///
/// Operation:
///  - Get the super constructor and the new target of the current environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCallPrepare;

impl SuperCallPrepare {
    #[inline(always)]
    pub(super) fn operation(dst: VaryingOperand, context: &mut Context) {
        let this_env = context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");
        let active_function = this_env.slots().function_object().clone();
        let super_constructor = active_function
            .__get_prototype_of__(&mut InternalMethodPropertyContext::new(context))
            .expect("function object must have prototype");
        context.vm.set_register(
            dst.into(),
            super_constructor.map_or_else(JsValue::null, JsValue::from),
        );
    }
}

impl Operation for SuperCallPrepare {
    const NAME: &'static str = "SuperCallPrepare";
    const INSTRUCTION: &'static str = "INST - SuperCallPrepare";
    const COST: u8 = 3;
}

/// `SuperCall` implements the Opcode Operation for `Opcode::SuperCall`
///
/// Operation:
///  - Execute the `super()` method.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCall;

impl SuperCall {
    #[inline(always)]
    pub(super) fn operation(argument_count: VaryingOperand, context: &mut Context) -> JsResult<()> {
        let super_constructor = context
            .vm
            .stack
            .calling_convention_get_function(argument_count.into())
            .clone();

        let Some(super_constructor) = super_constructor.as_constructor() else {
            return Err(JsNativeError::typ()
                .with_message("super constructor object must be constructor")
                .into());
        };

        let this_env = context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");

        let new_target = this_env
            .slots()
            .new_target()
            .expect("must have new.target")
            .clone();

        context.vm.stack.push(new_target);

        super_constructor
            .__construct__(argument_count.into())
            .resolve(context)?;
        Ok(())
    }
}

impl Operation for SuperCall {
    const NAME: &'static str = "SuperCall";
    const INSTRUCTION: &'static str = "INST - SuperCall";
    const COST: u8 = 3;
}

/// `SuperCallSpread` implements the Opcode Operation for `Opcode::SuperCallSpread`
///
/// Operation:
///  - Execute the `super()` method where the arguments contain spreads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCallSpread;

impl SuperCallSpread {
    #[inline(always)]
    pub(super) fn operation((): (), context: &mut Context) -> JsResult<()> {
        // Get the arguments that are stored as an array object on the stack.
        let arguments_array = context.vm.stack.pop();
        let arguments_array_object = arguments_array
            .as_object()
            .expect("arguments array in call spread function must be an object");
        let arguments = arguments_array_object
            .borrow()
            .properties()
            .to_dense_indexed_properties()
            .expect("arguments array in call spread function must be dense");

        let super_constructor = context.vm.stack.pop();

        let Some(super_constructor) = super_constructor.as_constructor() else {
            return Err(JsNativeError::typ()
                .with_message("super constructor object must be constructor")
                .into());
        };

        context.vm.stack.push(super_constructor.clone());

        context
            .vm
            .stack
            .calling_convention_push_arguments(&arguments);

        let this_env = context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");

        let new_target = this_env
            .slots()
            .new_target()
            .expect("must have new.target")
            .clone();

        context.vm.stack.push(new_target);

        super_constructor
            .__construct__(arguments.len())
            .resolve(context)?;
        Ok(())
    }
}

impl Operation for SuperCallSpread {
    const NAME: &'static str = "SuperCallSpread";
    const INSTRUCTION: &'static str = "INST - SuperCallSpread";
    const COST: u8 = 3;
}

/// `SuperCallDerived` implements the Opcode Operation for `Opcode::SuperCallDerived`
///
/// Operation:
///  - Execute the `super()` method when no constructor of the class is defined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SuperCallDerived;

impl SuperCallDerived {
    #[inline(always)]
    pub(super) fn operation((): (), context: &mut Context) -> JsResult<()> {
        let this_env = context
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
            .__get_prototype_of__(&mut InternalMethodPropertyContext::new(context))
            .expect("function object must have prototype")
            .expect("function object must have prototype");

        if !super_constructor.is_constructor() {
            return Err(JsNativeError::typ()
                .with_message("super constructor object must be constructor")
                .into());
        }

        context.vm.stack.push(JsValue::undefined());
        context.vm.stack.push(super_constructor.clone());
        for argument in Vec::from(context.vm.stack.get_arguments(context.vm.frame())) {
            context.vm.stack.push(argument.clone());
        }
        context.vm.stack.push(new_target);

        super_constructor
            .__construct__(context.vm.frame().argument_count as usize)
            .resolve(context)?;
        Ok(())
    }
}

impl Operation for SuperCallDerived {
    const NAME: &'static str = "SuperCallDerived";
    const INSTRUCTION: &'static str = "INST - SuperCallDerived";
    const COST: u8 = 3;
}

/// `BindThisValue` implements the Opcode Operation for `Opcode::BindThisValue`
///
/// Operation:
///  - Binds `this` value and initializes the instance elements.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BindThisValue;

impl BindThisValue {
    #[inline(always)]
    pub(super) fn operation(value: VaryingOperand, context: &mut Context) -> JsResult<()> {
        // Taken from `SuperCall : super Arguments` steps 7-12.
        //
        // <https://tc39.es/ecma262/#sec-super-keyword-runtime-semantics-evaluation>

        let result = context
            .vm
            .get_register(value.into())
            .as_object()
            .expect("construct result should be an object")
            .clone();

        // 7. Let thisER be GetThisEnvironment().
        let this_env = context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .expect("super call must be in function environment");

        // 8. Perform ? thisER.BindThisValue(result).
        this_env.bind_this_value(result.clone())?;

        // 9. Let F be thisER.[[FunctionObject]].
        // SKIP: 10. Assert: F is an ECMAScript function object.
        let active_function = this_env.slots().function_object().clone();

        // 11. Perform ? InitializeInstanceElements(result, F).
        result.initialize_instance_elements(&active_function, context)?;

        // 12. Return result.
        context.vm.set_register(value.into(), result.into());
        Ok(())
    }
}

impl Operation for BindThisValue {
    const NAME: &'static str = "BindThisValue";
    const INSTRUCTION: &'static str = "INST - BindThisValue";
    const COST: u8 = 6;
}
