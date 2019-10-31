use crate::{
    builtins::{
        function::NativeFunctionData,
        object::{Object, ObjectKind, INSTANCE_PROTOTYPE, PROTOTYPE},
        property::Property,
        value::{from_value, to_value, undefined, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use gc::Gc;
use std::borrow::Borrow;
use std::cmp::{max, min};

pub(crate) fn new_array(interpreter: &Interpreter) -> ResultValue {
    let array = ValueData::new_obj(Some(
        &interpreter
            .get_realm()
            .environment
            .get_global_object()
            .expect("Could not get global object"),
    ));
    array.set_kind(ObjectKind::Array);
    array.borrow().set_internal_slot(
        INSTANCE_PROTOTYPE,
        interpreter
            .get_realm()
            .environment
            .get_binding_value("Array")
            .borrow()
            .get_field_slice(PROTOTYPE),
    );
    array.borrow().set_field_slice("length", to_value(0));
    Ok(array)
}

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
        array_obj_ptr.set_field_slice(&n.to_string(), value.clone());
    }
    Ok(array_obj_ptr)
}

/// Utility function which takes an existing array object and puts additional
/// values on the end, correctly rewriting the length
pub(crate) fn add_to_array_object(array_ptr: &Value, add_values: &[Value]) -> ResultValue {
    let orig_length: i32 =
        from_value(array_ptr.get_field_slice("length")).expect("failed to conveert lenth to i32");

    for (n, value) in add_values.iter().enumerate() {
        let new_index = orig_length.wrapping_add(n as i32);
        array_ptr.set_field_slice(&new_index.to_string(), value.clone());
    }

    array_ptr.set_field_slice(
        "length",
        to_value(orig_length.wrapping_add(add_values.len() as i32)),
    );

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
            let size: i32 = from_value(args.get(0).expect("Could not get argument").clone())
                .expect("Could not convert argument to i32");
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

    let this_length: i32 =
        from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");
    for n in 0..this_length {
        new_values.push(this.get_field_slice(&n.to_string()));
    }

    for concat_array in args {
        let concat_length: i32 = from_value(concat_array.get_field_slice("length"))
            .expect("Could not convert argument to i32");
        for n in 0..concat_length {
            new_values.push(concat_array.get_field_slice(&n.to_string()));
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
    let curr_length: i32 =
        from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");
    if curr_length < 1 {
        return Ok(Gc::new(ValueData::Undefined));
    }
    let pop_index = curr_length.wrapping_sub(1);
    let pop_value: Value = this.get_field_slice(&pop_index.to_string());
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
        args.get(0).expect("Could not get argument").to_string()
    };

    let mut elem_strs: Vec<String> = Vec::new();
    let length: i32 =
        from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");
    for n in 0..length {
        let elem_str: String = this.get_field_slice(&n.to_string()).to_string();
        elem_strs.push(elem_str);
    }

    Ok(to_value(elem_strs.join(&separator)))
}

/// Array.prototype.reverse ( )
///
/// The elements of the array are rearranged so as to reverse their order.
/// The object is returned as the result of the call.
/// <https://tc39.es/ecma262/#sec-array.prototype.reverse/>
#[allow(clippy::else_if_without_else)]
pub fn reverse(this: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    let len: i32 =
        from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");
    let middle: i32 = len.wrapping_div(2);

    for lower in 0..middle {
        let upper = len.wrapping_sub(lower).wrapping_sub(1);

        let upper_exists = this.has_field(&upper.to_string());
        let lower_exists = this.has_field(&lower.to_string());

        let upper_value = this.get_field_slice(&upper.to_string());
        let lower_value = this.get_field_slice(&lower.to_string());

        if upper_exists && lower_exists {
            this.set_field_slice(&upper.to_string(), lower_value);
            this.set_field_slice(&lower.to_string(), upper_value);
        } else if upper_exists {
            this.set_field_slice(&lower.to_string(), upper_value);
            this.remove_prop(&upper.to_string());
        } else if lower_exists {
            this.set_field_slice(&upper.to_string(), lower_value);
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
    let len: i32 =
        from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");

    if len == 0 {
        this.set_field_slice("length", to_value(0_i32));
        // Since length is 0, this will be an Undefined value
        return Ok(this.get_field_slice(&0.to_string()));
    }

    let first: Value = this.get_field_slice(&0.to_string());

    for k in 1..len {
        let from = k.to_string();
        let to = (k.wrapping_sub(1)).to_string();

        let from_value = this.get_field_slice(&from);
        if from_value == Gc::new(ValueData::Undefined) {
            this.remove_prop(&to);
        } else {
            this.set_field_slice(&to, from_value);
        }
    }

    let final_index = len.wrapping_sub(1);
    this.remove_prop(&(final_index).to_string());
    this.set_field_slice("length", to_value(final_index));

    Ok(first)
}

/// Array.prototype.unshift ( ...items )
///
/// The arguments are prepended to the start of the array, such that their order
/// within the array is the same as the order in which they appear in the
/// argument list.
/// <https://tc39.es/ecma262/#sec-array.prototype.unshift/>
pub fn unshift(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let len: i32 =
        from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");
    let arg_c: i32 = args.len() as i32;

    if arg_c > 0 {
        for k in (1..=len).rev() {
            let from = (k.wrapping_sub(1)).to_string();
            let to = (k.wrapping_add(arg_c).wrapping_sub(1)).to_string();

            let from_value = this.get_field_slice(&from);
            if from_value == Gc::new(ValueData::Undefined) {
                this.remove_prop(&to);
            } else {
                this.set_field_slice(&to, from_value);
            }
        }
        for j in 0..arg_c {
            this.set_field_slice(
                &j.to_string(),
                args.get(j as usize)
                    .expect("Could not get argument")
                    .clone(),
            );
        }
    }

    let temp = len.wrapping_add(arg_c);
    this.set_field_slice("length", to_value(temp));
    Ok(to_value(temp))
}

/// Array.prototype.every ( callback, [ thisArg ] )
///
/// The every method executes the provided callback function once for each
/// element present in the array until it finds the one where callback returns
/// a falsy value. It returns `false` if it finds such element, otherwise it
/// returns `true`.
/// <https://tc39.es/ecma262/#sec-array.prototype.every/>
pub fn every(this: &Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        return Err(to_value(
            "missing callback when calling function Array.prototype.every".to_string(),
        ));
    }
    let callback = &args[0];
    let this_arg = if args.len() > 1 {
        args[1].clone()
    } else {
        Gc::new(ValueData::Undefined)
    };
    let mut i = 0;
    let max_len: i32 = from_value(this.get_field_slice("length")).unwrap();
    let mut len = max_len;
    while i < len {
        let element = this.get_field_slice(&i.to_string());
        let arguments = vec![element.clone(), to_value(i), this.clone()];
        let result = interpreter.call(callback, &this_arg, arguments)?.is_true();
        if !result {
            return Ok(to_value(false));
        }
        len = min(max_len, from_value(this.get_field_slice("length")).unwrap());
        i += 1;
    }
    Ok(to_value(true))
}

/// Array.prototype.map ( callback, [ thisArg ] )
///
/// For each element in the array the callback function is called, and a new
/// array is constructed from the return values of these calls.
/// <https://tc39.es/ecma262/#sec-array.prototype.map>
pub fn map(this: &Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        return Err(to_value(
            "missing argument 0 when calling function Array.prototype.map",
        ));
    }

    let callback = args.get(0).cloned().unwrap_or_else(undefined);
    let this_val = args.get(1).cloned().unwrap_or_else(undefined);

    let length: i32 =
        from_value(this.get_field_slice("length")).expect("Could not get `length` property.");

    let new = new_array(&interpreter)?;

    let values = (0..length)
        .map(|idx| {
            let element = this.get_field(&idx.to_string());

            let args = vec![element, to_value(idx), new.clone()];

            interpreter
                .call(&callback, &this_val, args)
                .unwrap_or_else(|_| undefined())
        })
        .collect::<Vec<Value>>();

    construct_array(&new, &values)
}

/// Array.prototype.indexOf ( searchElement[, fromIndex ] )
///
///
/// indexOf compares searchElement to the elements of the array, in ascending order,
/// using the Strict Equality Comparison algorithm, and if found at one or more indices,
/// returns the smallest such index; otherwise, -1 is returned.
///
/// The optional second argument fromIndex defaults to 0 (i.e. the whole array is searched).
/// If it is greater than or equal to the length of the array, -1 is returned,
/// i.e. the array will not be searched. If it is negative, it is used as the offset
/// from the end of the array to compute fromIndex. If the computed index is less than 0,
/// the whole array will be searched.
/// <https://tc39.es/ecma262/#sec-array.prototype.indexof>
pub fn index_of(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    // If no arguments, return -1. Not described in spec, but is what chrome does.
    if args.is_empty() {
        return Ok(to_value(-1));
    }

    let search_element = args[0].clone();
    let len: i32 = from_value(this.get_field_slice("length"))
        .expect("Expected array property \"length\" is not set.");

    let mut idx = match args.get(1) {
        Some(from_idx_ptr) => {
            let from_idx = from_value(from_idx_ptr.clone())
                .expect("Error parsing \"Array.prototype.indexOf - fromIndex\" argument");

            if from_idx < 0 {
                len + from_idx
            } else {
                from_idx
            }
        }
        None => 0,
    };

    while idx < len {
        let check_element = this.get_field_slice(&idx.to_string()).clone();

        if check_element == search_element {
            return Ok(to_value(idx));
        }

        idx += 1;
    }

    Ok(to_value(-1))
}

/// Array.prototype.lastIndexOf ( searchElement[, fromIndex ] )
///
///
/// lastIndexOf compares searchElement to the elements of the array in descending order
/// using the Strict Equality Comparison algorithm, and if found at one or more indices,
/// returns the largest such index; otherwise, -1 is returned.
///
/// The optional second argument fromIndex defaults to the array's length minus one
/// (i.e. the whole array is searched). If it is greater than or equal to the length of the array,
/// the whole array will be searched. If it is negative, it is used as the offset from the end
/// of the array to compute fromIndex. If the computed index is less than 0, -1 is returned.
/// <https://tc39.es/ecma262/#sec-array.prototype.lastindexof>
pub fn last_index_of(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    // If no arguments, return -1. Not described in spec, but is what chrome does.
    if args.is_empty() {
        return Ok(to_value(-1));
    }

    let search_element = args[0].clone();
    let len: i32 = from_value(this.get_field_slice("length"))
        .expect("Expected array property \"length\" is not set.");

    let mut idx = match args.get(1) {
        Some(from_idx_ptr) => {
            let from_idx = from_value(from_idx_ptr.clone())
                .expect("Error parsing \"Array.prototype.indexOf - fromIndex\" argument");

            if from_idx >= 0 {
                min(from_idx, len - 1)
            } else {
                len + from_idx
            }
        }
        None => len - 1,
    };

    while idx >= 0 {
        let check_element = this.get_field_slice(&idx.to_string()).clone();

        if check_element == search_element {
            return Ok(to_value(idx));
        }

        idx -= 1;
    }

    Ok(to_value(-1))
}

/// Array.prototype.find ( callback, [thisArg] )
///
/// The find method executes the callback function once for each index of the array
/// until the callback returns a truthy value. If so, find immediately returns the value
/// of that element. Otherwise, find returns undefined.
/// <https://tc39.es/ecma262/#sec-array.prototype.find>
pub fn find(this: &Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        return Err(to_value(
            "missing callback when calling function Array.prototype.find".to_string(),
        ));
    }
    let callback = &args[0];
    let this_arg = if args.len() > 1 {
        args[1].clone()
    } else {
        Gc::new(ValueData::Undefined)
    };
    let len: i32 = from_value(this.get_field_slice("length")).unwrap();
    for i in 0..len {
        let element = this.get_field_slice(&i.to_string());
        let arguments = vec![element.clone(), to_value(i), this.clone()];
        let result = interpreter.call(callback, &this_arg, arguments)?;
        if result.is_true() {
            return Ok(element);
        }
    }
    Ok(Gc::new(ValueData::Undefined))
}

/// Array.prototype.findIndex ( predicate [ , thisArg ] )
///
/// This method executes the provided predicate function for each element of the array.
/// If the predicate function returns `true` for an element, this method returns the index of the element.
/// If all elements return `false`, the value `-1` is returned.
/// <https://tc39.es/ecma262/#sec-array.prototype.findindex/>
pub fn find_index(this: &Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        return Err(to_value(
            "Missing argument for Array.prototype.findIndex".to_string(),
        ));
    }

    let predicate_arg = args.get(0).expect("Could not get `predicate` argument.");

    let this_arg = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| Gc::new(ValueData::Undefined));

    let length: i32 =
        from_value(this.get_field_slice("length")).expect("Could not get `length` property.");

    for i in 0..length {
        let element = this.get_field_slice(&i.to_string());
        let arguments = vec![element.clone(), to_value(i), this.clone()];

        let result = interpreter.call(predicate_arg, &this_arg, arguments)?;

        if result.is_true() {
            return Ok(Gc::new(ValueData::Number(f64::from(i))));
        }
    }

    Ok(Gc::new(ValueData::Number(f64::from(-1))))
}

/// Array.prototype.fill ( value[, start[, end]] )
///
/// The method fills (modifies) all the elements of an array from start index (default 0)
/// to an end index (default array length) with a static value. It returns the modified array
/// <https://tc39.es/ecma262/#sec-array.prototype.fill>
pub fn fill(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let len: i32 = from_value(this.get_field_slice("length")).expect("Could not get argument");
    let default_value = undefined();
    let value = args.get(0).unwrap_or(&default_value);
    let relative_start = args.get(1).unwrap_or(&default_value).to_num() as i32;
    let relative_end_val = args.get(2).unwrap_or(&default_value);
    let relative_end = if relative_end_val.is_undefined() {
        len
    } else {
        relative_end_val.to_num() as i32
    };
    let start = if relative_start < 0 {
        max(len + relative_start, 0)
    } else {
        min(relative_start, len)
    };
    let fin = if relative_end < 0 {
        max(len + relative_end, 0)
    } else {
        min(relative_end, len)
    };

    for i in start..fin {
        this.set_field(i.to_string(), value.clone());
    }

    Ok(this.clone())
}

pub fn includes_value(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let search_element = args
        .get(0)
        .cloned()
        .unwrap_or_else(|| Gc::new(ValueData::Undefined));

    let length: i32 =
        from_value(this.get_field_slice("length")).expect("Could not get `length` property.");

    for idx in 0..length {
        let check_element = this.get_field_slice(&idx.to_string()).clone();

        if check_element == search_element {
            return Ok(to_value(true));
        }
    }

    Ok(to_value(false))
}

/// Array.prototype.slice ( [begin[, end]] )
///
/// The slice method takes two arguments, start and end, and returns an array containing the
/// elements of the array from element start up to, but not including, element end (or through the
/// end of the array if end is undefined). If start is negative, it is treated as length + start
/// where length is the length of the array. If end is negative, it is treated as length + end where
/// length is the length of the array.
/// <https://tc39.es/ecma262/#sec-array.prototype.slice>
pub fn slice(this: &Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    let new_array = new_array(interpreter)?;
    let len: i32 =
        from_value(this.get_field_slice("length")).expect("Could not convert argument to i32");

    let start = match args.get(0) {
        Some(v) => from_value(v.clone()).expect("failed to parse argument for Array method"),
        None => 0,
    };
    let end = match args.get(1) {
        Some(v) => from_value(v.clone()).expect("failed to parse argument for Array method"),
        None => len,
    };

    let from = if start < 0 {
        max(len.wrapping_add(start), 0)
    } else {
        min(start, len)
    };
    let to = if end < 0 {
        max(len.wrapping_add(end), 0)
    } else {
        min(end, len)
    };

    let span = max(to.wrapping_sub(from), 0);
    let mut new_array_len: i32 = 0;
    for i in from..from.wrapping_add(span) {
        new_array.set_field(new_array_len.to_string(), this.get_field(&i.to_string()));
        new_array_len = new_array_len.wrapping_add(1);
    }
    new_array.set_field_slice("length", to_value(new_array_len));
    Ok(new_array)
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
    let index_of_func = to_value(index_of as NativeFunctionData);
    index_of_func.set_field_slice("length", to_value(1_i32));
    let last_index_of_func = to_value(last_index_of as NativeFunctionData);
    last_index_of_func.set_field_slice("length", to_value(1_i32));
    let includes_func = to_value(includes_value as NativeFunctionData);
    includes_func.set_field_slice("length", to_value(1_i32));
    let map_func = to_value(map as NativeFunctionData);
    map_func.set_field_slice("length", to_value(1_i32));
    let fill_func = to_value(fill as NativeFunctionData);
    fill_func.set_field_slice("length", to_value(1_i32));

    array_prototype.set_field_slice("push", push_func);
    array_prototype.set_field_slice("pop", to_value(pop as NativeFunctionData));
    array_prototype.set_field_slice("join", to_value(join as NativeFunctionData));
    array_prototype.set_field_slice("reverse", to_value(reverse as NativeFunctionData));
    array_prototype.set_field_slice("shift", to_value(shift as NativeFunctionData));
    array_prototype.set_field_slice("unshift", to_value(unshift as NativeFunctionData));
    array_prototype.set_field_slice("every", to_value(every as NativeFunctionData));
    array_prototype.set_field_slice("find", to_value(find as NativeFunctionData));
    array_prototype.set_field_slice("findIndex", to_value(find_index as NativeFunctionData));
    array_prototype.set_field_slice("includes", includes_func);
    array_prototype.set_field_slice("indexOf", index_of_func);
    array_prototype.set_field_slice("lastIndexOf", last_index_of_func);
    array_prototype.set_field_slice("fill", fill_func);
    array_prototype.set_field_slice("slice", to_value(slice as NativeFunctionData));
    array_prototype.set_field_slice("map", map_func);

    let array = to_value(array_constructor);
    array.set_field_slice(PROTOTYPE, to_value(array_prototype.clone()));

    array_prototype.set_field_slice("constructor", array.clone());
    array
}

#[cfg(test)]
mod tests {
    use crate::exec::Executor;
    use crate::forward;
    use crate::realm::Realm;

    #[test]
    fn concat() {
        //TODO: array display formatter
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var empty = new Array();
        var one = new Array(1);
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
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
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

    #[test]
    fn every() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        // taken from https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/every
        let init = r#"
        var empty = [];

        var array = [11, 23, 45];
        function callback(element) {
            return element > 10;
        }
        function callback2(element) {
            return element < 10;
        }

        var appendArray = [1,2,3,4];
        function appendingCallback(elem,index,arr) {
          arr.push('new');
          return elem !== "new";
        }

        var delArray = [1,2,3,4];
        function deletingCallback(elem,index,arr) {
          arr.pop()
          return elem < 3;
        }
        "#;
        forward(&mut engine, init);
        let result = forward(&mut engine, "array.every(callback);");
        assert_eq!(result, "true");

        let result = forward(&mut engine, "empty.every(callback);");
        assert_eq!(result, "true");

        let result = forward(&mut engine, "array.every(callback2);");
        assert_eq!(result, "false");

        let result = forward(&mut engine, "appendArray.every(appendingCallback);");
        assert_eq!(result, "true");

        let result = forward(&mut engine, "delArray.every(deletingCallback);");
        assert_eq!(result, "true");
    }

    #[test]
    fn find() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        function comp(a) {
            return a == "a";
        }
        var many = ["a", "b", "c"];
        "#;
        forward(&mut engine, init);
        let found = forward(&mut engine, "many.find(comp)");
        assert_eq!(found, String::from("a"));
    }

    #[test]
    fn find_index() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);

        let code = r#"
        function comp(item) {
            return item == 2;
        }
        var many = [1, 2, 3];
        var empty = [];
        var missing = [4, 5, 6];
        "#;

        forward(&mut engine, code);

        let many = forward(&mut engine, "many.findIndex(comp)");
        assert_eq!(many, String::from("1"));

        let empty = forward(&mut engine, "empty.findIndex(comp)");
        assert_eq!(empty, String::from("-1"));

        let missing = forward(&mut engine, "missing.findIndex(comp)");
        assert_eq!(missing, String::from("-1"));
    }

    #[test]
    fn push() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var arr = [1, 2];
        "#;
        forward(&mut engine, init);

        assert_eq!(forward(&mut engine, "arr.push()"), "2");
        assert_eq!(forward(&mut engine, "arr.push(3, 4)"), "4");
        assert_eq!(forward(&mut engine, "arr[2]"), "3");
        assert_eq!(forward(&mut engine, "arr[3]"), "4");
    }

    #[test]
    fn pop() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var empty = [ ];
        var one = [1];
        var many = [1, 2, 3, 4];
        "#;
        forward(&mut engine, init);

        assert_eq!(
            forward(&mut engine, "empty.pop()"),
            String::from("undefined")
        );
        assert_eq!(forward(&mut engine, "one.pop()"), "1");
        assert_eq!(forward(&mut engine, "one.length"), "0");
        assert_eq!(forward(&mut engine, "many.pop()"), "4");
        assert_eq!(forward(&mut engine, "many[0]"), "1");
        assert_eq!(forward(&mut engine, "many.length"), "3");
    }

    #[test]
    fn shift() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var empty = [ ];
        var one = [1];
        var many = [1, 2, 3, 4];
        "#;
        forward(&mut engine, init);

        assert_eq!(
            forward(&mut engine, "empty.shift()"),
            String::from("undefined")
        );
        assert_eq!(forward(&mut engine, "one.shift()"), "1");
        assert_eq!(forward(&mut engine, "one.length"), "0");
        assert_eq!(forward(&mut engine, "many.shift()"), "1");
        assert_eq!(forward(&mut engine, "many[0]"), "2");
        assert_eq!(forward(&mut engine, "many.length"), "3");
    }

    #[test]
    fn unshift() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var arr = [3, 4];
        "#;
        forward(&mut engine, init);

        assert_eq!(forward(&mut engine, "arr.unshift()"), "2");
        assert_eq!(forward(&mut engine, "arr.unshift(1, 2)"), "4");
        assert_eq!(forward(&mut engine, "arr[0]"), "1");
        assert_eq!(forward(&mut engine, "arr[1]"), "2");
    }

    #[test]
    fn reverse() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var arr = [1, 2];
        var reversed = arr.reverse();
        "#;
        forward(&mut engine, init);
        assert_eq!(forward(&mut engine, "reversed[0]"), "2");
        assert_eq!(forward(&mut engine, "reversed[1]"), "1");
        assert_eq!(forward(&mut engine, "arr[0]"), "2");
        assert_eq!(forward(&mut engine, "arr[1]"), "1");
    }

    #[test]
    fn index_of() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        var duplicates = ["a", "b", "c", "a", "b"];
        "#;
        forward(&mut engine, init);

        // Empty
        let empty = forward(&mut engine, "empty.indexOf('a')");
        assert_eq!(empty, String::from("-1"));

        // One
        let one = forward(&mut engine, "one.indexOf('a')");
        assert_eq!(one, String::from("0"));
        // Missing from one
        let missing_from_one = forward(&mut engine, "one.indexOf('b')");
        assert_eq!(missing_from_one, String::from("-1"));

        // First in many
        let first_in_many = forward(&mut engine, "many.indexOf('a')");
        assert_eq!(first_in_many, String::from("0"));
        // Second in many
        let second_in_many = forward(&mut engine, "many.indexOf('b')");
        assert_eq!(second_in_many, String::from("1"));

        // First in duplicates
        let first_in_many = forward(&mut engine, "duplicates.indexOf('a')");
        assert_eq!(first_in_many, String::from("0"));
        // Second in duplicates
        let second_in_many = forward(&mut engine, "duplicates.indexOf('b')");
        assert_eq!(second_in_many, String::from("1"));

        // Positive fromIndex greater than array length
        let fromindex_greater_than_length = forward(&mut engine, "one.indexOf('a', 2)");
        assert_eq!(fromindex_greater_than_length, String::from("-1"));
        // Positive fromIndex missed match
        let fromindex_misses_match = forward(&mut engine, "many.indexOf('a', 1)");
        assert_eq!(fromindex_misses_match, String::from("-1"));
        // Positive fromIndex matched
        let fromindex_matches = forward(&mut engine, "many.indexOf('b', 1)");
        assert_eq!(fromindex_matches, String::from("1"));
        // Positive fromIndex with duplicates
        let first_in_many = forward(&mut engine, "duplicates.indexOf('a', 1)");
        assert_eq!(first_in_many, String::from("3"));

        // Negative fromIndex greater than array length
        let fromindex_greater_than_length = forward(&mut engine, "one.indexOf('a', -2)");
        assert_eq!(fromindex_greater_than_length, String::from("0"));
        // Negative fromIndex missed match
        let fromindex_misses_match = forward(&mut engine, "many.indexOf('b', -1)");
        assert_eq!(fromindex_misses_match, String::from("-1"));
        // Negative fromIndex matched
        let fromindex_matches = forward(&mut engine, "many.indexOf('c', -1)");
        assert_eq!(fromindex_matches, String::from("2"));
        // Negative fromIndex with duplicates
        let second_in_many = forward(&mut engine, "duplicates.indexOf('b', -2)");
        assert_eq!(second_in_many, String::from("4"));
    }

    #[test]
    fn last_index_of() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        var duplicates = ["a", "b", "c", "a", "b"];
        "#;
        forward(&mut engine, init);

        // Empty
        let empty = forward(&mut engine, "empty.lastIndexOf('a')");
        assert_eq!(empty, String::from("-1"));

        // One
        let one = forward(&mut engine, "one.lastIndexOf('a')");
        assert_eq!(one, String::from("0"));
        // Missing from one
        let missing_from_one = forward(&mut engine, "one.lastIndexOf('b')");
        assert_eq!(missing_from_one, String::from("-1"));

        // First in many
        let first_in_many = forward(&mut engine, "many.lastIndexOf('a')");
        assert_eq!(first_in_many, String::from("0"));
        // Second in many
        let second_in_many = forward(&mut engine, "many.lastIndexOf('b')");
        assert_eq!(second_in_many, String::from("1"));

        // 4th in duplicates
        let first_in_many = forward(&mut engine, "duplicates.lastIndexOf('a')");
        assert_eq!(first_in_many, String::from("3"));
        // 5th in duplicates
        let second_in_many = forward(&mut engine, "duplicates.lastIndexOf('b')");
        assert_eq!(second_in_many, String::from("4"));

        // Positive fromIndex greater than array length
        let fromindex_greater_than_length = forward(&mut engine, "one.lastIndexOf('a', 2)");
        assert_eq!(fromindex_greater_than_length, String::from("0"));
        // Positive fromIndex missed match
        let fromindex_misses_match = forward(&mut engine, "many.lastIndexOf('c', 1)");
        assert_eq!(fromindex_misses_match, String::from("-1"));
        // Positive fromIndex matched
        let fromindex_matches = forward(&mut engine, "many.lastIndexOf('b', 1)");
        assert_eq!(fromindex_matches, String::from("1"));
        // Positive fromIndex with duplicates
        let first_in_many = forward(&mut engine, "duplicates.lastIndexOf('a', 1)");
        assert_eq!(first_in_many, String::from("0"));

        // Negative fromIndex greater than array length
        let fromindex_greater_than_length = forward(&mut engine, "one.lastIndexOf('a', -2)");
        assert_eq!(fromindex_greater_than_length, String::from("-1"));
        // Negative fromIndex missed match
        let fromindex_misses_match = forward(&mut engine, "many.lastIndexOf('c', -2)");
        assert_eq!(fromindex_misses_match, String::from("-1"));
        // Negative fromIndex matched
        let fromindex_matches = forward(&mut engine, "many.lastIndexOf('c', -1)");
        assert_eq!(fromindex_matches, String::from("2"));
        // Negative fromIndex with duplicates
        let second_in_many = forward(&mut engine, "duplicates.lastIndexOf('b', -2)");
        assert_eq!(second_in_many, String::from("1"));
    }

    #[test]
    fn fill() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);

        forward(&mut engine, "var a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4).join()"),
            String::from("4,4,4")
        );
        // make sure the array is modified
        assert_eq!(forward(&mut engine, "a.join()"), String::from("4,4,4"));

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, '1').join()"),
            String::from("1,4,4")
        );

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, 1, 2).join()"),
            String::from("1,4,3")
        );

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, 1, 1).join()"),
            String::from("1,2,3")
        );

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, 3, 3).join()"),
            String::from("1,2,3")
        );

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, -3, -2).join()"),
            String::from("4,2,3")
        );

        // TODO: uncomment when NaN support is added
        // forward(&mut engine, "a = [1, 2, 3];");
        // assert_eq!(
        //     forward(&mut engine, "a.fill(4, NaN, NaN).join()"),
        //     String::from("1,2,3")
        // );

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, 3, 5).join()"),
            String::from("1,2,3")
        );

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, '1.2', '2.5').join()"),
            String::from("1,4,3")
        );

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, 'str').join()"),
            String::from("4,4,4")
        );

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, 'str', 'str').join()"),
            String::from("1,2,3")
        );

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, undefined, null).join()"),
            String::from("1,2,3")
        );

        forward(&mut engine, "a = [1, 2, 3];");
        assert_eq!(
            forward(&mut engine, "a.fill(4, undefined, undefined).join()"),
            String::from("4,4,4")
        );

        assert_eq!(
            forward(&mut engine, "a.fill().join()"),
            String::from("undefined,undefined,undefined")
        );

        // test object reference
        forward(&mut engine, "a = (new Array(3)).fill({});");
        forward(&mut engine, "a[0].hi = 'hi';");
        assert_eq!(forward(&mut engine, "a[1].hi"), String::from("hi"));
    }

    #[test]
    fn inclues_value() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var empty = [ ];
        var one = ["a"];
        var many = ["a", "b", "c"];
        var duplicates = ["a", "b", "c", "a", "b"];
        var undefined = [undefined];
        "#;
        forward(&mut engine, init);

        // Empty
        let empty = forward(&mut engine, "empty.includes('a')");
        assert_eq!(empty, String::from("false"));

        // One
        let one = forward(&mut engine, "one.includes('a')");
        assert_eq!(one, String::from("true"));
        // Missing from one
        let missing_from_one = forward(&mut engine, "one.includes('b')");
        assert_eq!(missing_from_one, String::from("false"));

        // In many
        let first_in_many = forward(&mut engine, "many.includes('c')");
        assert_eq!(first_in_many, String::from("true"));
        // Missing from many
        let second_in_many = forward(&mut engine, "many.includes('d')");
        assert_eq!(second_in_many, String::from("false"));

        // In duplicates
        let first_in_many = forward(&mut engine, "duplicates.includes('a')");
        assert_eq!(first_in_many, String::from("true"));
        // Missing from duplicates
        let second_in_many = forward(&mut engine, "duplicates.includes('d')");
        assert_eq!(second_in_many, String::from("false"));
    }

    #[test]
    fn map() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);

        let js = r#"
        var empty = [];
        var one = ["x"];
        var many = ["x", "y", "z"];

        // TODO: uncomment when `this` has been implemented
        // var _this = { answer: 42 };

        // function callbackThatUsesThis() {
        //      return 'The answer to life is: ' + this.answer;
        // }

        var empty_mapped = empty.map(v => v + '_');
        var one_mapped = one.map(v => '_' + v);
        var many_mapped = many.map(v => '_' + v + '_');
        "#;

        forward(&mut engine, js);

        // assert the old arrays have not been modified
        assert_eq!(forward(&mut engine, "one[0]"), String::from("x"));
        assert_eq!(
            forward(&mut engine, "many[2] + many[1] + many[0]"),
            String::from("zyx")
        );

        // NB: These tests need to be rewritten once `Display` has been implemented for `Array`
        // Empty
        assert_eq!(
            forward(&mut engine, "empty_mapped.length"),
            String::from("0")
        );

        // One
        assert_eq!(forward(&mut engine, "one_mapped.length"), String::from("1"));
        assert_eq!(forward(&mut engine, "one_mapped[0]"), String::from("_x"));

        // Many
        assert_eq!(
            forward(&mut engine, "many_mapped.length"),
            String::from("3")
        );
        assert_eq!(
            forward(
                &mut engine,
                "many_mapped[0] + many_mapped[1] + many_mapped[2]"
            ),
            String::from("_x__y__z_")
        );

        // TODO: uncomment when `this` has been implemented
        // One but it uses `this` inside the callback
        // let one_with_this = forward(&mut engine, "one.map(callbackThatUsesThis, _this)[0];");
        // assert_eq!(one_with_this, String::from("The answer to life is: 42"))
    }

    #[test]
    fn slice() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var empty = [ ].slice();
        var one = ["a"].slice();
        var many1 = ["a", "b", "c", "d"].slice(1);
        var many2 = ["a", "b", "c", "d"].slice(2, 3);
        var many3 = ["a", "b", "c", "d"].slice(7);
        "#;
        forward(&mut engine, init);

        assert_eq!(forward(&mut engine, "empty.length"), "0");
        assert_eq!(forward(&mut engine, "one[0]"), "a");
        assert_eq!(forward(&mut engine, "many1[0]"), "b");
        assert_eq!(forward(&mut engine, "many1[1]"), "c");
        assert_eq!(forward(&mut engine, "many1[2]"), "d");
        assert_eq!(forward(&mut engine, "many1.length"), "3");
        assert_eq!(forward(&mut engine, "many2[0]"), "c");
        assert_eq!(forward(&mut engine, "many2.length"), "1");
        assert_eq!(forward(&mut engine, "many3.length"), "0");
    }
}
