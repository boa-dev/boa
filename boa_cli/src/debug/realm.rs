use boa_engine::{
    object::ObjectInitializer, Context, DefaultContext, JsObject, JsResult, JsValue, NativeFunction,
};

/// Creates a new ECMAScript Realm and returns the global object of the realm.
fn create(_: &JsValue, _: &[JsValue], _: &mut dyn Context<'_>) -> JsResult<JsValue> {
    let context: &dyn Context<'_> = &mut DefaultContext::default();

    Ok(context.global_object().into())
}

pub(super) fn create_object(context: &mut dyn Context<'_>) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(create), "create", 0)
        .build()
}
