use boa_engine::{
    Context, JsNativeError, JsObject, JsResult, JsValue, NativeFunction, js_string,
    object::{IndexProperties, ObjectInitializer},
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

/// Returns objects pointer in memory.
fn indexed_storage_type(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
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

    let typ = match object.borrow().properties().index_properties() {
        IndexProperties::DenseI32(_) => "DenseI32",
        IndexProperties::DenseF64(_) => "DenseF64",
        IndexProperties::DenseElement(_) => "DenseElement",
        IndexProperties::SparseElement(_) => "SparseElement",
        IndexProperties::SparseProperty(_) => "SparseProperty",
    };
    Ok(js_string!(typ).into())
}

pub(super) fn create_object(context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(id), js_string!("id"), 1)
        .function(
            NativeFunction::from_fn_ptr(indexed_storage_type),
            js_string!("indexedStorageType"),
            1,
        )
        .build()
}
