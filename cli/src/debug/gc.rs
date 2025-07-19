use boa_engine::{
    Context, JsObject, JsResult, JsValue, NativeFunction, js_string, object::ObjectInitializer,
};

/// Trigger garbage collection.
fn collect(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    boa_gc::force_collect();
    Ok(JsValue::undefined())
}

pub(super) fn create_object(context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(collect),
            js_string!("collect"),
            0,
        )
        .build()
}
