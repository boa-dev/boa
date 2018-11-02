use gc::Gc;
use js::function::Function;
use js::object::PROTOTYPE;
use js::value::{to_value, ResultValue, Value, ValueData};

/// Create a new error
pub fn make_error(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    if args.len() >= 1 {
        this.set_field_slice("message", to_value(args.get(0).unwrap().to_string()));
    }
    Ok(Gc::new(ValueData::Undefined))
}
/// Get the string representation of the error
pub fn to_string(_: Vec<Value>, _: Value, _: Value, this: Value) -> ResultValue {
    let name = this.get_field_slice("name");
    let message = this.get_field_slice("message");
    Ok(to_value(format!("{}: {}", name, message).to_string()))
}
/// Create a new `Error` object
pub fn _create(global: Value) -> Value {
    let prototype = ValueData::new_obj(Some(global));
    let prototype_ptr = prototype;
    prototype_ptr.set_field_slice("message", to_value(""));
    prototype_ptr.set_field_slice("name", to_value("Error"));
    prototype_ptr.set_field_slice("toString", to_value(to_string));
    let error = to_value(make_error);
    let error_ptr = error;
    error_ptr.set_field_slice(PROTOTYPE, prototype);
    error
}
/// Initialise the global object with the `Error` object
pub fn init(global: Value) {
    global.set_field_slice("Error", _create(global.clone()));
}
