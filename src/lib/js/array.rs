use crate::js::function::NativeFunctionData;
use crate::js::object::{Property, PROTOTYPE};
use crate::js::value::{from_value, to_value, ResultValue, Value, ValueData};
use gc::Gc;

/// Utility function for creating array objects: array_obj can be any array with
/// prototype already set (it will be wiped and recreated from array_contents)
fn create_array_object(array_obj: Value, array_contents: Vec<Value>) -> ResultValue {
    let array_obj_ptr = array_obj.clone();

    // Wipe existing contents of the array object
    let orig_length: i32 = from_value(array_obj.get_field_slice("length")).unwrap();
    for n in 0..orig_length {
        array_obj_ptr.remove_prop(&n.to_string());
    }

    for (n, value) in array_contents.iter().enumerate() {
        array_obj_ptr.set_field(n.to_string(), value.clone());
    }

    array_obj_ptr.set_field_slice("length", to_value(array_contents.len() as i32));
    Ok(array_obj_ptr)
}

/// Create a new array
pub fn make_array(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    // Make a new Object which will internally represent the Array (mapping
    // between indices and values): this creates an Object with no prototype
    this.set_field_slice("length", to_value(0i32));
    match args.len() {
        0 => {
            create_array_object(this, Vec::new())
        }
        1 => {
            let array = create_array_object(this, Vec::new()).unwrap();
            let size: i32 = from_value(args[0].clone()).unwrap();
            array.set_field_slice("length", to_value(size));
            Ok(array)
        }
        _ => {
            create_array_object(this, args)
        }
    }
}

/// Get an array's length
pub fn get_array_length(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    // Access the inner hash map which represents the actual Array contents
    // (mapping between indices and values)
    Ok(this.get_field_slice("length"))
}

/// Array.prototype.concat(...arguments)
/// 
/// When the concat method is called with zero or more arguments, it returns an
/// array containing the array elements of the object followed by the array
/// elements of each argument in order.
/// https://tc39.es/ecma262/#sec-array.prototype.concat
pub fn concat(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    if args.len() == 0 {
        // If concat is called with no arguments, it returns the original array
        return Ok(this.clone())
    }

    // Make a new array (using this object as the prototype basis for the new
    // one)
    let mut new_values: Vec<Value> = Vec::new();

    let this_length: i32 = from_value(this.get_field_slice("length")).unwrap();
    for n in 0..this_length {
        new_values.push(this.get_field(n.to_string()));
    }

    for concat_array in args {
        let concat_length: i32 = from_value(concat_array.get_field_slice("length")).unwrap();
        for n in 0..concat_length {
            new_values.push(concat_array.get_field(n.to_string()));
        }
    }

    create_array_object(this, new_values)
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
    proto.set_field_slice("concat", to_value(concat as NativeFunctionData));
    array.set_field_slice(PROTOTYPE, proto);
    array
}
/// Initialise the global object with the `Array` object
pub fn init(global: &Value) {
    global.set_field_slice("Array", _create(global));
}
