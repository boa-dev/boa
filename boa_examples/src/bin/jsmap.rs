use boa_engine::{
    object::{JsArray, JsMap},
    Context, JsResult, JsValue,
};

fn main() -> JsResult<()> {
    let context = &mut Context::default();

    let map = JsMap::new(context);

    map.set(JsValue::new("Key-1"), JsValue::new("Value-1"), context)?;

    let map_check = map.has(JsValue::new("Key-1"), context)?;
    assert_eq!(map_check, JsValue::new(true));

    map.set(JsValue::new("Key-2"), JsValue::new("Value-2"), context)?;

    assert_eq!(map.get_size(context)?, JsValue::new(2));

    let current_key_one = map.get(JsValue::new("Key-1"), context)?;
    assert_eq!(current_key_one, JsValue::new("Value-1"));

    map.delete(JsValue::new("Key-1"), context)?;
    assert_eq!(map.get_size(context)?, JsValue::new(1));

    let deleted_key_one = map.get(JsValue::new("Key-1"), context)?;

    assert_eq!(deleted_key_one, JsValue::undefined());

    let entries = map.entries(context)?;

    let _first_value = entries.next(context)?;

    let js_array = JsArray::new(context);

    let vec_one = vec![JsValue::new("first-key"), JsValue::new("first-value")];
    let vec_two = vec![JsValue::new("second-key"), JsValue::new("second-value")];

    js_array.push(JsArray::from_iter(vec_one, context), context)?;
    js_array.push(JsArray::from_iter(vec_two, context), context)?;

    let iter_map = JsMap::from_js_iterable(&js_array.into(), context)?;

    assert_eq!(
        iter_map.get(JsValue::new("first-key"), context)?,
        "first-value".into()
    );

    iter_map.set(
        JsValue::new("third-key"),
        JsValue::new("third-value"),
        context,
    )?;

    assert_eq!(iter_map.get_size(context)?, JsValue::new(3));

    Ok(())
}
