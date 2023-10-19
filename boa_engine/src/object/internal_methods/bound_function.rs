use crate::{object::JsObject, Context, JsResult, JsValue};

use super::{CallValue, InternalObjectMethods, ORDINARY_INTERNAL_METHODS};

/// Definitions of the internal object methods for function objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects
pub(crate) static BOUND_FUNCTION_EXOTIC_INTERNAL_METHODS: InternalObjectMethods =
    InternalObjectMethods {
        __call__: bound_function_exotic_call,
        ..ORDINARY_INTERNAL_METHODS
    };

pub(crate) static BOUND_CONSTRUCTOR_EXOTIC_INTERNAL_METHODS: InternalObjectMethods =
    InternalObjectMethods {
        __call__: bound_function_exotic_call,
        __construct__: bound_function_exotic_construct,
        ..ORDINARY_INTERNAL_METHODS
    };

/// Internal method `[[Call]]` for Bound Function Exotic Objects
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-bound-function-exotic-objects-call-thisargument-argumentslist
#[allow(clippy::unnecessary_wraps)]
fn bound_function_exotic_call(
    obj: &JsObject,
    argument_count: usize,
    context: &mut Context<'_>,
) -> JsResult<CallValue> {
    let obj = obj.borrow();
    let bound_function = obj
        .as_bound_function()
        .expect("bound function exotic method should only be callable from bound function objects");

    let arguments_start_index = context.vm.stack.len() - argument_count;

    // 1. Let target be F.[[BoundTargetFunction]].
    let target = bound_function.target_function();
    context.vm.stack[arguments_start_index - 1] = target.clone().into();

    // 2. Let boundThis be F.[[BoundThis]].
    let bound_this = bound_function.this();
    context.vm.stack[arguments_start_index - 2] = bound_this.clone();

    // 3. Let boundArgs be F.[[BoundArguments]].
    let bound_args = bound_function.args();

    // 4. Let args be the list-concatenation of boundArgs and argumentsList.
    context
        .vm
        .insert_values_at(bound_args, arguments_start_index);

    // 5. Return ? Call(target, boundThis, args).
    Ok(target.__call__(bound_args.len() + argument_count))
}

/// Internal method `[[Construct]]` for Bound Function Exotic Objects
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-bound-function-exotic-objects-construct-argumentslist-newtarget
#[allow(clippy::unnecessary_wraps)]
fn bound_function_exotic_construct(
    function_object: &JsObject,
    argument_count: usize,
    context: &mut Context<'_>,
) -> JsResult<CallValue> {
    let new_target = context.vm.pop();

    debug_assert!(new_target.is_object(), "new.target should be an object");

    let object = function_object.borrow();
    let bound_function = object
        .as_bound_function()
        .expect("bound function exotic method should only be callable from bound function objects");

    // 1. Let target be F.[[BoundTargetFunction]].
    let target = bound_function.target_function();

    // 2. Assert: IsConstructor(target) is true.

    // 3. Let boundArgs be F.[[BoundArguments]].
    let bound_args = bound_function.args();

    // 4. Let args be the list-concatenation of boundArgs and argumentsList.
    let arguments_start_index = context.vm.stack.len() - argument_count;
    context
        .vm
        .insert_values_at(bound_args, arguments_start_index);

    // 5. If SameValue(F, newTarget) is true, set newTarget to target.
    let function_object: JsValue = function_object.clone().into();
    let new_target = if JsValue::same_value(&function_object, &new_target) {
        target.clone().into()
    } else {
        new_target
    };

    // 6. Return ? Construct(target, args, newTarget).
    context.vm.push(new_target);
    Ok(target.__construct__(bound_args.len() + argument_count))
}
