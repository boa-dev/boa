use crate::{
    object::{
        internal_methods::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS},
        JsObject,
    },
    Context, JsResult, JsValue,
};

/// Definitions of the internal object methods for function objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects
pub(crate) static FUNCTION_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __call__: Some(function_call),
    __construct__: None,
    ..ORDINARY_INTERNAL_METHODS
};

pub(crate) static CONSTRUCTOR_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __call__: Some(function_call),
    __construct__: Some(function_construct),
    ..ORDINARY_INTERNAL_METHODS
};

/// Call this object.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-prepareforordinarycall>
// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-call-thisargument-argumentslist>
#[track_caller]
#[inline]
fn function_call(
    obj: &JsObject,
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    obj.call_internal(this, args, context)
}

/// Construct an instance of this object with the specified arguments.
///
/// # Panics
///
/// Panics if the object is currently mutably borrowed.
// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget>
#[track_caller]
#[inline]
fn function_construct(
    obj: &JsObject,
    args: &[JsValue],
    new_target: &JsObject,
    context: &mut Context,
) -> JsResult<JsObject> {
    obj.construct_internal(args, &new_target.clone().into(), context)
}
