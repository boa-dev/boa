use boa_engine::{
    js_string, object::ObjectInitializer, Context, JsNativeError, JsObject, JsResult, JsValue,
    NativeFunction,
};

/// Returns objects pointer in memory.
fn id(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let Some(value) = args.first() else {
        return Err(JsNativeError::typ()
            .with_message("expected object argument")
            .into());
    };

    let Some(object) = value.as_object() else {
        return Err(JsNativeError::typ()
            .with_message(format!("expected object, got {}", value.type_of()))
            .into());
    };

    let ptr: *const _ = object.as_ref();
    Ok(js_string!(format!("0x{:X}", ptr.cast::<()>() as usize)).into())
}

pub(super) fn create_object(context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(id), js_string!("id"), 1)
        .build()
}
