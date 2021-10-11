use crate::{object::JsObject, Context, JsResult, JsValue};

use super::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS};

/// Definitions of the internal object methods for function objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects
pub(crate) static BOUND_FUNCTION_EXOTIC_INTERNAL_METHODS: InternalObjectMethods =
    InternalObjectMethods {
        __call__: Some(bound_function_exotic_call),
        __construct__: None,
        ..ORDINARY_INTERNAL_METHODS
    };

pub(crate) static BOUND_CONSTRUCTOR_EXOTIC_INTERNAL_METHODS: InternalObjectMethods =
    InternalObjectMethods {
        __call__: Some(bound_function_exotic_call),
        __construct__: Some(bound_function_exotic_construct),
        ..ORDINARY_INTERNAL_METHODS
    };

/// Internal method `[[Call]]` for Bound Function Exotic Objects
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-bound-function-exotic-objects-call-thisargument-argumentslist
#[track_caller]
#[inline]
fn bound_function_exotic_call(
    obj: &JsObject,
    _: &JsValue,
    arguments_list: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = obj.borrow();
    let bound_function = obj
        .as_bound_function()
        .expect("bound function exotic method should only be callable from bound function objects");

    // 1. Let target be F.[[BoundTargetFunction]].
    let target = bound_function.target_function();

    // 2. Let boundThis be F.[[BoundThis]].
    let bound_this = bound_function.this();

    // 3. Let boundArgs be F.[[BoundArguments]].
    let bound_args = bound_function.args();

    // 4. Let args be the list-concatenation of boundArgs and argumentsList.
    let mut args = bound_args.to_vec();
    args.extend_from_slice(arguments_list);

    // 5. Return ? Call(target, boundThis, args).
    target.call(bound_this, &args, context)
}

/// Internal method `[[Construct]]` for Bound Function Exotic Objects
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-bound-function-exotic-objects-construct-argumentslist-newtarget
#[track_caller]
#[inline]
fn bound_function_exotic_construct(
    obj: &JsObject,
    arguments_list: &[JsValue],
    new_target: &JsValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    let object = obj.borrow();
    let bound_function = object
        .as_bound_function()
        .expect("bound function exotic method should only be callable from bound function objects");

    // 1. Let target be F.[[BoundTargetFunction]].
    let target = bound_function.target_function();

    // 2. Assert: IsConstructor(target) is true.

    // 3. Let boundArgs be F.[[BoundArguments]].
    let bound_args = bound_function.args();

    // 4. Let args be the list-concatenation of boundArgs and argumentsList.
    let mut args = bound_args.to_vec();
    args.extend_from_slice(arguments_list);

    // 5. If SameValue(F, newTarget) is true, set newTarget to target.
    let new_target = match new_target {
        JsValue::Object(new_target) if JsObject::equals(obj, new_target) => target.clone().into(),
        _ => new_target.clone(),
    };

    // 6. Return ? Construct(target, args, newTarget).
    target.construct(&args, &new_target, context)
}
