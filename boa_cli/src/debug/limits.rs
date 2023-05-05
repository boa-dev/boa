use boa_engine::{
    object::{FunctionObjectBuilder, ObjectInitializer},
    property::Attribute,
    Context, JsArgs, JsObject, JsResult, JsValue, NativeFunction,
};

fn get_loop(_: &JsValue, _: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    let max = context.runtime_limits().loop_iteration_limit();
    Ok(JsValue::from(max))
}

fn set_loop(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_length(context)?;
    context.runtime_limits_mut().set_loop_iteration_limit(value);
    Ok(JsValue::undefined())
}

pub(super) fn create_object(context: &mut Context<'_>) -> JsObject {
    let get_loop = FunctionObjectBuilder::new(context, NativeFunction::from_fn_ptr(get_loop))
        .name("get loop")
        .length(0)
        .build();
    let set_loop = FunctionObjectBuilder::new(context, NativeFunction::from_fn_ptr(set_loop))
        .name("set loop")
        .length(1)
        .build();
    ObjectInitializer::new(context)
        .accessor(
            "loop",
            Some(get_loop),
            Some(set_loop),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .build()
}
