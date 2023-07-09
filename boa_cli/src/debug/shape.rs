use boa_engine::{
    js_string, object::ObjectInitializer, Context, JsArgs, JsNativeError, JsObject, JsResult,
    JsValue, NativeFunction,
};

fn get_object(args: &[JsValue], position: usize) -> JsResult<&JsObject> {
    let value = args.get_or_undefined(position);

    let Some(object) = value.as_object() else {
        return Err(JsNativeError::typ()
            .with_message(format!(
                "expected object in argument position {position}, got {}",
                value.type_of()
            ))
            .into());
    };

    Ok(object)
}

/// Returns object's shape pointer in memory.
fn id(_: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
    let object = get_object(args, 0)?;
    let object = object.borrow();
    let shape = object.shape();
    Ok(format!("0x{:X}", shape.to_addr_usize()).into())
}

/// Returns object's shape type.
fn r#type(_: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
    let object = get_object(args, 0)?;
    let object = object.borrow();
    let shape = object.shape();

    Ok(if shape.is_shared() {
        js_string!("shared")
    } else {
        js_string!("unique")
    }
    .into())
}

/// Returns object's shape type.
fn same(_: &JsValue, args: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
    let lhs = get_object(args, 0)?;
    let rhs = get_object(args, 1)?;

    let lhs_shape_ptr = {
        let object = lhs.borrow();
        let shape = object.shape();
        shape.to_addr_usize()
    };

    let rhs_shape_ptr = {
        let object = rhs.borrow();
        let shape = object.shape();
        shape.to_addr_usize()
    };

    Ok(JsValue::new(lhs_shape_ptr == rhs_shape_ptr))
}

pub(super) fn create_object(context: &mut Context<'_>) -> JsObject {
    ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(id), "id", 1)
        .function(NativeFunction::from_fn_ptr(r#type), "type", 1)
        .function(NativeFunction::from_fn_ptr(same), "same", 2)
        .build()
}
