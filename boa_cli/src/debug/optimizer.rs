use boa_engine::{
    object::{FunctionObjectBuilder, ObjectInitializer},
    optimizer::OptimizerOptions,
    property::Attribute,
    Context, JsArgs, JsObject, JsResult, JsValue, NativeFunction,
};

fn get_constant_folding(
    _: &JsValue,
    _: &[JsValue],
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    Ok(context
        .optimizer_options()
        .contains(OptimizerOptions::CONSTANT_FOLDING)
        .into())
}

fn set_constant_folding(
    _: &JsValue,
    args: &[JsValue],
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_boolean();
    let mut options = context.optimizer_options();
    options.set(OptimizerOptions::CONSTANT_FOLDING, value);
    context.set_optimizer_options(options);
    Ok(JsValue::undefined())
}

fn get_statistics(_: &JsValue, _: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    Ok(context
        .optimizer_options()
        .contains(OptimizerOptions::STATISTICS)
        .into())
}

fn set_statistics(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_boolean();
    let mut options = context.optimizer_options();
    options.set(OptimizerOptions::STATISTICS, value);
    context.set_optimizer_options(options);
    Ok(JsValue::undefined())
}

pub(super) fn create_object(context: &mut Context<'_>) -> JsObject {
    let get_constant_folding = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(get_constant_folding),
    )
    .name("get constantFolding")
    .length(0)
    .build();
    let set_constant_folding = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(set_constant_folding),
    )
    .name("set constantFolding")
    .length(1)
    .build();

    let get_statistics =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(get_statistics))
            .name("get statistics")
            .length(0)
            .build();
    let set_statistics =
        FunctionObjectBuilder::new(context.realm(), NativeFunction::from_fn_ptr(set_statistics))
            .name("set statistics")
            .length(1)
            .build();
    ObjectInitializer::new(context)
        .accessor(
            "constantFolding",
            Some(get_constant_folding),
            Some(set_constant_folding),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .accessor(
            "statistics",
            Some(get_statistics),
            Some(set_statistics),
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .build()
}
