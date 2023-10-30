use crate::{
    builtins::function::{arguments::Arguments, ThisMode},
    context::intrinsics::StandardConstructors,
    environments::{FunctionSlots, ThisBindingStatus},
    native_function::NativeFunctionObject,
    object::{
        internal_methods::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS},
        JsObject, ObjectData,
    },
    vm::{CallFrame, CallFrameFlags},
    Context, JsNativeError, JsResult, JsValue,
};

use super::{get_prototype_from_constructor, CallValue};

/// Definitions of the internal object methods for function objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects
pub(crate) static FUNCTION_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __call__: function_call,
    ..ORDINARY_INTERNAL_METHODS
};

pub(crate) static CONSTRUCTOR_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __call__: function_call,
    __construct__: function_construct,
    ..ORDINARY_INTERNAL_METHODS
};

/// Call this object.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-prepareforordinarycall>
// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-call-thisargument-argumentslist>
pub(crate) fn function_call(
    function_object: &JsObject,
    argument_count: usize,
    context: &mut Context<'_>,
) -> JsResult<CallValue> {
    context.check_runtime_limits()?;

    let object = function_object.borrow();
    let function = object.as_function().expect("not a function");
    let realm = function.realm().clone();

    if function.code.is_class_constructor() {
        debug_assert!(
            function.is_ordinary(),
            "only ordinary functions can be classes"
        );
        return Err(JsNativeError::typ()
            .with_message("class constructor cannot be invoked without 'new'")
            .with_realm(realm)
            .into());
    }

    let code = function.code.clone();
    let environments = function.environments.clone();
    let script_or_module = function.script_or_module.clone();

    drop(object);

    let env_fp = environments.len() as u32;

    let frame = CallFrame::new(code.clone(), script_or_module, environments, realm)
        .with_argument_count(argument_count as u32)
        .with_env_fp(env_fp);

    context.vm.push_frame(frame);

    let fp = context.vm.stack.len() - argument_count - CallFrame::FUNCTION_PROLOGUE;
    context.vm.frame_mut().fp = fp as u32;

    let this = context.vm.stack[fp + CallFrame::THIS_POSITION].clone();

    let lexical_this_mode = code.this_mode == ThisMode::Lexical;

    let this = if lexical_this_mode {
        ThisBindingStatus::Lexical
    } else if code.strict() {
        ThisBindingStatus::Initialized(this.clone())
    } else if this.is_null_or_undefined() {
        ThisBindingStatus::Initialized(context.realm().global_this().clone().into())
    } else {
        ThisBindingStatus::Initialized(
            this.to_object(context)
                .expect("conversion cannot fail")
                .into(),
        )
    };

    let mut last_env = 0;

    if code.has_binding_identifier() {
        let index = context
            .vm
            .environments
            .push_lexical(code.constant_compile_time_environment(last_env));
        context
            .vm
            .environments
            .put_lexical_value(index, 0, function_object.clone().into());
        last_env += 1;
    }

    context.vm.environments.push_function(
        code.constant_compile_time_environment(last_env),
        FunctionSlots::new(this, function_object.clone(), None),
    );

    if code.has_parameters_env_bindings() {
        last_env += 1;
        context
            .vm
            .environments
            .push_lexical(code.constant_compile_time_environment(last_env));
    }

    // Taken from: `FunctionDeclarationInstantiation` abstract function.
    //
    // Spec: https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
    //
    // 22. If argumentsObjectNeeded is true, then
    if code.needs_arguments_object() {
        // a. If strict is true or simpleParameterList is false, then
        //     i. Let ao be CreateUnmappedArgumentsObject(argumentsList).
        // b. Else,
        //     i. NOTE: A mapped argument object is only provided for non-strict functions
        //              that don't have a rest parameter, any parameter
        //              default value initializers, or any destructured parameters.
        //     ii. Let ao be CreateMappedArgumentsObject(func, formals, argumentsList, env).
        let args = context.vm.stack[(fp + CallFrame::FIRST_ARGUMENT_POSITION)..].to_vec();
        let arguments_obj = if code.strict() || !code.params.is_simple() {
            Arguments::create_unmapped_arguments_object(&args, context)
        } else {
            let env = context.vm.environments.current();
            Arguments::create_mapped_arguments_object(
                function_object,
                &code.params,
                &args,
                env.declarative_expect(),
                context,
            )
        };
        let env_index = context.vm.environments.len() as u32 - 1;
        context
            .vm
            .environments
            .put_lexical_value(env_index, 0, arguments_obj.into());
    }

    Ok(CallValue::Ready)
}

/// Construct an instance of this object with the specified arguments.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget>
fn function_construct(
    this_function_object: &JsObject,
    argument_count: usize,
    context: &mut Context<'_>,
) -> JsResult<CallValue> {
    context.check_runtime_limits()?;

    let object = this_function_object.borrow();
    let function = object.as_function().expect("not a function");
    let realm = function.realm().clone();

    debug_assert!(
        function.is_ordinary(),
        "only ordinary functions can be constructed"
    );

    let code = function.code.clone();
    let environments = function.environments.clone();
    let script_or_module = function.script_or_module.clone();
    drop(object);

    let env_fp = environments.len() as u32;

    let new_target = context.vm.pop();

    let at = context.vm.stack.len() - argument_count;

    let this = if code.is_derived_constructor() {
        None
    } else {
        // If the prototype of the constructor is not an object, then use the default object
        // prototype as prototype for the new object
        // see <https://tc39.es/ecma262/#sec-ordinarycreatefromconstructor>
        // see <https://tc39.es/ecma262/#sec-getprototypefromconstructor>
        let prototype =
            get_prototype_from_constructor(&new_target, StandardConstructors::object, context)?;
        let this = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ObjectData::ordinary(),
        );

        this.initialize_instance_elements(this_function_object, context)?;

        Some(this)
    };

    let frame = CallFrame::new(code.clone(), script_or_module, environments, realm)
        .with_argument_count(argument_count as u32)
        .with_env_fp(env_fp)
        .with_flags(CallFrameFlags::CONSTRUCT);

    context.vm.push_frame(frame);

    context.vm.frame_mut().fp = at as u32 - 1;

    let mut last_env = 0;

    if code.has_binding_identifier() {
        let index = context
            .vm
            .environments
            .push_lexical(code.constant_compile_time_environment(last_env));
        context
            .vm
            .environments
            .put_lexical_value(index, 0, this_function_object.clone().into());
        last_env += 1;
    }

    context.vm.environments.push_function(
        code.constant_compile_time_environment(last_env),
        FunctionSlots::new(
            this.clone().map_or(ThisBindingStatus::Uninitialized, |o| {
                ThisBindingStatus::Initialized(o.into())
            }),
            this_function_object.clone(),
            Some(
                new_target
                    .as_object()
                    .expect("new.target should be an object")
                    .clone(),
            ),
        ),
    );

    if code.has_parameters_env_bindings() {
        last_env += 1;
        context
            .vm
            .environments
            .push_lexical(code.constant_compile_time_environment(last_env));
    }

    // Taken from: `FunctionDeclarationInstantiation` abstract function.
    //
    // Spec: https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
    //
    // 22. If argumentsObjectNeeded is true, then
    if code.needs_arguments_object() {
        // a. If strict is true or simpleParameterList is false, then
        //     i. Let ao be CreateUnmappedArgumentsObject(argumentsList).
        // b. Else,
        //     i. NOTE: A mapped argument object is only provided for non-strict functions
        //              that don't have a rest parameter, any parameter
        //              default value initializers, or any destructured parameters.
        //     ii. Let ao be CreateMappedArgumentsObject(func, formals, argumentsList, env).
        let args = context.vm.stack[at..].to_vec();
        let arguments_obj = if code.strict() || !code.params.is_simple() {
            Arguments::create_unmapped_arguments_object(&args, context)
        } else {
            let env = context.vm.environments.current();
            Arguments::create_mapped_arguments_object(
                this_function_object,
                &code.params,
                &args,
                env.declarative_expect(),
                context,
            )
        };
        let env_index = context.vm.environments.len() as u32 - 1;
        context
            .vm
            .environments
            .put_lexical_value(env_index, 0, arguments_obj.into());
    }

    // Insert `this` value
    context
        .vm
        .stack
        .insert(at - 1, this.map(JsValue::new).unwrap_or_default());

    Ok(CallValue::Ready)
}

/// Definitions of the internal object methods for native function objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects
pub(crate) static NATIVE_FUNCTION_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __call__: native_function_call,
    ..ORDINARY_INTERNAL_METHODS
};

pub(crate) static NATIVE_CONSTRUCTOR_INTERNAL_METHODS: InternalObjectMethods =
    InternalObjectMethods {
        __call__: native_function_call,
        __construct__: native_function_construct,
        ..ORDINARY_INTERNAL_METHODS
    };

/// Call this object.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-built-in-function-objects-call-thisargument-argumentslist>
pub(crate) fn native_function_call(
    obj: &JsObject,
    argument_count: usize,
    context: &mut Context<'_>,
) -> JsResult<CallValue> {
    let args = context.vm.pop_n_values(argument_count);
    let _func = context.vm.pop();
    let this = context.vm.pop();

    // We technically don't need this since native functions don't push any new frames to the
    // vm, but we'll eventually have to combine the native stack with the vm stack.
    context.check_runtime_limits()?;
    let this_function_object = obj.clone();

    let NativeFunctionObject {
        f: function,
        constructor,
        realm,
    } = obj
        .borrow()
        .as_native_function()
        .cloned()
        .expect("the object should be a native function object");

    let mut realm = realm.unwrap_or_else(|| context.realm().clone());

    context.swap_realm(&mut realm);
    context.vm.native_active_function = Some(this_function_object);

    let result = if constructor.is_some() {
        function.call(&JsValue::undefined(), &args, context)
    } else {
        function.call(&this, &args, context)
    }
    .map_err(|err| err.inject_realm(context.realm().clone()));

    context.vm.native_active_function = None;
    context.swap_realm(&mut realm);

    context.vm.push(result?);

    Ok(CallValue::Complete)
}

/// Construct an instance of this object with the specified arguments.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-built-in-function-objects-construct-argumentslist-newtarget>
fn native_function_construct(
    obj: &JsObject,
    argument_count: usize,
    context: &mut Context<'_>,
) -> JsResult<CallValue> {
    // We technically don't need this since native functions don't push any new frames to the
    // vm, but we'll eventually have to combine the native stack with the vm stack.
    context.check_runtime_limits()?;
    let this_function_object = obj.clone();

    let NativeFunctionObject {
        f: function,
        constructor,
        realm,
    } = obj
        .borrow()
        .as_native_function()
        .cloned()
        .expect("the object should be a native function object");

    let mut realm = realm.unwrap_or_else(|| context.realm().clone());

    context.swap_realm(&mut realm);
    context.vm.native_active_function = Some(this_function_object);

    let new_target = context.vm.pop();
    let args = context.vm.pop_n_values(argument_count);
    let _func = context.vm.pop();

    let result = function
        .call(&new_target, &args, context)
        .map_err(|err| err.inject_realm(context.realm().clone()))
        .and_then(|v| match v {
            JsValue::Object(ref o) => Ok(o.clone()),
            val => {
                if constructor.expect("must be a constructor").is_base() || val.is_undefined() {
                    let prototype = get_prototype_from_constructor(
                        &new_target,
                        StandardConstructors::object,
                        context,
                    )?;
                    Ok(JsObject::from_proto_and_data_with_shared_shape(
                        context.root_shape(),
                        prototype,
                        ObjectData::ordinary(),
                    ))
                } else {
                    Err(JsNativeError::typ()
                        .with_message("derived constructor can only return an Object or undefined")
                        .into())
                }
            }
        });

    context.vm.native_active_function = None;
    context.swap_realm(&mut realm);

    context.vm.push(result?);

    Ok(CallValue::Complete)
}
