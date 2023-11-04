use boa_engine::{
    js_string, object::ObjectInitializer, property::Attribute, string::JsStrVariant, Context,
    JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
};

fn storage(_: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
    let Some(value) = args.get(0) else {
        return Err(JsNativeError::typ()
            .with_message("expected string argument")
            .into());
    };

    let Some(string) = value.as_string() else {
        return Err(JsNativeError::typ()
            .with_message(format!("expected string, got {}", value.type_of()))
            .into());
    };

    let storage = if string.is_static() { "static" } else { "heap" };
    Ok(js_string!(storage).into())
}

fn encoding(_: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
    let Some(value) = args.get(0) else {
        return Err(JsNativeError::typ()
            .with_message("expected string argument")
            .into());
    };

    let Some(string) = value.as_string() else {
        return Err(JsNativeError::typ()
            .with_message(format!("expected string, got {}", value.type_of()))
            .into());
    };

    let str = string.as_str();
    let encoding = match str.variant() {
        JsStrVariant::Ascii(_) => "ascii",
        JsStrVariant::U16(_) => "U16",
    };
    Ok(js_string!(encoding).into())
}

fn summary(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    let Some(value) = args.get(0) else {
        return Err(JsNativeError::typ()
            .with_message("expected string argument")
            .into());
    };

    let Some(string) = value.as_string() else {
        return Err(JsNativeError::typ()
            .with_message(format!("expected string, got {}", value.type_of()))
            .into());
    };

    let storage = if string.is_static() { "static" } else { "heap" };
    let encoding = match string.as_str().variant() {
        JsStrVariant::Ascii(_) => "ascii",
        JsStrVariant::U16(_) => "U16",
    };

    let summary = ObjectInitializer::new(context)
        .property("storage", js_string!(storage), Attribute::all())
        .property("encoding", js_string!(encoding), Attribute::all())
        .build();

    Ok(summary.into())
}

pub(super) fn create_string(context: &mut Context<'_>) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(storage), "storage", 1)
        .function(NativeFunction::from_fn_ptr(encoding), "encoding", 1)
        .function(NativeFunction::from_fn_ptr(summary), "summary", 1)
        .build()
}
