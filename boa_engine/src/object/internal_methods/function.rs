use crate::{
    context::intrinsics::StandardConstructors,
    object::{
        internal_methods::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS},
        JsObject, ObjectData, ObjectKind,
    },
    Context, JsNativeError, JsResult, JsValue,
};

use super::get_prototype_from_constructor;

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
fn function_call(
    obj: &JsObject,
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    obj.call_internal(this, args, context)
}

/// Construct an instance of this object with the specified arguments.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget>
fn function_construct(
    obj: &JsObject,
    args: &[JsValue],
    new_target: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<JsObject> {
    obj.construct_internal(args, &new_target.clone().into(), context)
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
///
// <https://tc39.es/ecma262/#sec-built-in-function-objects-call-thisargument-argumentslist>
#[track_caller]
pub(crate) fn native_function_call(
    obj: &JsObject,
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    let this_function_object = obj.clone();
    let object = obj.borrow();

    let ObjectKind::NativeFunction {
        function,
        constructor,
        realm,
    } = object.kind()
    else {
        unreachable!("the object should be a native function object");
    };

    let mut realm = realm.clone();
    let function = function.clone();
    let constructor = *constructor;
    drop(object);

    context.swap_realm(&mut realm);
    context.vm.native_active_function = Some(this_function_object);

    let result = if constructor.is_some() {
        function.call(&JsValue::undefined(), args, context)
    } else {
        function.call(this, args, context)
    }
    .map_err(|err| err.inject_realm(context.realm().clone()));

    context.vm.native_active_function = None;
    context.swap_realm(&mut realm);

    result
}

/// Construct an instance of this object with the specified arguments.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-built-in-function-objects-construct-argumentslist-newtarget>
#[track_caller]
fn native_function_construct(
    obj: &JsObject,
    args: &[JsValue],
    new_target: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<JsObject> {
    let this_function_object = obj.clone();
    let object = obj.borrow();

    let ObjectKind::NativeFunction {
        function,
        constructor,
        realm,
    } = object.kind()
    else {
        unreachable!("the object should be a native function object");
    };

    let mut realm = realm.clone();
    let function = function.clone();
    let constructor = *constructor;
    drop(object);

    context.swap_realm(&mut realm);
    context.vm.native_active_function = Some(this_function_object);

    let new_target = new_target.clone().into();
    let result = function
        .call(&new_target, args, context)
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

    result
}
