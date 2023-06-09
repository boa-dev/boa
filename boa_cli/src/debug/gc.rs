use boa_engine::{object::ObjectInitializer, Context, JsObject, JsResult, JsValue, NativeFunction};

/// Trigger garbage collection.
fn collect(_: &JsValue, _: &[JsValue], _: &mut dyn Context<'_>) -> JsResult<JsValue> {
    boa_gc::force_collect();
    Ok(JsValue::undefined())
}

pub(super) fn create_object(context: &mut dyn Context<'_>) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(collect), "collect", 0)
        .build()
}
