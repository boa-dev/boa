/// The JSON Object
/// https://tc39.github.io/ecma262/#sec-json-object
use gc::GcCell;
use js::value::{to_value, ResultValue, Value, ValueData};

/// Parse a JSON string into a Javascript object
/// https://tc39.github.io/ecma262/#sec-json.parse
pub fn parse(args: Vec<Value>) -> ResultValue {
    match serde_json::from_str(args.get(0).borrow().to_str().as_slice()) {
        Ok(json) => Ok(GcCell::new(ValueData::from_json(json))),
        Err(err) => Err(GcCell::new(Value::String(err.to_str()))),
    }
}
/// Process a Javascript object into a JSON string
pub fn stringify(args: Vec<Value>) -> ResultValue {
    let obj = args.get(0);
    let json = serde_json::to_string_pretty(obj.borrow()).unwrap();
    Ok(GcCell::new(Value::String(json.to_pretty_str())))
}

/// Create a new `JSON` object
pub fn _create(global: Value) -> Value {
    let object = ValueData::new_obj(Some(global));
    let object_ptr = object.borrow();
    object_ptr.set_field_slice("stringify", to_value(stringify));
    object_ptr.set_field_slice("parse", to_value(parse));
    object
}

/// Initialise the global object with the `JSON` object
pub fn init(global: Value) {
    let global_ptr = global.borrow();
    global_ptr.set_field_slice("JSON", _create(global));
}
