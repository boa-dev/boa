use js::function::Function;
use js::object::PROTOTYPE;
use js::value::{to_value, ResultValue, Value};

/// Create a new error
pub fn make_error(args: Vec<Value>, _: Value, _: Value, this: Value) -> ResultValue {
    if args.len() >= 1 {
        this.set_field_slice("message", to_value(args.get(0).unwrap().to_string()));
    }
    Ok(Value::undefined())
}
/// Get the string representation of the error
pub fn to_string(_: Vec<Value>, _: Value, _: Value, this: Value) -> ResultValue {
    let name = this.get_field_slice("name");
    let message = this.get_field_slice("message");
    Ok(to_value(format!("{}: {}", name, message).to_string()))
}
/// Create a new `Error` object
pub fn _create(global: Value) -> Value {
    let prototype = Value::new_obj(Some(global));
    prototype.set_field_slice("message", to_value(""));
    prototype.set_field_slice("name", to_value("Error"));
    prototype.set_field_slice("toString", Function::make(to_string, &[]));
    let error = Function::make(make_error, &["message"]);
    error.set_field_slice(PROTOTYPE, prototype);
    error
}
/// Initialise the global object with the `Error` object
pub fn init(global: Value) {
    global.set_field_slice("Error", _create(global.clone()));
}
