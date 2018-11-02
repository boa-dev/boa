use gc::Gc;
use js::function::NativeFunctionData;
use js::object::{Property, PROTOTYPE};
use js::value::{from_value, to_value, ResultValue, Value, ValueData};

/// Create new string
pub fn make_string(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    this.set_field_slice("length", to_value(0i32));
    Ok(Gc::new(ValueData::Undefined))
}
/// Get a string's length
pub fn get_string_length(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let this_str: String = from_value(this).unwrap();
    Ok(to_value::<i32>(this_str.len() as i32))
}
/// Create a new `String` object
pub fn _create(global: Value) -> Value {
    let string = to_value(make_string as NativeFunctionData);
    let proto = ValueData::new_obj(Some(global));
    let prop = Property {
        configurable: false,
        enumerable: false,
        writable: false,
        value: Gc::new(ValueData::Undefined),
        get: to_value(get_string_length),
        set: Gc::new(ValueData::Undefined),
    };
    proto.set_prop_slice("length", prop);
    string.set_field_slice(PROTOTYPE, proto);
    string
}
/// Initialise the `String` object on the global object
pub fn init(global: Value) {
    global.set_field_slice("String", _create(global));
}
