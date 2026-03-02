use boa_engine::{
    Context, JsObject, JsResult, JsValue, NativeFunction, js_string, object::ObjectInitializer,
};

/// Creates a new ECMAScript Realm and returns the global object of the realm.
fn create(_: &JsValue, _: &[JsValue], _: &Context) -> JsResult<JsValue> {
    let context = &Context::default();

    Ok(context.global_object().into())
}

pub(super) fn create_object(context: &Context) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(create), js_string!("create"), 0)
        .build()
}
