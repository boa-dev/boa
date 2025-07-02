use boa_engine::{
    Context, JsObject, JsResult, JsValue, NativeFunction, js_string, object::ObjectInitializer,
};

/// Creates a new ECMAScript Realm and returns the global object of the realm.
fn create(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let context = &mut Context::default();

    Ok(context.global_object().into())
}

pub(super) fn create_object(context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(create), js_string!("create"), 0)
        .build()
}
