use boa_engine::{
    Context, JsNativeError, JsObject, JsResult, JsValue, NativeFunction, js_string,
    object::{IndexProperties, ObjectInitializer},
};

/// Returns the memory address of an object as a hex-formatted string.
///
/// Useful for identity debugging — determining whether two variables reference
/// the same underlying object, or inspecting object identity across operations.
///
/// # Errors
///
/// Returns a `TypeError` if the argument is not an object.
///
/// # Examples
///
/// ```ignore
/// let o = { x: 10, y: 20 };
/// $boa.object.id(o);    // '0x7F5B3251B718'
/// $boa.object.id($boa); // '0x7F5B3251B5D8'
/// ```
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

/// Returns the internal indexed storage type of an object.
///
/// The returned string indicates how the engine stores numerically-indexed
/// properties internally, transitioning to more flexible (but slower)
/// representations as the access pattern changes:
///
/// | Storage Type | Description |
/// |---|---|
/// | `DenseI32` | All integer elements, compact array storage |
/// | `DenseF64` | All floating-point elements |
/// | `DenseElement` | Mixed-type elements, dense array |
/// | `SparseElement` | Holey array with gaps (`undefined` holes) |
/// | `SparseProperty` | Non-default property descriptors present |
///
/// # Errors
///
/// Returns a `TypeError` if the argument is not an object.
///
/// # Examples
///
/// ```ignore
/// let a = [1, 2];
/// $boa.object.indexedStorageType(a);             // 'DenseI32'
/// a.push(0.5);
/// $boa.object.indexedStorageType(a);             // 'DenseF64'
/// a.push("Hello");
/// $boa.object.indexedStorageType(a);             // 'DenseElement'
/// a[100] = 100;
/// $boa.object.indexedStorageType(a);             // 'SparseElement'
/// ```
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
