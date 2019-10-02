use crate::exec::Interpreter;
use crate::builtins::function::NativeFunctionData;
/// The JSON Object
/// <https://tc39.github.io/ecma262/#sec-json-object>
use crate::builtins::value::{to_value, ResultValue, Value, ValueData};
use serde_json::{self, to_string_pretty, Value as JSONValue};

/// Parse a JSON string into a Javascript object
/// <https://tc39.github.io/ecma262/#sec-json.parse>
pub fn parse(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    match serde_json::from_str::<JSONValue>(
        &args
            .get(0)
            .expect("cannot get argument for JSON.parse")
            .clone()
            .to_string(),
    ) {
        Ok(json) => Ok(to_value(json)),
        Err(err) => Err(to_value(err.to_string())),
    }
}
/// Process a Javascript object into a JSON string
pub fn stringify(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let obj = args.get(0).expect("cannot get argument for JSON.stringify");
    let json = obj.to_json();
    Ok(to_value(to_string_pretty(&json).expect("")))
}

/// Create a new `JSON` object
pub fn _create(global: &Value) -> Value {
    let object = ValueData::new_obj(Some(global));
    object.set_field_slice("stringify", to_value(stringify as NativeFunctionData));
    object.set_field_slice("parse", to_value(parse as NativeFunctionData));
    object
}

/// Initialise the global object with the `JSON` object
pub fn init(global: &Value) {
    global.set_field_slice("JSON", _create(global));
}
