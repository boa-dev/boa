//! This module implements the global `Array` object.
//!
//! The JavaScript `Array` class is a global object that is used in the construction of arrays; which are high-level, list-like objects.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-array-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array

#[cfg(test)]
mod tests;

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

/// Creates a new `Array` instance.
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

/// Utility function for creating array objects.
///
/// `array_obj` can be any array with prototype already set (it will be wiped and
/// recreated from `array_contents`)
pub fn construct_array(array_obj: &Value, array_contents: &[Value]) -> ResultValue {
    let array_obj_ptr = array_obj.clone();

    // Wipe existing contents of the array object
    let orig_length: i32 =
        from_value(array_obj.get_field_slice("length")).expect("failed to convert length to i32");
    for n in 0..orig_length {
        array_obj_ptr.remove_prop(&n.to_string());
    }

    // Create length
    let length = Property::new()
        .value(to_value(array_contents.len() as i32))
        .writable(true)
        .configurable(false)
        .enumerable(false);

    array_obj_ptr.set_prop("length".to_string(), length);

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
pub fn make_array(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
    // Make a new Object which will internally represent the Array (mapping
    // between indices and values): this creates an Object with no prototype

    // Create length
    let length = Property::new()
        .value(to_value(args.len() as i32))
        .writable(true)
        .configurable(false)
        .enumerable(false);

    this.set_prop("length".to_string(), length);

    // Set Prototype
    let array_prototype = ctx
        .realm
        .global_obj
        .get_field_slice("Array")
        .get_field_slice(PROTOTYPE);

    this.set_internal_slot(INSTANCE_PROTOTYPE, array_prototype);
    // This value is used by console.log and other routines to match Object type
    // to its Javascript Identifier (global constructor method name)
    this.set_kind(ObjectKind::Array);

    // And finally add our arguments in
    for (n, value) in args.iter().enumerate() {
        this.set_field_slice(&n.to_string(), value.clone());
    }

    Ok(this.clone())
}

/// `Array.isArray( arg )`
///
/// The isArray function takes one argument arg, and returns the Boolean value true
/// if the argument is an object whose class internal property is "Array"; otherwise it returns false.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.isarray
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/isArray
pub fn is_array(_this: &Value, args: &[Value], _interpreter: &mut Interpreter) -> ResultValue {
    let value_true = Gc::new(ValueData::Boolean(true));
    let value_false = Gc::new(ValueData::Boolean(false));

    match args.get(0) {
        Some(arg) => {
            match *(*arg).clone() {
                // 1.
                ValueData::Object(ref obj) => {
                    // 2.
                    if obj.borrow().kind == ObjectKind::Array {
                        return Ok(value_true);
                    }
                    Ok(value_false)
                }
                // 3.
                _ => Ok(value_false),
            }
        }
        None => Ok(value_false),
    }
}

/// `Array.prototype.concat(...arguments)`
///
/// When the concat method is called with zero or more arguments, it returns an
/// array containing the array elements of the object followed by the array
/// elements of each argument in order.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.concat
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/concat
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

/// `Array.prototype.push( ...items )`
///
/// The arguments are appended to the end of the array, in the order in which
/// they appear. The new length of the array is returned as the result of the
/// call.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.push
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/push
pub fn push(this: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let new_array = add_to_array_object(this, args)?;
    Ok(new_array.get_field_slice("length"))
}

/// `Array.prototype.pop()`
///
/// The last element of the array is removed from the array and returned.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.pop
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/pop
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

/// `Array.prototype.forEach( callbackFn [ , thisArg ] )`
///
/// This method executes the provided callback function for each element in the array.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.foreach
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/forEach
pub fn for_each(this: &Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        return Err(to_value(
            "Missing argument for Array.prototype.forEach".to_string(),
        ));
    }

    let callback_arg = args.get(0).expect("Could not get `callbackFn` argument.");
    let this_arg = args.get(1).cloned().unwrap_or_else(undefined);

    let length: i32 =
        from_value(this.get_field_slice("length")).expect("Could not get `length` property.");

    for i in 0..length {
        let element = this.get_field_slice(&i.to_string());
        let arguments = vec![element.clone(), to_value(i), this.clone()];

        interpreter.call(callback_arg, &this_arg, arguments)?;
    }

    Ok(Gc::new(ValueData::Undefined))
}

/// `Array.prototype.join( separator )`
///
/// The elements of the array are converted to Strings, and these Strings are
/// then concatenated, separated by occurrences of the separator. If no
/// separator is provided, a single comma is used as the separator.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.join
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/join
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

/// `Array.prototype.toString( separator )`
///
/// The toString function is intentionally generic; it does not require that
/// its this value be an Array object. Therefore it can be transferred to
/// other kinds of objects for use as a method.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.tostring
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/toString
pub fn to_string(this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let method_name = "join";
    let mut arguments = vec![to_value(",")];
    // 2.
    let mut method: Value =
        from_value(this.get_field_slice(method_name)).expect("failed to get Array.prototype.join");
    // 3.
    if !method.is_function() {
        method = _ctx
            .realm
            .global_obj
            .get_field_slice("Object")
            .get_field_slice(PROTOTYPE)
            .get_field_slice("toString");

        method = from_value(method).expect("failed to get Object.prototype.toString");
        arguments = vec![];
    }
    // 4.
    let join_result = _ctx.call(&method, this, arguments);
    let match_string = match join_result {
        Ok(v) => match *v {
            ValueData::String(ref s) => (*s).clone(),
            _ => "".to_string(),
        },
        Err(v) => format!("error: {}", v),
    };
    Ok(to_value(match_string))
}

/// `Array.prototype.reverse()`
///
/// The elements of the array are rearranged so as to reverse their order.
/// The object is returned as the result of the call.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.reverse
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/reverse
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

/// `Array.prototype.shift()`
///
/// The first element of the array is removed from the array and returned.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.shift
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/shift
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

/// `Array.prototype.unshift( ...items )`
///
/// The arguments are prepended to the start of the array, such that their order
/// within the array is the same as the order in which they appear in the
/// argument list.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.unshift
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/unshift
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

/// `Array.prototype.every( callback, [ thisArg ] )`
///
/// The every method executes the provided callback function once for each
/// element present in the array until it finds the one where callback returns
/// a falsy value. It returns `false` if it finds such element, otherwise it
/// returns `true`.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.every
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/every
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

/// `Array.prototype.map( callback, [ thisArg ] )`
///
/// For each element in the array the callback function is called, and a new
/// array is constructed from the return values of these calls.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.map
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/map
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
            let element = this.get_field_slice(&idx.to_string());

            let args = vec![element, to_value(idx), new.clone()];

            interpreter
                .call(&callback, &this_val, args)
                .unwrap_or_else(|_| undefined())
        })
        .collect::<Vec<Value>>();

    construct_array(&new, &values)
}

/// `Array.prototype.indexOf( searchElement[, fromIndex ] )`
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
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.indexof
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/indexOf
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

/// `Array.prototype.lastIndexOf( searchElement[, fromIndex ] )`
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
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.lastindexof
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/lastIndexOf
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

/// `Array.prototype.find( callback, [thisArg] )`
///
/// The find method executes the callback function once for each index of the array
/// until the callback returns a truthy value. If so, find immediately returns the value
/// of that element. Otherwise, find returns undefined.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.find
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/find
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

/// `Array.prototype.findIndex( predicate [ , thisArg ] )`
///
/// This method executes the provided predicate function for each element of the array.
/// If the predicate function returns `true` for an element, this method returns the index of the element.
/// If all elements return `false`, the value `-1` is returned.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.findindex
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/findIndex
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
            return Ok(Gc::new(ValueData::Rational(f64::from(i))));
        }
    }

    Ok(Gc::new(ValueData::Rational(f64::from(-1))))
}

/// `Array.prototype.fill( value[, start[, end]] )`
///
/// The method fills (modifies) all the elements of an array from start index (default 0)
/// to an end index (default array length) with a static value. It returns the modified array.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.fill
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/fill
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
        this.set_field_slice(&i.to_string(), value.clone());
    }

    Ok(this.clone())
}

/// `Array.prototype.includes( valueToFind [, fromIndex] )`
///
/// Determines whether an array includes a certain value among its entries, returning `true` or `false` as appropriate.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.includes
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/includes
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

/// `Array.prototype.slice( [begin[, end]] )`
///
/// The slice method takes two arguments, start and end, and returns an array containing the
/// elements of the array from element start up to, but not including, element end (or through the
/// end of the array if end is undefined). If start is negative, it is treated as length + start
/// where length is the length of the array. If end is negative, it is treated as length + end where
/// length is the length of the array.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.slice
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/slice
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
        new_array.set_field_slice(
            &new_array_len.to_string(),
            this.get_field_slice(&i.to_string()),
        );
        new_array_len = new_array_len.wrapping_add(1);
    }
    new_array.set_field_slice("length", to_value(new_array_len));
    Ok(new_array)
}

/// `Array.prototype.filter( callback, [ thisArg ] )`
///
/// For each element in the array the callback function is called, and a new
/// array is constructed for every value whose callback returned a truthy value.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.filter
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/filter
pub fn filter(this: &Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        return Err(to_value(
            "missing argument 0 when calling function Array.prototype.filter",
        ));
    }

    let callback = args.get(0).cloned().unwrap_or_else(undefined);
    let this_val = args.get(1).cloned().unwrap_or_else(undefined);

    let length: i32 =
        from_value(this.get_field_slice("length")).expect("Could not get `length` property.");

    let new = new_array(&interpreter)?;

    let values = (0..length)
        .filter_map(|idx| {
            let element = this.get_field_slice(&idx.to_string());

            let args = vec![element.clone(), to_value(idx), new.clone()];

            let callback_result = interpreter
                .call(&callback, &this_val, args)
                .unwrap_or_else(|_| undefined());

            if callback_result.is_true() {
                Some(element)
            } else {
                None
            }
        })
        .collect::<Vec<Value>>();

    construct_array(&new, &values)
}

/// Array.prototype.some ( callbackfn [ , thisArg ] )
///
/// The some method tests whether at least one element in the array passes
/// the test implemented by the provided callback function. It returns a Boolean value,
/// true if the callback function returns a truthy value for at least one element
/// in the array. Otherwise, false.
///
/// Caution: Calling this method on an empty array returns false for any condition!
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-array.prototype.some
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/some
pub fn some(this: &Value, args: &[Value], interpreter: &mut Interpreter) -> ResultValue {
    if args.is_empty() {
        return Err(to_value(
            "missing callback when calling function Array.prototype.some".to_string(),
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
        if result {
            return Ok(to_value(true));
        }
        // the length of the array must be updated because the callback can mutate it.
        len = min(max_len, from_value(this.get_field_slice("length")).unwrap());
        i += 1;
    }
    Ok(to_value(false))
}

/// Create a new `Array` object.
pub fn create_constructor(global: &Value) -> Value {
    // Create Constructor
    let object_prototype = global.get_field_slice("Object").get_field_slice(PROTOTYPE);
    let mut array_constructor = Object::create(object_prototype);
    array_constructor.kind = ObjectKind::Function;
    array_constructor.set_internal_method("construct", make_array);
    // Todo: add call function
    array_constructor.set_internal_method("call", make_array);

    // Create prototype
    let array_prototype = ValueData::new_obj(None);
    let length = Property::default().value(to_value(0_i32));
    array_prototype.set_prop_slice("length", length);

    make_builtin_fn!(concat, named "concat", with length 1, of array_prototype);
    make_builtin_fn!(push, named "push", with length 1, of array_prototype);
    make_builtin_fn!(index_of, named "indexOf", with length 1, of array_prototype);
    make_builtin_fn!(last_index_of, named "lastIndexOf", with length 1, of array_prototype);
    make_builtin_fn!(includes_value, named "includes", with length 1, of array_prototype);
    make_builtin_fn!(map, named "map", with length 1, of array_prototype);
    make_builtin_fn!(fill, named "fill", with length 1, of array_prototype);
    make_builtin_fn!(for_each, named "forEach", with length 1, of array_prototype);
    make_builtin_fn!(filter, named "filter", with length 1, of array_prototype);
    make_builtin_fn!(pop, named "pop", of array_prototype);
    make_builtin_fn!(join, named "join", with length 1, of array_prototype);
    make_builtin_fn!(to_string, named "toString", of array_prototype);
    make_builtin_fn!(reverse, named "reverse", of array_prototype);
    make_builtin_fn!(shift, named "shift", of array_prototype);
    make_builtin_fn!(unshift, named "unshift", with length 1, of array_prototype);
    make_builtin_fn!(every, named "every", with length 1, of array_prototype);
    make_builtin_fn!(find, named "find", with length 1, of array_prototype);
    make_builtin_fn!(find_index, named "findIndex", with length 1, of array_prototype);
    make_builtin_fn!(slice, named "slice", with length 2, of array_prototype);
    make_builtin_fn!(some, named "some", with length 2, of array_prototype);

    let array = to_value(array_constructor);
    make_builtin_fn!(is_array, named "isArray", with length 1, of array);
    array.set_field_slice(PROTOTYPE, to_value(array_prototype.clone()));

    array_prototype.set_field_slice("constructor", array.clone());
    array
}
