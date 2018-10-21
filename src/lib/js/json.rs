use js::function::Function;
/// The JSON Object
/// https://tc39.github.io/ecma262/#sec-json-object
use js::value::{to_value, ResultValue, Value};
use serde_json::{self, to_string_pretty, Value as JSONValue};

/// Parse a JSON string into a Javascript object
/// https://tc39.github.io/ecma262/#sec-json.parse
pub fn parse(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    match serde_json::from_str::<JSONValue>(&args.get(0).unwrap().clone().to_string()) {
        Ok(json) => Ok(to_value(json)),
        Err(err) => Err(to_value(err.to_string())),
    }
}
/// Process a Javascript object into a JSON string
pub fn stringify(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    let obj = args.get(0).unwrap();
    let json = obj.to_json();
    Ok(to_value(to_string_pretty(&json).unwrap()))
}

/// Create a new `JSON` object
pub fn _create(global: Value) -> Value {
    let object = Value::new_obj(Some(global));
    object.set_field_slice("stringify", Function::make(stringify, &["JSON"]));
    object.set_field_slice("parse", Function::make(parse, &["JSON_string"]));
    object
}

/// Initialise the global object with the `JSON` object
pub fn init(global: Value) {
    global.set_field_slice("JSON", _create(global.clone()));
}
