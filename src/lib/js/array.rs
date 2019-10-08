use crate::{
    exec::Interpreter,
    js::{
        function::NativeFunctionData,
        object::{Object, ObjectKind, PROTOTYPE},
        property::Property,
        value::{from_value, to_value, ResultValue, Value, ValueData},
    },
};
use gc::Gc;

/// Utility function for creating array objects: `array_obj` can be any array with
/// prototype already set (it will be wiped and recreated from `array_contents`)
fn construct_array(array_obj: &Value, array_contents: &[Value]) -> ResultValue {
    let array_obj_ptr = array_obj.clone();

    // Wipe existing contents of the array object
    let orig_length: i32 =
        from_value(array_obj.get_field_slice("length")).expect("failed to convert length to i32");
    for n in 0..orig_length {
        array_obj_ptr.remove_prop(&n.to_string());
    }

    array_obj_ptr.set_field_slice("length", to_value(array_contents.len() as i32));
    for (n, value) in array_contents.iter().enumerate() {
        array_obj_ptr.set_field(n.to_string(), value.clone());
    }
    Ok(array_obj_ptr)
}

/// Utility function which takes an existing array object and puts additional
/// values on the end, correctly rewriting the length
fn add_to_array_object(array_ptr: &Value, add_values: &[Value]) -> ResultValue {
    let orig_length: i32 =
        from_value(array_ptr.get_field_slice("length")).expect("failed to conveert lenth to i32");

    for (n, value) in add_values.iter().enumerate() {
        let new_index = orig_length + (n as i32);
        array_ptr.set_field(new_index.to_string(), value.clone());
    }

    array_ptr.set_field_slice("length", to_value(orig_length + add_values.len() as i32));

    Ok(array_ptr.clone())
}

/// Create a new array
pub fn make_array(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    // Make a new Object which will internally represent the Array (mapping
    // between indices and values): this creates an Object with no prototype
    this.set_field_slice("length", to_value(0_i32));
    // This value is used by console.log and other routines to match Object type
    // to its Javascript Identifier (global constructor method name)
    this.set_kind(ObjectKind::Array);
    match args.len() {
        0 => construct_array(this, &[]),
        1 => {
            let array = construct_array(this, &[]).expect("Could not construct array");
            let size: i32 = from_value(args[0].clone()).expect("Could not convert argument to i32");
            array.set_field_slice("length", to_value(size));
            Ok(array)
        }
        _ => construct_array(this, args),
    }
}

/// Get an array's length
pub fn get_array_length(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    // Access the inner hash map which represents the actual Array contents
    // (mapping between indices and values)
    Ok(this.get_field_slice("length"))
}

/// Array.prototype.concat(...arguments)
///
/// When the concat method is called with zero or more arguments, it returns an
/// array containing the array elements of the object followed by the array
/// elements of each argument in order.
/// <https://tc39.es/ecma262/#sec-array.prototype.concat>
pub fn concat(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        // If concat is called with no arguments, it returns the original array
        return Ok(this.clone());
    }

    // Make a new array (using this object as the prototype basis for the new
    // one)
    let mut new_values: Vec<Value> = Vec::new();

    let this_length: i32 = from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");
    for n in 0..this_length {
        new_values.push(this.get_field(&n.to_string()));
    }

    for concat_array in args {
        let concat_length: i32 = from_value(concat_array.get_field_slice("length")).expect("Could not convert argument to i32");
        for n in 0..concat_length {
            new_values.push(concat_array.get_field(&n.to_string()));
        }
    }

    construct_array(this, &new_values)
}

/// Array.prototype.push ( ...items )
///
/// The arguments are appended to the end of the array, in the order in which
/// they appear. The new length of the array is returned as the result of the
/// call.
/// <https://tc39.es/ecma262/#sec-array.prototype.push>
pub fn push(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let new_array = add_to_array_object(this, args)?;
    Ok(new_array.get_field_slice("length"))
}

/// Array.prototype.pop ( )
///
/// The last element of the array is removed from the array and returned.
/// <https://tc39.es/ecma262/#sec-array.prototype.pop>
pub fn pop(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let curr_length: i32 = from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");
    if curr_length < 1 {
        return Err(to_value(
            "Cannot pop() on an array with zero length".to_string(),
        ));
    }
    let pop_index = curr_length - 1;
    let pop_value: Value = this.get_field(&pop_index.to_string());
    this.remove_prop(&pop_index.to_string());
    this.set_field_slice("length", to_value(pop_index));
    Ok(pop_value)
}

/// Array.prototype.join ( separator )
///
/// The elements of the array are converted to Strings, and these Strings are
/// then concatenated, separated by occurrences of the separator. If no
/// separator is provided, a single comma is used as the separator.
/// <https://tc39.es/ecma262/#sec-array.prototype.join>
pub fn join(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let separator = if args.is_empty() {
        String::from(",")
    } else {
        args[0].to_string()
    };

    let mut elem_strs: Vec<String> = Vec::new();
    let length: i32 = from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");
    for n in 0..length {
        let elem_str: String = this.get_field(&n.to_string()).to_string();
        elem_strs.push(elem_str);
    }

    Ok(to_value(elem_strs.join(&separator)))
}

/// Array.prototype.reverse ( )
///
/// The elements of the array are rearranged so as to reverse their order.
/// The object is returned as the result of the call.
/// <https://tc39.es/ecma262/#sec-array.prototype.reverse/>
pub fn reverse(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let len: i32 = from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");
    let middle: i32 = len / 2;

    for lower in 0..middle {
        let upper = len - lower - 1;

        let upper_exists = this.has_field(&upper.to_string());
        let lower_exists = this.has_field(&lower.to_string());

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

    Ok(this.clone())
}

/// Array.prototype.shift ( )
///
/// The first element of the array is removed from the array and returned.
/// <https://tc39.es/ecma262/#sec-array.prototype.shift/>
pub fn shift(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let len: i32 = from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");

    if len == 0 {
        this.set_field_slice("length", to_value(0_i32));
        // Since length is 0, this will be an Undefined value
        return Ok(this.get_field(&0.to_string()));
    }

    let first: Value = this.get_field(&0.to_string());

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
/// <https://tc39.es/ecma262/#sec-array.prototype.unshift/>
pub fn unshift(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let len: i32 = from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");
    let arg_c: i32 = args.len() as i32;

    if arg_c > 0 {
        for k in (1..=len).rev() {
            let from = (k - 1).to_string();
            let to = (k + arg_c - 1).to_string();

            let from_value = this.get_field(&from);
            if from_value == Gc::new(ValueData::Undefined) {
                this.remove_prop(&to);
            } else {
                this.set_field(to, from_value);
            }
        }
        for j in 0..arg_c {
            this.set_field_slice(&j.to_string(), args[j as usize].clone());
        }
    }

    this.set_field_slice("length", to_value(len + arg_c));
    Ok(to_value(len + arg_c))
}

/// Create a new `Array` object
pub fn create_constructor(global: &Value) -> Value {
    // Create Constructor
    let mut array_constructor = Object::default();
    array_constructor.kind = ObjectKind::Function;
    array_constructor.set_internal_method("construct", make_array);
    // Todo: add call function
    array_constructor.set_internal_method("call", make_array);

    // Create prototype
    let array_prototype = ValueData::new_obj(Some(global));

    let length = Property::default().get(to_value(get_array_length as NativeFunctionData));

    array_prototype.set_prop_slice("length", length);
    let concat_func = to_value(concat as NativeFunctionData);
    concat_func.set_field_slice("length", to_value(1_i32));
    array_prototype.set_field_slice("concat", concat_func);
    let push_func = to_value(push as NativeFunctionData);
    push_func.set_field_slice("length", to_value(1_i32));

    array_prototype.set_field_slice("push", push_func);
    array_prototype.set_field_slice("pop", to_value(pop as NativeFunctionData));
    array_prototype.set_field_slice("join", to_value(join as NativeFunctionData));
    array_prototype.set_field_slice("reverse", to_value(reverse as NativeFunctionData));
    array_prototype.set_field_slice("shift", to_value(shift as NativeFunctionData));
    array_prototype.set_field_slice("unshift", to_value(unshift as NativeFunctionData));

    let array = to_value(array_constructor);
    array.set_field_slice(PROTOTYPE, to_value(array_prototype.clone()));

    array_prototype.set_field_slice("constructor", array.clone());
    array
}

#[cfg(test)]
mod tests {
    use crate::exec::Executor;
    use crate::forward;

    #[test]
    fn concat() {
        //TODO: array display formatter
        let mut engine = Executor::new();
        let init = r#"
        let empty = new Array();
        let one = new Array(1);
        "#;
        forward(&mut engine, init);
        // Empty ++ Empty
        let _ee = forward(&mut engine, "empty.concat(empty)");
        //assert_eq!(ee, String::from(""));
        // Empty ++ NonEmpty
        let _en = forward(&mut engine, "empty.concat(one)");
        //assert_eq!(en, String::from("a"));
        // NonEmpty ++ Empty
        let _ne = forward(&mut engine, "one.concat(empty)");
        //assert_eq!(ne, String::from("a.b.c"));
        // NonEmpty ++ NonEmpty
        let _nn = forward(&mut engine, "one.concat(one)");
        //assert_eq!(nn, String::from("a.b.c"));
    }

    #[test]
    fn join() {
        let mut engine = Executor::new();
        let init = r#"
        let empty = [ ];
        let one = ["a"];
        let many = ["a", "b", "c"];
        "#;
        forward(&mut engine, init);
        // Empty
        let empty = forward(&mut engine, "empty.join('.')");
        assert_eq!(empty, String::from(""));
        // One
        let one = forward(&mut engine, "one.join('.')");
        assert_eq!(one, String::from("a"));
        // Many
        let many = forward(&mut engine, "many.join('.')");
        assert_eq!(many, String::from("a.b.c"));
    }
}
