use crate::js::function::NativeFunctionData;
use crate::js::object::PROTOTYPE;
use crate::js::value::{to_value, ResultValue, Value, ValueData};
use gc::Gc;

/// Create a new error
pub fn make_error(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    if !args.is_empty() {
        this.set_field_slice("message", to_value(args.get(0).unwrap().to_string()));
    }
    Ok(Gc::new(ValueData::Undefined))
}
/// Get the string representation of the error
pub fn to_string(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let name = this.get_field_slice("name");
    let message = this.get_field_slice("message");
    Ok(to_value(format!("{}: {}", name, message).to_string()))
}
/// Create a new `Error` object
pub fn _create(global: &Value) -> Value {
    let prototype = ValueData::new_obj(Some(global));
    prototype.set_field_slice("message", to_value(""));
    prototype.set_field_slice("name", to_value("Error"));
    prototype.set_field_slice("toString", to_value(to_string as NativeFunctionData));
    let error = to_value(make_error as NativeFunctionData);
    error.set_field_slice(PROTOTYPE, prototype);
    error
}
/// Initialise the global object with the `Error` object
pub fn init(global: &Value) {
    global.set_field_slice("Error", _create(global));
}
