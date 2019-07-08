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

/// Utility function which takes an existing array object and puts additional
/// values on the end, correctly rewriting the length
fn add_to_array_object(array_ptr: Value, add_values: Vec<Value>) -> ResultValue {
    let orig_length: i32 = from_value(array_ptr.get_field_slice("length")).unwrap();

    for (n, value) in add_values.iter().enumerate() {
        let new_index = orig_length + (n as i32);
        array_ptr.set_field(new_index.to_string(), value.clone());
    }

    array_ptr.set_field_slice("length", to_value(orig_length + add_values.len() as i32));

    Ok(array_ptr)
}

/// Create a new array
pub fn make_array(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    // Make a new Object which will internally represent the Array (mapping
    // between indices and values): this creates an Object with no prototype
    this.set_field_slice("length", to_value(0i32));
    match args.len() {
        0 => create_array_object(this, Vec::new()),
        1 => {
            let array = create_array_object(this, Vec::new()).unwrap();
            let size: i32 = from_value(args[0].clone()).unwrap();
            array.set_field_slice("length", to_value(size));
            Ok(array)
        }
        _ => create_array_object(this, args),
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
        return Ok(this.clone());
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

/// Array.prototype.push ( ...items )
///
/// The arguments are appended to the end of the array, in the order in which
/// they appear. The new length of the array is returned as the result of the
/// call.
/// https://tc39.es/ecma262/#sec-array.prototype.push
pub fn push(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let new_array = add_to_array_object(this, args)?;
    Ok(new_array.get_field_slice("length"))
}

/// Array.prototype.pop ( )
///
/// The last element of the array is removed from the array and returned.
/// https://tc39.es/ecma262/#sec-array.prototype.pop
pub fn pop(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let curr_length: i32 = from_value(this.get_field_slice("length")).unwrap();
    if curr_length < 1 {
        return Err(to_value(
            "Cannot pop() on an array with zero length".to_string(),
        ));
    }
    let pop_index = curr_length - 1;
    let pop_value: Value = this.get_field(pop_index.to_string());
    this.remove_prop(&pop_index.to_string());
    this.set_field_slice("length", to_value(pop_index));
    Ok(pop_value)
}

/// Array.prototype.join ( separator )
///
/// The elements of the array are converted to Strings, and these Strings are
/// then concatenated, separated by occurrences of the separator. If no
/// separator is provided, a single comma is used as the separator.
/// https://tc39.es/ecma262/#sec-array.prototype.join
pub fn join(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let separator: String;

    if args.len() > 0 {
        separator = args[0].to_string();
    } else {
        separator = ",".to_string();
    }

    let mut elem_strs: Vec<String> = Vec::new();
    let length: i32 = from_value(this.get_field_slice("length")).unwrap();
    for n in 0..length {
        let elem_str: String = this.get_field(n.to_string()).to_string();
        elem_strs.push(elem_str);
    }

    Ok(to_value(elem_strs.join(&separator)))
}

/// Array.prototype.reverse ( )
///
/// The elements of the array are rearranged so as to reverse their order.
/// The object is returned as the result of the call.
/// https://tc39.es/ecma262/#sec-array.prototype.reverse
pub fn reverse(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let len: i32 = from_value(this.get_field_slice("length")).unwrap();
    let middle: i32 = len / 2;

    for lower in 0..middle {
        let upper = len - lower - 1;

        let upper_exists = this.has_field(upper.to_string());
        let lower_exists = this.has_field(lower.to_string());

        let upper_value = this.get_field(&upper.to_string());
        let lower_value = this.get_field(&lower.to_string());

        if upper_exists && lower_exists {
            this.set_field(upper.to_string(), lower_value);
            this.set_field(lower.to_string(), upper_value);
        } else if upper_exists {
            this.set_field(lower.to_string(), upper_value);
            this.remove_prop(&upper.to_string());
        } else if lower_exists {
            this.set_field(upper.to_string(), lower_value);
            this.remove_prop(&lower.to_string());
        }
    }

    Ok(this)
}

/// Array.prototype.shift ( )
///
/// The first element of the array is removed from the array and returned.
/// https://tc39.es/ecma262/#sec-array.prototype.shift
pub fn shift(this: Value, _: Value, _: Vec<Value>) -> ResultValue {
    let len: i32 = from_value(this.get_field_slice("length")).unwrap();

    if len == 0 {
        this.set_field_slice("length", to_value(0i32));
        // Since length is 0, this will be an Undefined value
        return Ok(this.get_field(&0.to_string()));
    }

    let first: Value = this.get_field(0.to_string());

    for k in 1..len {
        let from = k.to_string();
        let to = (k - 1).to_string();

        let from_value = this.get_field(&from);
        if from_value == Gc::new(ValueData::Undefined) {
            this.remove_prop(&to);
        } else {
            this.set_field(to, from_value);
        }
    }

    this.remove_prop(&(len - 1).to_string());
    this.set_field_slice("length", to_value(len - 1));

    Ok(first)
}

/// Array.prototype.unshift ( ...items )
///
/// The arguments are prepended to the start of the array, such that their order
/// within the array is the same as the order in which they appear in the
/// argument list.
/// https://tc39.es/ecma262/#sec-array.prototype.unshift
pub fn unshift(this: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let len: i32 = from_value(this.get_field_slice("length")).unwrap();
    let argc: i32 = args.len() as i32;

    if argc > 0 {
        for k in (1..=len).rev() {
            let from = (k - 1).to_string();
            let to = (k + argc - 1).to_string();

            let from_value = this.get_field(&from);
            if from_value != Gc::new(ValueData::Undefined) {
                this.set_field(to, from_value);
            } else {
                this.remove_prop(&to);
            }
        }
        for j in 0..argc {
            this.set_field_slice(&j.to_string(), args[j as usize].clone());
        }
    }

    this.set_field_slice("length", to_value(len + argc));
    Ok(to_value(len + argc))
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
    let concat_func = to_value(concat as NativeFunctionData);
    concat_func.set_field_slice("length", to_value(1 as i32));
    proto.set_field_slice("concat", concat_func);
    let push_func = to_value(push as NativeFunctionData);
    push_func.set_field_slice("length", to_value(1 as i32));
    proto.set_field_slice("push", push_func);
    proto.set_field_slice("pop", to_value(pop as NativeFunctionData));
    proto.set_field_slice("join", to_value(join as NativeFunctionData));
    proto.set_field_slice("reverse", to_value(reverse as NativeFunctionData));
    proto.set_field_slice("shift", to_value(shift as NativeFunctionData));
    proto.set_field_slice("unshift", to_value(unshift as NativeFunctionData));
    array.set_field_slice(PROTOTYPE, proto);
    array
}
/// Initialise the global object with the `Array` object
pub fn init(global: &Value) {
    global.set_field_slice("Array", _create(global));
}
