use boa_engine::{
    Context, JsNativeError, JsObject, JsResult, JsValue, NativeFunction, js_string,
    object::ObjectInitializer, property::Attribute, string::JsStrVariant,
};

fn storage(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let Some(value) = args.first() else {
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

fn encoding(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let Some(value) = args.first() else {
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
        JsStrVariant::Latin1(_) => "latin1",
        JsStrVariant::Utf16(_) => "utf16",
    };
    Ok(js_string!(encoding).into())
}

fn summary(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let Some(value) = args.first() else {
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
        JsStrVariant::Latin1(_) => "latin1",
        JsStrVariant::Utf16(_) => "utf16",
    };

    let summary = ObjectInitializer::new(context)
        .property(js_string!("storage"), js_string!(storage), Attribute::all())
        .property(
            js_string!("encoding"),
            js_string!(encoding),
            Attribute::all(),
        )
        .build();

    Ok(summary.into())
}

pub(super) fn create_string(context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(storage),
            js_string!("storage"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(encoding),
            js_string!("encoding"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(summary),
            js_string!("summary"),
            1,
        )
        .build()
}
