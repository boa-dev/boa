use boa_engine::{
    js_string,
    native_function::NativeFunction,
    object::{JsObject, ObjectInitializer},
    property::Attribute,
    Context, JsArgs, JsNativeError, JsResult, JsValue, Source,
};

/// Creates the object $262 in the context.
pub(super) fn register_js262(context: &mut Context<'_>) -> JsObject {
    let global_obj = context.global_object();

    let js262 = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(create_realm),
            js_string!("createRealm"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(detach_array_buffer),
            js_string!("detachArrayBuffer"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(eval_script),
            js_string!("evalScript"),
            1,
        )
        .function(NativeFunction::from_fn_ptr(gc), js_string!("gc"), 0)
        .property(
            js_string!("global"),
            global_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        // .property("agent", agent, Attribute::default())
        .build();

    context
        .register_global_property(
            js_string!("$262"),
            js262.clone(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .expect("shouldn't fail with the default global");

    js262
}

/// The `$262.createRealm()` function.
///
/// Creates a new ECMAScript Realm, defines this API on the new realm's global object, and
/// returns the `$262` property of the new realm's global object.
#[allow(clippy::unnecessary_wraps)]
fn create_realm(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
    let context = &mut Context::default();

    let js262 = register_js262(context);

    Ok(JsValue::new(js262))
}

/// The `$262.detachArrayBuffer()` function.
///
/// Implements the `DetachArrayBuffer` abstract operation.
fn detach_array_buffer(_: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
    fn type_err() -> JsNativeError {
        JsNativeError::typ().with_message("The provided object was not an ArrayBuffer")
    }

    let array_buffer = args
        .get(0)
        .and_then(JsValue::as_object)
        .ok_or_else(type_err)?;
    let mut array_buffer = array_buffer.borrow_mut();
    let array_buffer = array_buffer.as_array_buffer_mut().ok_or_else(type_err)?;

    // 1. Assert: IsSharedArrayBuffer(arrayBuffer) is false. TODO
    // 2. If key is not present, set key to undefined.
    let key = args.get_or_undefined(1);

    // 3. If SameValue(arrayBuffer.[[ArrayBufferDetachKey]], key) is false, throw a TypeError exception.
    if !JsValue::same_value(&array_buffer.array_buffer_detach_key, key) {
        return Err(JsNativeError::typ()
            .with_message("Cannot detach array buffer with different key")
            .into());
    }

    // 4. Set arrayBuffer.[[ArrayBufferData]] to null.
    array_buffer.array_buffer_data = None;

    // 5. Set arrayBuffer.[[ArrayBufferByteLength]] to 0.
    array_buffer.array_buffer_byte_length = 0;

    // 6. Return NormalCompletion(null).
    Ok(JsValue::null())
}

/// The `$262.evalScript()` function.
///
/// Accepts a string value as its first argument and executes it as an ECMAScript script.
fn eval_script(_this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
    args.get(0).and_then(JsValue::as_string).map_or_else(
        || Ok(JsValue::undefined()),
        |source_text| context.eval(Source::from_bytes(&source_text.to_std_string_escaped())),
    )
}

/// The `$262.gc()` function.
///
/// Wraps the host's garbage collection invocation mechanism, if such a capability exists.
/// Must throw an exception if no capability exists. This is necessary for testing the
/// semantics of any feature that relies on garbage collection, e.g. the `WeakRef` API.
#[allow(clippy::unnecessary_wraps)]
fn gc(_this: &JsValue, _: &[JsValue], _context: &mut Context<'_>) -> JsResult<JsValue> {
    boa_gc::force_collect();
    Ok(JsValue::undefined())
}
