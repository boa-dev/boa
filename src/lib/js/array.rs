use crate::js::function::NativeFunctionData;
use crate::js::object::{Property, PROTOTYPE};
use crate::js::value::{from_value, to_value, ResultValue, Value, ValueData};
use gc::Gc;

/// Create a new array
#[allow(clippy::needless_pass_by_value)]
pub fn make_array(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let this_ptr = this.clone();
    // Make a new Object which will internally represent the Array (mapping
    // between indices and values): this creates an Object with no prototype
    match args.len() {
        0 => {
            this_ptr.set_field_slice("length", to_value(0_i32));
        }
        1 => {
            let length_chosen: i32 = from_value(args[0].clone()).unwrap();
            this_ptr.set_field_slice("length", to_value(length_chosen));
        }
        n => {
            this_ptr.set_field_slice("length", to_value(n));
            for (k, arg) in args.into_iter().enumerate() {
                let index_str = k.to_string();
                this_ptr.set_field(index_str, arg);
            }
        }
    }
    Ok(this_ptr)
}

/// Get an array's length
#[allow(clippy::needless_pass_by_value)]
pub fn get_array_length(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    // Access the inner hash map which represents the actual Array contents
    // (mapping between indices and values)
    Ok(this.get_field_slice("length"))
}

/// Create a new `Array` object
pub fn _create(global: &Value) -> Value {
    let array = to_value(make_array as NativeFunctionData);
    let proto = ValueData::new_obj(Some(global));
    let length = Property {
        configurable: false,
        enumerable: false,
        writable: false,
        value: Gc::new(ValueData::Undefined),
        get: to_value(get_array_length as NativeFunctionData),
        set: Gc::new(ValueData::Undefined),
    };
    proto.set_prop_slice("length", length);
    array.set_field_slice(PROTOTYPE, proto);
    array
}
/// Initialise the global object with the `Array` object
pub fn init(global: &Value) {
    global.set_field_slice("Array", _create(global));
}
