use gc::Gc;
use crate::js::function::NativeFunctionData;
use crate::js::object::{Property, PROTOTYPE};
use crate::js::value::{from_value, to_value, ResultValue, Value, ValueData};

/// Create new string
/// https://searchfox.org/mozilla-central/source/js/src/vm/StringObject.h#19
pub fn make_string(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    // If we're constructing a string, we should set the initial length
    // To do this we need to convert the string back to a Rust String, then get the .len()
    let a: String = from_value(args[0].clone()).unwrap();
    this.set_field_slice("length", to_value(a.len() as i32));
    Ok(this)
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
        get: to_value(get_string_length as NativeFunctionData),
        set: Gc::new(ValueData::Undefined),
    };
    proto.set_prop_slice("length", prop);
    string.set_field_slice(PROTOTYPE, proto);
    string
}
/// Initialise the `String` object on the global object
pub fn init(global: Value) {
    global.set_field_slice("String", _create(global.clone()));
}
