use boa_engine::{
    object::{FunctionObjectBuilder, ObjectInitializer},
    property::Attribute,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};

fn get_loop(_: &JsValue, _: &[JsValue], context: &mut dyn Context<'_>) -> JsResult<JsValue> {
    let max = context.runtime_limits().loop_iteration_limit();
    Ok(JsValue::from(max))
}

fn set_loop(_: &JsValue, args: &[JsValue], context: &mut dyn Context<'_>) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_length(context)?;
    context.runtime_limits_mut().set_loop_iteration_limit(value);
    Ok(JsValue::undefined())
}

fn get_recursion(_: &JsValue, _: &[JsValue], context: &mut dyn Context<'_>) -> JsResult<JsValue> {
    let max = context.runtime_limits().recursion_limit();
    Ok(JsValue::from(max))
}

fn set_recursion(
    _: &JsValue,
    args: &[JsValue],
    context: &mut dyn Context<'_>,
) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_length(context)?;
    let Ok(value) = value.try_into() else {
        return Err(
            JsNativeError::range().with_message(format!("Argument {value} greater than usize::MAX")).into()
        );
    };
    context.runtime_limits_mut().set_recursion_limit(value);
    Ok(JsValue::undefined())
}

pub(super) fn create_object(context: &mut dyn Context<'_>) -> JsObject {
    let get_loop = FunctionObjectBuilder::new(context, NativeFunction::from_fn_ptr(get_loop))
        .name("get loop")
        .length(0)
        .build();
    let set_loop = FunctionObjectBuilder::new(context, NativeFunction::from_fn_ptr(set_loop))
        .name("set loop")
        .length(1)
        .build();

    let get_recursion =
        FunctionObjectBuilder::new(context, NativeFunction::from_fn_ptr(get_recursion))
            .name("get recursion")
            .length(0)
            .build();
    let set_recursion =
        FunctionObjectBuilder::new(context, NativeFunction::from_fn_ptr(set_recursion))
            .name("set recursion")
            .length(1)
            .build();
    ObjectInitializer::new(context)
        .accessor(
            "loop",
            Some(get_loop),
            Some(set_loop),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .accessor(
            "recursion",
            Some(get_recursion),
            Some(set_recursion),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .build()
}
