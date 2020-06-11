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

use super::function::{make_builtin_fn, make_constructor_fn};
use crate::{
    builtins::{
        object::{ObjectData, INSTANCE_PROTOTYPE, PROTOTYPE},
        property::Property,
        value::{same_value_zero, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
    BoaProfiler,
};
use std::{
    borrow::Borrow,
    cmp::{max, min},
    ops::Deref,
};

/// JavaScript `Array` built-in implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Array;

impl Array {
    /// Creates a new `Array` instance.
    pub(crate) fn new_array(interpreter: &Interpreter) -> ResultValue {
        let array = Value::new_object(Some(
            &interpreter
                .realm()
                .environment
                .get_global_object()
                .expect("Could not get global object"),
        ));
        array.set_data(ObjectData::Array);
        array.borrow().set_internal_slot(
            INSTANCE_PROTOTYPE,
            interpreter
                .realm()
                .environment
                .get_binding_value("Array")
                .borrow()
                .get_field(PROTOTYPE),
        );
        array.borrow().set_field("length", Value::from(0));
        Ok(array)
    }

    /// Utility function for creating array objects.
    ///
    /// `array_obj` can be any array with prototype already set (it will be wiped and
    /// recreated from `array_contents`)
    pub(crate) fn construct_array(array_obj: &Value, array_contents: &[Value]) -> ResultValue {
        let array_obj_ptr = array_obj.clone();

        // Wipe existing contents of the array object
        let orig_length = i32::from(&array_obj.get_field("length"));
        for n in 0..orig_length {
            array_obj_ptr.remove_property(&n.to_string());
        }

        // Create length
        let length = Property::new()
            .value(Value::from(array_contents.len() as i32))
            .writable(true)
            .configurable(false)
            .enumerable(false);

        array_obj_ptr.set_property("length".to_string(), length);

        for (n, value) in array_contents.iter().enumerate() {
            array_obj_ptr.set_field(n.to_string(), value);
        }
        Ok(array_obj_ptr)
    }

    /// Utility function which takes an existing array object and puts additional
    /// values on the end, correctly rewriting the length
    pub(crate) fn add_to_array_object(array_ptr: &Value, add_values: &[Value]) -> ResultValue {
        let orig_length = i32::from(&array_ptr.get_field("length"));

        for (n, value) in add_values.iter().enumerate() {
            let new_index = orig_length.wrapping_add(n as i32);
            array_ptr.set_field(new_index.to_string(), value);
        }

        array_ptr.set_field(
            "length",
            Value::from(orig_length.wrapping_add(add_values.len() as i32)),
        );

        Ok(array_ptr.clone())
    }

    /// Create a new array
    pub(crate) fn make_array(
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        // Make a new Object which will internally represent the Array (mapping
        // between indices and values): this creates an Object with no prototype

        // Set Prototype
        let prototype = ctx.realm.global_obj.get_field("Array").get_field(PROTOTYPE);

        this.set_internal_slot(INSTANCE_PROTOTYPE, prototype);
        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        this.set_data(ObjectData::Array);

        // add our arguments in
        let mut length = args.len() as i32;
        match args.len() {
            1 if args[0].is_integer() => {
                length = i32::from(&args[0]);
                // TODO: It should not create an array of undefineds, but an empty array ("holy" array in V8) with length `n`.
                for n in 0..length {
                    this.set_field(n.to_string(), Value::undefined());
                }
            }
            1 if args[0].is_double() => {
                return ctx.throw_range_error("invalid array length");
            }
            _ => {
                for (n, value) in args.iter().enumerate() {
                    this.set_field(n.to_string(), value.clone());
                }
            }
        }

        // finally create length property
        let length = Property::new()
            .value(Value::from(length))
            .writable(true)
            .configurable(false)
            .enumerable(false);

        this.set_property("length".to_string(), length);

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
    pub(crate) fn is_array(
        _this: &mut Value,
        args: &[Value],
        _interpreter: &mut Interpreter,
    ) -> ResultValue {
        let value_true = Value::boolean(true);
        let value_false = Value::boolean(false);

        match args.get(0) {
            Some(arg) => {
                match arg.data() {
                    // 1.
                    ValueData::Object(ref obj) => {
                        // 2.
                        if let ObjectData::Array = (*obj).deref().borrow().data {
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
    pub(crate) fn concat(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
        if args.is_empty() {
            // If concat is called with no arguments, it returns the original array
            return Ok(this.clone());
        }

        // Make a new array (using this object as the prototype basis for the new
        // one)
        let mut new_values: Vec<Value> = Vec::new();

        let this_length = i32::from(&this.get_field("length"));
        for n in 0..this_length {
            new_values.push(this.get_field(n.to_string()));
        }

        for concat_array in args {
            let concat_length = i32::from(&concat_array.get_field("length"));
            for n in 0..concat_length {
                new_values.push(concat_array.get_field(n.to_string()));
            }
        }

        Self::construct_array(this, &new_values)
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
    pub(crate) fn push(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
        let new_array = Self::add_to_array_object(this, args)?;
        Ok(new_array.get_field("length"))
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
    pub(crate) fn pop(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
        let curr_length = i32::from(&this.get_field("length"));
        if curr_length < 1 {
            return Ok(Value::undefined());
        }
        let pop_index = curr_length.wrapping_sub(1);
        let pop_value: Value = this.get_field(pop_index.to_string());
        this.remove_property(&pop_index.to_string());
        this.set_field("length", Value::from(pop_index));
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
    pub(crate) fn for_each(
        this: &mut Value,
        args: &[Value],
        interpreter: &mut Interpreter,
    ) -> ResultValue {
        if args.is_empty() {
            return Err(Value::from("Missing argument for Array.prototype.forEach"));
        }

        let callback_arg = args.get(0).expect("Could not get `callbackFn` argument.");
        let mut this_arg = args.get(1).cloned().unwrap_or_else(Value::undefined);

        let length = i32::from(&this.get_field("length"));

        for i in 0..length {
            let element = this.get_field(i.to_string());
            let arguments = [element, Value::from(i), this.clone()];

            interpreter.call(callback_arg, &mut this_arg, &arguments)?;
        }

        Ok(Value::undefined())
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
    pub(crate) fn join(this: &mut Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let separator = if args.is_empty() {
            String::from(",")
        } else {
            ctx.to_string(args.get(0).expect("Could not get argument"))?
        };

        let mut elem_strs: Vec<String> = Vec::new();
        let length = i32::from(&this.get_field("length"));
        for n in 0..length {
            let elem_str: String = ctx.to_string(&this.get_field(n.to_string()))?;
            elem_strs.push(elem_str);
        }

        Ok(Value::from(elem_strs.join(&separator)))
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
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(
        this: &mut Value,
        _args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        let method_name = "join";
        let mut arguments = vec![Value::from(",")];
        // 2.
        let mut method = this.get_field(method_name);
        // 3.
        if !method.is_function() {
            method = ctx
                .realm
                .global_obj
                .get_field("Object")
                .get_field(PROTOTYPE)
                .get_field("toString");

            arguments = Vec::new();
        }
        // 4.
        let join = ctx.call(&method, this, &arguments)?;

        let string = if let ValueData::String(ref s) = join.data() {
            Value::from(s.as_str())
        } else {
            Value::from("")
        };

        Ok(string)
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
    pub(crate) fn reverse(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
        let len = i32::from(&this.get_field("length"));
        let middle: i32 = len.wrapping_div(2);

        for lower in 0..middle {
            let upper = len.wrapping_sub(lower).wrapping_sub(1);

            let upper_exists = this.has_field(&upper.to_string());
            let lower_exists = this.has_field(&lower.to_string());

            let upper_value = this.get_field(upper.to_string());
            let lower_value = this.get_field(lower.to_string());

            if upper_exists && lower_exists {
                this.set_field(upper.to_string(), lower_value);
                this.set_field(lower.to_string(), upper_value);
            } else if upper_exists {
                this.set_field(lower.to_string(), upper_value);
                this.remove_property(&upper.to_string());
            } else if lower_exists {
                this.set_field(upper.to_string(), lower_value);
                this.remove_property(&lower.to_string());
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
    pub(crate) fn shift(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
        let len = i32::from(&this.get_field("length"));

        if len == 0 {
            this.set_field("length", Value::from(0));
            // Since length is 0, this will be an Undefined value
            return Ok(this.get_field(0.to_string()));
        }

        let first: Value = this.get_field(0.to_string());

        for k in 1..len {
            let from = k.to_string();
            let to = (k.wrapping_sub(1)).to_string();

            let from_value = this.get_field(from);
            if from_value.is_undefined() {
                this.remove_property(&to);
            } else {
                this.set_field(to, from_value);
            }
        }

        let final_index = len.wrapping_sub(1);
        this.remove_property(&(final_index).to_string());
        this.set_field("length", Value::from(final_index));

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
    pub(crate) fn unshift(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
        let len = i32::from(&this.get_field("length"));
        let arg_c: i32 = args.len() as i32;

        if arg_c > 0 {
            for k in (1..=len).rev() {
                let from = (k.wrapping_sub(1)).to_string();
                let to = (k.wrapping_add(arg_c).wrapping_sub(1)).to_string();

                let from_value = this.get_field(from);
                if from_value.is_undefined() {
                    this.remove_property(&to);
                } else {
                    this.set_field(to, from_value);
                }
            }
            for j in 0..arg_c {
                this.set_field(
                    j.to_string(),
                    args.get(j as usize)
                        .expect("Could not get argument")
                        .clone(),
                );
            }
        }

        let temp = len.wrapping_add(arg_c);
        this.set_field("length", Value::from(temp));
        Ok(Value::from(temp))
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
    pub(crate) fn every(
        this: &mut Value,
        args: &[Value],
        interpreter: &mut Interpreter,
    ) -> ResultValue {
        if args.is_empty() {
            return Err(Value::from(
                "missing callback when calling function Array.prototype.every",
            ));
        }
        let callback = &args[0];
        let mut this_arg = if args.len() > 1 {
            args[1].clone()
        } else {
            Value::undefined()
        };
        let mut i = 0;
        let max_len = i32::from(&this.get_field("length"));
        let mut len = max_len;
        while i < len {
            let element = this.get_field(i.to_string());
            let arguments = [element, Value::from(i), this.clone()];
            let result = interpreter
                .call(callback, &mut this_arg, &arguments)?
                .is_true();
            if !result {
                return Ok(Value::from(false));
            }
            len = min(max_len, i32::from(&this.get_field("length")));
            i += 1;
        }
        Ok(Value::from(true))
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
    pub(crate) fn map(
        this: &mut Value,
        args: &[Value],
        interpreter: &mut Interpreter,
    ) -> ResultValue {
        if args.is_empty() {
            return Err(Value::from(
                "missing argument 0 when calling function Array.prototype.map",
            ));
        }

        let callback = args.get(0).cloned().unwrap_or_else(Value::undefined);
        let mut this_val = args.get(1).cloned().unwrap_or_else(Value::undefined);

        let length = i32::from(&this.get_field("length"));

        let new = Self::new_array(interpreter)?;

        let values: Vec<Value> = (0..length)
            .map(|idx| {
                let element = this.get_field(idx.to_string());
                let args = [element, Value::from(idx), new.clone()];

                interpreter
                    .call(&callback, &mut this_val, &args)
                    .unwrap_or_else(|_| Value::undefined())
            })
            .collect();

        Self::construct_array(&new, &values)
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
    pub(crate) fn index_of(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
        // If no arguments, return -1. Not described in spec, but is what chrome does.
        if args.is_empty() {
            return Ok(Value::from(-1));
        }

        let search_element = args[0].clone();
        let len = i32::from(&this.get_field("length"));

        let mut idx = match args.get(1) {
            Some(from_idx_ptr) => {
                let from_idx = i32::from(from_idx_ptr);

                if from_idx < 0 {
                    len + from_idx
                } else {
                    from_idx
                }
            }
            None => 0,
        };

        while idx < len {
            let check_element = this.get_field(idx.to_string()).clone();

            if check_element.strict_equals(&search_element) {
                return Ok(Value::from(idx));
            }

            idx += 1;
        }

        Ok(Value::from(-1))
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
    pub(crate) fn last_index_of(
        this: &mut Value,
        args: &[Value],
        _: &mut Interpreter,
    ) -> ResultValue {
        // If no arguments, return -1. Not described in spec, but is what chrome does.
        if args.is_empty() {
            return Ok(Value::from(-1));
        }

        let search_element = args[0].clone();
        let len = i32::from(&this.get_field("length"));

        let mut idx = match args.get(1) {
            Some(from_idx_ptr) => {
                let from_idx = i32::from(from_idx_ptr);

                if from_idx >= 0 {
                    min(from_idx, len - 1)
                } else {
                    len + from_idx
                }
            }
            None => len - 1,
        };

        while idx >= 0 {
            let check_element = this.get_field(idx.to_string()).clone();

            if check_element.strict_equals(&search_element) {
                return Ok(Value::from(idx));
            }

            idx -= 1;
        }

        Ok(Value::from(-1))
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
    pub(crate) fn find(
        this: &mut Value,
        args: &[Value],
        interpreter: &mut Interpreter,
    ) -> ResultValue {
        if args.is_empty() {
            return Err(Value::from(
                "missing callback when calling function Array.prototype.find",
            ));
        }
        let callback = &args[0];
        let mut this_arg = if args.len() > 1 {
            args[1].clone()
        } else {
            Value::undefined()
        };
        let len = i32::from(&this.get_field("length"));
        for i in 0..len {
            let element = this.get_field(i.to_string());
            let arguments = [element.clone(), Value::from(i), this.clone()];
            let result = interpreter.call(callback, &mut this_arg, &arguments)?;
            if result.is_true() {
                return Ok(element);
            }
        }
        Ok(Value::undefined())
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
    pub(crate) fn find_index(
        this: &mut Value,
        args: &[Value],
        interpreter: &mut Interpreter,
    ) -> ResultValue {
        if args.is_empty() {
            return Err(Value::from(
                "Missing argument for Array.prototype.findIndex",
            ));
        }

        let predicate_arg = args.get(0).expect("Could not get `predicate` argument.");

        let mut this_arg = args.get(1).cloned().unwrap_or_else(Value::undefined);

        let length = i32::from(&this.get_field("length"));

        for i in 0..length {
            let element = this.get_field(i.to_string());
            let arguments = [element, Value::from(i), this.clone()];

            let result = interpreter.call(predicate_arg, &mut this_arg, &arguments)?;

            if result.is_true() {
                return Ok(Value::rational(f64::from(i)));
            }
        }

        Ok(Value::rational(-1_f64))
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
    pub(crate) fn fill(this: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
        let len: i32 = i32::from(&this.get_field("length"));
        let default_value = Value::undefined();
        let value = args.get(0).unwrap_or(&default_value);
        let relative_start = args.get(1).unwrap_or(&default_value).to_number() as i32;
        let relative_end_val = args.get(2).unwrap_or(&default_value);
        let relative_end = if relative_end_val.is_undefined() {
            len
        } else {
            relative_end_val.to_number() as i32
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
    pub(crate) fn includes_value(
        this: &mut Value,
        args: &[Value],
        _: &mut Interpreter,
    ) -> ResultValue {
        let search_element = args.get(0).cloned().unwrap_or_else(Value::undefined);

        let length = i32::from(&this.get_field("length"));

        for idx in 0..length {
            let check_element = this.get_field(idx.to_string()).clone();

            if same_value_zero(&check_element, &search_element) {
                return Ok(Value::from(true));
            }
        }

        Ok(Value::from(false))
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
    pub(crate) fn slice(
        this: &mut Value,
        args: &[Value],
        interpreter: &mut Interpreter,
    ) -> ResultValue {
        let new_array = Self::new_array(interpreter)?;
        let len = i32::from(&this.get_field("length"));

        let start = match args.get(0) {
            Some(v) => i32::from(v),
            None => 0,
        };
        let end = match args.get(1) {
            Some(v) => i32::from(v),
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
            new_array.set_field(new_array_len.to_string(), this.get_field(i.to_string()));
            new_array_len = new_array_len.wrapping_add(1);
        }
        new_array.set_field("length", Value::from(new_array_len));
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
    pub(crate) fn filter(
        this: &mut Value,
        args: &[Value],
        interpreter: &mut Interpreter,
    ) -> ResultValue {
        if args.is_empty() {
            return Err(Value::from(
                "missing argument 0 when calling function Array.prototype.filter",
            ));
        }

        let callback = args.get(0).cloned().unwrap_or_else(Value::undefined);
        let mut this_val = args.get(1).cloned().unwrap_or_else(Value::undefined);

        let length = i32::from(&this.get_field("length"));

        let new = Self::new_array(interpreter)?;

        let values = (0..length)
            .filter_map(|idx| {
                let element = this.get_field(idx.to_string());

                let args = [element.clone(), Value::from(idx), new.clone()];

                let callback_result = interpreter
                    .call(&callback, &mut this_val, &args)
                    .unwrap_or_else(|_| Value::undefined());

                if callback_result.is_true() {
                    Some(element)
                } else {
                    None
                }
            })
            .collect::<Vec<Value>>();

        Self::construct_array(&new, &values)
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
    pub(crate) fn some(
        this: &mut Value,
        args: &[Value],
        interpreter: &mut Interpreter,
    ) -> ResultValue {
        if args.is_empty() {
            return Err(Value::from(
                "missing callback when calling function Array.prototype.some",
            ));
        }
        let callback = &args[0];
        let mut this_arg = if args.len() > 1 {
            args[1].clone()
        } else {
            Value::undefined()
        };
        let mut i = 0;
        let max_len = i32::from(&this.get_field("length"));
        let mut len = max_len;
        while i < len {
            let element = this.get_field(i.to_string());
            let arguments = [element, Value::from(i), this.clone()];
            let result = interpreter
                .call(callback, &mut this_arg, &arguments)?
                .is_true();
            if result {
                return Ok(Value::from(true));
            }
            // the length of the array must be updated because the callback can mutate it.
            len = min(max_len, i32::from(&this.get_field("length")));
            i += 1;
        }
        Ok(Value::from(false))
    }

    /// Create a new `Array` object.
    pub(crate) fn create(global: &Value) -> Value {
        // Create prototype
        let prototype = Value::new_object(None);
        let length = Property::default().value(Value::from(0));

        prototype.set_property("length", length);

        make_builtin_fn(Self::concat, "concat", &prototype, 1);
        make_builtin_fn(Self::push, "push", &prototype, 1);
        make_builtin_fn(Self::index_of, "indexOf", &prototype, 1);
        make_builtin_fn(Self::last_index_of, "lastIndexOf", &prototype, 1);
        make_builtin_fn(Self::includes_value, "includes", &prototype, 1);
        make_builtin_fn(Self::map, "map", &prototype, 1);
        make_builtin_fn(Self::fill, "fill", &prototype, 1);
        make_builtin_fn(Self::for_each, "forEach", &prototype, 1);
        make_builtin_fn(Self::filter, "filter", &prototype, 1);
        make_builtin_fn(Self::pop, "pop", &prototype, 0);
        make_builtin_fn(Self::join, "join", &prototype, 1);
        make_builtin_fn(Self::to_string, "toString", &prototype, 0);
        make_builtin_fn(Self::reverse, "reverse", &prototype, 0);
        make_builtin_fn(Self::shift, "shift", &prototype, 0);
        make_builtin_fn(Self::unshift, "unshift", &prototype, 1);
        make_builtin_fn(Self::every, "every", &prototype, 1);
        make_builtin_fn(Self::find, "find", &prototype, 1);
        make_builtin_fn(Self::find_index, "findIndex", &prototype, 1);
        make_builtin_fn(Self::slice, "slice", &prototype, 2);
        make_builtin_fn(Self::some, "some", &prototype, 2);

        let array = make_constructor_fn("Array", 1, Self::make_array, global, prototype, true);

        // Static Methods
        make_builtin_fn(Self::is_array, "isArray", &array, 1);

        array
    }

    /// Initialise the `Array` object on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) -> (&str, Value) {
        let _timer = BoaProfiler::global().start_event("array", "init");

        ("Array", Self::create(global))
    }
}
