use boa_engine::{
    object::JsMap, 
    Context, JsResult, JsValue
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

    Ok(())
}
