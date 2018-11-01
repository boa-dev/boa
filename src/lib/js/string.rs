use js::function::Function;
use js::object::{Property, PROTOTYPE};
use js::value::{from_value, to_value, ResultValue, Value};

/// Create new string
pub fn make_string(_: Vec<Value>, _: Value, _: Value, this: Value) -> ResultValue {
    this.set_field_slice("length", to_value(0i32));
    Ok(Value::undefined())
}
/// Get a string's length
pub fn get_string_length(_: Vec<Value>, _: Value, _: Value, this: Value) -> ResultValue {
    let this_str: String = from_value(this).unwrap();
    Ok(to_value::<i32>(this_str.len() as i32))
}
/// Create a new `String` object
pub fn _create(global: Value) -> Value {
    let string = Function::make(make_string, &["string"]);
    let proto = Value::new_obj(Some(global));
    let prop = Property {
        configurable: false,
        enumerable: false,
        writable: false,
        value: Value::undefined(),
        get: Function::make(get_string_length, &[]),
        set: Value::undefined(),
    };
    proto.set_prop_slice("length", prop);
    string.set_field_slice(PROTOTYPE, proto);
    string
}
/// Initialise the `String` object on the global object
pub fn init(global: Value) {
    global.set_field_slice("String", _create(global));
}
