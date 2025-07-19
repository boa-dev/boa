// This example shows how to access the keys and values of a `JsObject`

use boa_engine::{
    Context, JsError, JsNativeError, JsValue, Source, js_string, property::PropertyKey,
};

fn main() -> Result<(), JsError> {
    // We create a new `Context` to create a new Javascript executor.
    let mut context = Context::default();

    let value = context.eval(Source::from_bytes("({ x: 10, '1': 20 })"))?;
    let object = value
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("Expected object"))?;

    let keys = object.own_property_keys(&mut context)?;

    assert_eq!(
        keys,
        &[PropertyKey::from(1), PropertyKey::from(js_string!("x"))]
    );

    let mut values = Vec::new();
    for key in keys {
        values.push(object.get(key, &mut context)?);
    }

    assert_eq!(values, &[JsValue::from(20), JsValue::from(10)]);

    Ok(())
}
