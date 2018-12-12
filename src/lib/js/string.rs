use crate::js::function::NativeFunctionData;
use crate::js::object::{Property, PROTOTYPE};
use crate::js::value::{from_value, to_value, ResultValue, Value, ValueData};
use gc::Gc;

/// Create new string
/// https://searchfox.org/mozilla-central/source/js/src/vm/StringObject.h#19
// This gets called when a new String() is created, it's called by exec:346
pub fn make_string(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    // If we're constructing a string, we should set the initial length
    // To do this we need to convert the string back to a Rust String, then get the .len()
    // let a: String = from_value(args[0].clone()).unwrap();
    // this.set_field_slice("length", to_value(a.len() as i32));

    this.set_private_field_slice("PrimitiveValue", args[0].clone());
    Ok(this)
}

/// Get a string's length
pub fn get_string_length(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let this_str: String =
        from_value(this.get_private_field(String::from("PrimitiveValue"))).unwrap();
    Ok(to_value::<i32>(this_str.len() as i32))
}

/// Get the string representation of the error
pub fn to_string(_: Value, _: Value, _: Vec<Value>) -> ResultValue {
    Ok(to_value(format!("{}", String::from("test")).to_string()))
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
    proto.set_field_slice("toString", to_value(to_string as NativeFunctionData));
    string.set_field_slice(PROTOTYPE, proto);
    string
}
/// Initialise the `String` object on the global object
pub fn init(global: Value) {
    global.set_field_slice("String", _create(global.clone()));
}
