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

pub mod array_iterator;
#[cfg(test)]
mod tests;

use crate::{
    builtins::array::array_iterator::{ArrayIterationKind, ArrayIterator},
    builtins::BuiltIn,
    gc::GcObject,
    object::{ConstructorBuilder, FunctionBuilder, ObjectData, PROTOTYPE},
    property::{Attribute, DataDescriptor},
    value::{same_value_zero, IntegerOrInfinity, Value},
    BoaProfiler, Context, Result,
};
use num_traits::*;
use std::{
    cmp::{max, min},
    convert::{TryFrom, TryInto},
};

/// JavaScript `Array` built-in implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Array;

impl BuiltIn for Array {
    const NAME: &'static str = "Array";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let symbol_iterator = context.well_known_symbols().iterator_symbol();

        let values_function = FunctionBuilder::new(context, Self::values)
            .name("values")
            .length(0)
            .callable(true)
            .constructable(false)
            .build();

        let array = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().array_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .property(
            "length",
            0,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        )
        .property(
            "values",
            values_function.clone(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .property(
            symbol_iterator,
            values_function,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .method(Self::concat, "concat", 1)
        .method(Self::push, "push", 1)
        .method(Self::index_of, "indexOf", 1)
        .method(Self::last_index_of, "lastIndexOf", 1)
        .method(Self::includes_value, "includes", 1)
        .method(Self::map, "map", 1)
        .method(Self::fill, "fill", 1)
        .method(Self::for_each, "forEach", 1)
        .method(Self::filter, "filter", 1)
        .method(Self::pop, "pop", 0)
        .method(Self::join, "join", 1)
        .method(Self::to_string, "toString", 0)
        .method(Self::reverse, "reverse", 0)
        .method(Self::shift, "shift", 0)
        .method(Self::unshift, "unshift", 1)
        .method(Self::every, "every", 1)
        .method(Self::find, "find", 1)
        .method(Self::find_index, "findIndex", 1)
        .method(Self::slice, "slice", 2)
        .method(Self::some, "some", 2)
        .method(Self::reduce, "reduce", 2)
        .method(Self::reduce_right, "reduceRight", 2)
        .method(Self::keys, "keys", 0)
        .method(Self::entries, "entries", 0)
        // Static Methods
        .static_method(Self::is_array, "isArray", 1)
        .build();

        (Self::NAME, array.into(), Self::attribute())
    }
}

impl Array {
    const LENGTH: usize = 1;

    fn constructor(new_target: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.get(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| context.standard_objects().array_object().prototype());
        // Delegate to the appropriate constructor based on the number of arguments
        match args.len() {
            0 => Ok(Array::construct_array_empty(prototype, context)),
            1 => Array::construct_array_length(prototype, &args[0], context),
            _ => Array::construct_array_values(prototype, args, context),
        }
    }

    /// No argument constructor for `Array`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array-constructor-array
    fn construct_array_empty(proto: GcObject, context: &mut Context) -> Value {
        Array::array_create(0, Some(proto), context)
    }

    /// By length constructor for `Array`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array-len
    fn construct_array_length(
        prototype: GcObject,
        length: &Value,
        context: &mut Context,
    ) -> Result<Value> {
        let array = Array::array_create(0, Some(prototype), context);

        if !length.is_number() {
            array.set_property(0, DataDescriptor::new(length, Attribute::all()));
            array.set_field("length", 1, context)?;
        } else {
            if length.is_double() {
                return context.throw_range_error("Invalid array length");
            }
            array.set_field("length", length.to_u32(context).unwrap(), context)?;
        }

        Ok(array)
    }

    /// From items constructor for `Array`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array-items
    fn construct_array_values(
        prototype: GcObject,
        items: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        let items_len = items.len().try_into().map_err(interror_to_value)?;
        let array = Array::array_create(items_len, Some(prototype), context);

        for (k, item) in items.iter().enumerate() {
            array.set_property(k, DataDescriptor::new(item.clone(), Attribute::all()));
        }

        Ok(array)
    }

    /// Utility for constructing `Array` objects.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraycreate
    fn array_create(length: u32, prototype: Option<GcObject>, context: &mut Context) -> Value {
        let prototype = match prototype {
            Some(prototype) => prototype,
            None => context.standard_objects().array_object().prototype(),
        };
        let array = Value::new_object(context);

        array
            .as_object()
            .expect("'array' should be an object")
            .set_prototype_instance(prototype.into());
        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        array.set_data(ObjectData::Array);

        let length = DataDescriptor::new(
            length,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
        array.set_property("length", length);

        array
    }

    /// Creates a new `Array` instance.
    pub(crate) fn new_array(context: &Context) -> Value {
        let array = Value::new_object(context);
        array.set_data(ObjectData::Array);
        array
            .as_object()
            .expect("'array' should be an object")
            .set_prototype_instance(context.standard_objects().array_object().prototype().into());
        let length = DataDescriptor::new(
            Value::from(0),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
        array.set_property("length", length);
        array
    }

    /// Utility function for creating array objects.
    ///
    /// `array_obj` can be any array with prototype already set (it will be wiped and
    /// recreated from `array_contents`)
    pub(crate) fn construct_array(
        array_obj: &Value,
        array_contents: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        let array_obj_ptr = array_obj.clone();

        // Wipe existing contents of the array object
        let orig_length = array_obj.get_field("length", context)?.to_length(context)?;
        for n in 0..orig_length {
            array_obj_ptr.remove_property(n);
        }

        // Create length
        let length = DataDescriptor::new(
            array_contents.len(),
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
        array_obj_ptr.set_property("length".to_string(), length);

        for (n, value) in array_contents.iter().enumerate() {
            array_obj_ptr.set_property(n, DataDescriptor::new(value, Attribute::all()));
        }
        Ok(array_obj_ptr)
    }

    /// Utility function which takes an existing array object and puts additional
    /// values on the end, correctly rewriting the length
    pub(crate) fn add_to_array_object(
        array_ptr: &Value,
        add_values: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        let orig_length = array_ptr.get_field("length", context)?.to_length(context)?;

        for (n, value) in add_values.iter().enumerate() {
            let new_index = orig_length.wrapping_add(n);
            array_ptr.set_property(new_index, DataDescriptor::new(value, Attribute::all()));
        }

        array_ptr.set_field(
            "length",
            Value::from(orig_length.wrapping_add(add_values.len())),
            context,
        )?;

        Ok(array_ptr.clone())
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
    pub(crate) fn is_array(_: &Value, args: &[Value], _: &mut Context) -> Result<Value> {
        match args.get(0).and_then(|x| x.as_object()) {
            Some(object) => Ok(Value::from(object.borrow().is_array())),
            None => Ok(Value::from(false)),
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
    pub(crate) fn concat(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if args.is_empty() {
            // If concat is called with no arguments, it returns the original array
            return Ok(this.clone());
        }

        // Make a new array (using this object as the prototype basis for the new
        // one)
        let mut new_values: Vec<Value> = Vec::new();

        let this_length = this.get_field("length", context)?.to_length(context)?;
        for n in 0..this_length {
            new_values.push(this.get_field(n, context)?);
        }

        for concat_array in args {
            let concat_length = concat_array
                .get_field("length", context)?
                .to_length(context)?;
            for n in 0..concat_length {
                new_values.push(concat_array.get_field(n, context)?);
            }
        }

        Self::construct_array(this, &new_values, context)
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
    pub(crate) fn push(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let new_array = Self::add_to_array_object(this, args, context)?;
        new_array.get_field("length", context)
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
    pub(crate) fn pop(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        let curr_length = this.get_field("length", context)?.to_length(context)?;

        if curr_length < 1 {
            return Ok(Value::undefined());
        }
        let pop_index = curr_length.wrapping_sub(1);
        let pop_value: Value = this.get_field(pop_index.to_string(), context)?;
        this.remove_property(pop_index);
        this.set_field("length", Value::from(pop_index), context)?;
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
    pub(crate) fn for_each(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if args.is_empty() {
            return Err(Value::from("Missing argument for Array.prototype.forEach"));
        }

        let callback_arg = args.get(0).expect("Could not get `callbackFn` argument.");
        let this_arg = args.get(1).cloned().unwrap_or_else(Value::undefined);

        let length = this.get_field("length", context)?.to_length(context)?;

        for i in 0..length {
            let element = this.get_field(i, context)?;
            let arguments = [element, Value::from(i), this.clone()];

            context.call(callback_arg, &this_arg, &arguments)?;
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
    pub(crate) fn join(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let separator = if args.is_empty() {
            String::from(",")
        } else {
            args.get(0)
                .expect("Could not get argument")
                .to_string(context)?
                .to_string()
        };

        let mut elem_strs = Vec::new();
        let length = this.get_field("length", context)?.to_length(context)?;
        for n in 0..length {
            let elem_str = this.get_field(n, context)?.to_string(context)?.to_string();
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
    pub(crate) fn to_string(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        let method_name = "join";
        let mut arguments = vec![Value::from(",")];
        // 2.
        let mut method = this.get_field(method_name, context)?;
        // 3.
        if !method.is_function() {
            let object_prototype: Value = context
                .standard_objects()
                .object_object()
                .prototype()
                .into();
            method = object_prototype.get_field("toString", context)?;

            arguments = Vec::new();
        }
        // 4.
        let join = context.call(&method, this, &arguments)?;

        let string = if let Value::String(ref s) = join {
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
    pub(crate) fn reverse(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        let len = this.get_field("length", context)?.to_length(context)?;

        let middle = len.wrapping_div(2);

        for lower in 0..middle {
            let upper = len.wrapping_sub(lower).wrapping_sub(1);

            let upper_exists = this.has_field(upper);
            let lower_exists = this.has_field(lower);

            let upper_value = this.get_field(upper, context)?;
            let lower_value = this.get_field(lower, context)?;

            if upper_exists && lower_exists {
                this.set_property(upper, DataDescriptor::new(lower_value, Attribute::all()));
                this.set_property(lower, DataDescriptor::new(upper_value, Attribute::all()));
            } else if upper_exists {
                this.set_property(lower, DataDescriptor::new(upper_value, Attribute::all()));
                this.remove_property(upper);
            } else if lower_exists {
                this.set_property(upper, DataDescriptor::new(lower_value, Attribute::all()));
                this.remove_property(lower);
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
    pub(crate) fn shift(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        let len = this.get_field("length", context)?.to_length(context)?;

        if len == 0 {
            this.set_field("length", 0, context)?;
            return Ok(Value::undefined());
        }

        let first: Value = this.get_field(0, context)?;

        for k in 1..len {
            let from = k;
            let to = k.wrapping_sub(1);

            let from_value = this.get_field(from, context)?;
            if from_value.is_undefined() {
                this.remove_property(to);
            } else {
                this.set_property(to, DataDescriptor::new(from_value, Attribute::all()));
            }
        }

        let final_index = len.wrapping_sub(1);
        this.remove_property(final_index);
        this.set_field("length", Value::from(final_index), context)?;

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
    pub(crate) fn unshift(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let len = this.get_field("length", context)?.to_length(context)?;

        let arg_c = args.len();

        if arg_c > 0 {
            for k in (1..=len).rev() {
                let from = k.wrapping_sub(1);
                let to = k.wrapping_add(arg_c).wrapping_sub(1);

                let from_value = this.get_field(from, context)?;
                if from_value.is_undefined() {
                    this.remove_property(to);
                } else {
                    this.set_property(to, DataDescriptor::new(from_value, Attribute::all()));
                }
            }
            for j in 0..arg_c {
                this.set_property(
                    j,
                    DataDescriptor::new(
                        args.get(j).expect("Could not get argument").clone(),
                        Attribute::all(),
                    ),
                );
            }
        }

        let temp = len.wrapping_add(arg_c);
        this.set_field("length", Value::from(temp), context)?;
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
    pub(crate) fn every(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if args.is_empty() {
            return Err(Value::from(
                "missing callback when calling function Array.prototype.every",
            ));
        }
        let callback = &args[0];
        let this_arg = if args.len() > 1 {
            args[1].clone()
        } else {
            Value::undefined()
        };
        let mut i = 0;
        let max_len = this.get_field("length", context)?.to_length(context)?;
        let mut len = max_len;
        while i < len {
            let element = this.get_field(i, context)?;
            let arguments = [element, Value::from(i), this.clone()];
            let result = context.call(callback, &this_arg, &arguments)?;
            if !result.to_boolean() {
                return Ok(Value::from(false));
            }
            len = min(
                max_len,
                this.get_field("length", context)?.to_length(context)?,
            );
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
    pub(crate) fn map(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if args.is_empty() {
            return Err(Value::from(
                "missing argument 0 when calling function Array.prototype.map",
            ));
        }

        let callback = args.get(0).cloned().unwrap_or_else(Value::undefined);
        let this_val = args.get(1).cloned().unwrap_or_else(Value::undefined);

        let length = this.get_field("length", context)?.to_length(context)?;

        if length > 2usize.pow(32) - 1 {
            return context.throw_range_error("Invalid array length");
        }

        let new = Self::new_array(context);

        let values = (0..length)
            .map(|idx| {
                let element = this.get_field(idx, context)?;
                let args = [element, Value::from(idx), new.clone()];

                context.call(&callback, &this_val, &args)
            })
            .collect::<Result<Vec<Value>>>()?;

        Self::construct_array(&new, &values, context)
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
    pub(crate) fn index_of(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // If no arguments, return -1. Not described in spec, but is what chrome does.
        if args.is_empty() {
            return Ok(Value::from(-1));
        }

        let search_element = args[0].clone();
        let len = this.get_field("length", context)?.to_length(context)?;

        let mut idx = match args.get(1) {
            Some(from_idx_ptr) => {
                let from_idx = from_idx_ptr.to_number(context)?;

                if !from_idx.is_finite() {
                    return Ok(Value::from(-1));
                } else if from_idx < 0.0 {
                    let k =
                        isize::try_from(len).map_err(interror_to_value)? + f64_to_isize(from_idx)?;
                    usize::try_from(max(0, k)).map_err(interror_to_value)?
                } else {
                    f64_to_usize(from_idx)?
                }
            }
            None => 0,
        };

        while idx < len {
            let check_element = this.get_field(idx, context)?.clone();

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
        this: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        // If no arguments, return -1. Not described in spec, but is what chrome does.
        if args.is_empty() {
            return Ok(Value::from(-1));
        }

        let search_element = args[0].clone();
        let len: isize = this
            .get_field("length", context)?
            .to_length(context)?
            .try_into()
            .map_err(interror_to_value)?;

        let mut idx = match args.get(1) {
            Some(from_idx_ptr) => {
                let from_idx = from_idx_ptr.to_integer(context)?;

                if !from_idx.is_finite() {
                    return Ok(Value::from(-1));
                } else if from_idx < 0.0 {
                    len + f64_to_isize(from_idx)?
                } else {
                    min(f64_to_isize(from_idx)?, len - 1)
                }
            }
            None => len - 1,
        };

        while idx >= 0 {
            let check_element = this.get_field(idx, context)?.clone();

            if check_element.strict_equals(&search_element) {
                return Ok(Value::from(i32::try_from(idx).map_err(interror_to_value)?));
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
    pub(crate) fn find(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if args.is_empty() {
            return Err(Value::from(
                "missing callback when calling function Array.prototype.find",
            ));
        }
        let callback = &args[0];
        let this_arg = args.get(1).cloned().unwrap_or_else(Value::undefined);
        let len = this.get_field("length", context)?.to_length(context)?;
        for i in 0..len {
            let element = this.get_field(i, context)?;
            let arguments = [element.clone(), Value::from(i), this.clone()];
            let result = context.call(callback, &this_arg, &arguments)?;
            if result.to_boolean() {
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
    pub(crate) fn find_index(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if args.is_empty() {
            return Err(Value::from(
                "Missing argument for Array.prototype.findIndex",
            ));
        }

        let predicate_arg = args.get(0).expect("Could not get `predicate` argument.");

        let this_arg = args.get(1).cloned().unwrap_or_else(Value::undefined);

        let length = this.get_field("length", context)?.to_length(context)?;

        for i in 0..length {
            let element = this.get_field(i, context)?;
            let arguments = [element, Value::from(i), this.clone()];

            let result = context.call(predicate_arg, &this_arg, &arguments)?;

            if result.to_boolean() {
                let result = i32::try_from(i).map_err(interror_to_value)?;
                return Ok(Value::integer(result));
            }
        }

        Ok(Value::integer(-1))
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
    pub(crate) fn fill(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let len = this.get_field("length", context)?.to_length(context)?;

        let default_value = Value::undefined();
        let value = args.get(0).unwrap_or(&default_value);
        let start = Self::get_relative_start(context, args.get(1), len)?;
        let fin = Self::get_relative_end(context, args.get(2), len)?;

        for i in start..fin {
            this.set_property(i, DataDescriptor::new(value.clone(), Attribute::all()));
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
        this: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        let search_element = args.get(0).cloned().unwrap_or_else(Value::undefined);

        let length = this.get_field("length", context)?.to_length(context)?;

        for idx in 0..length {
            let check_element = this.get_field(idx, context)?.clone();

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
    pub(crate) fn slice(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let new_array = Self::new_array(context);

        let len = this.get_field("length", context)?.to_length(context)?;
        let from = Self::get_relative_start(context, args.get(0), len)?;
        let to = Self::get_relative_end(context, args.get(1), len)?;

        let span = max(to.saturating_sub(from), 0);
        if span > 2usize.pow(32) - 1 {
            return context.throw_range_error("Invalid array length");
        }
        let mut new_array_len: i32 = 0;
        for i in from..from.saturating_add(span) {
            new_array.set_property(
                new_array_len,
                DataDescriptor::new(this.get_field(i, context)?, Attribute::all()),
            );
            new_array_len = new_array_len.saturating_add(1);
        }
        new_array.set_field("length", Value::from(new_array_len), context)?;
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
    pub(crate) fn filter(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if args.is_empty() {
            return Err(Value::from(
                "missing argument 0 when calling function Array.prototype.filter",
            ));
        }

        let callback = args.get(0).cloned().unwrap_or_else(Value::undefined);
        let this_val = args.get(1).cloned().unwrap_or_else(Value::undefined);

        let length = this.get_field("length", context)?.to_length(context)?;

        let new = Self::new_array(context);

        let values = (0..length)
            .map(|idx| {
                let element = this.get_field(idx, context)?;

                let args = [element.clone(), Value::from(idx), new.clone()];

                let callback_result = context.call(&callback, &this_val, &args)?;

                if callback_result.to_boolean() {
                    Ok(Some(element))
                } else {
                    Ok(None)
                }
            })
            .collect::<Result<Vec<Option<Value>>>>()?;
        let values = values.into_iter().flatten().collect::<Vec<_>>();

        Self::construct_array(&new, &values, context)
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
    pub(crate) fn some(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if args.is_empty() {
            return Err(Value::from(
                "missing callback when calling function Array.prototype.some",
            ));
        }
        let callback = &args[0];
        let this_arg = if args.len() > 1 {
            args[1].clone()
        } else {
            Value::undefined()
        };
        let mut i = 0;
        let max_len = this.get_field("length", context)?.to_length(context)?;
        let mut len = max_len;
        while i < len {
            let element = this.get_field(i, context)?;
            let arguments = [element, Value::from(i), this.clone()];
            let result = context.call(callback, &this_arg, &arguments)?;
            if result.to_boolean() {
                return Ok(Value::from(true));
            }
            // the length of the array must be updated because the callback can mutate it.
            len = min(
                max_len,
                this.get_field("length", context)?.to_length(context)?,
            );
            i += 1;
        }
        Ok(Value::from(false))
    }

    /// `Array.prototype.reduce( callbackFn [ , initialValue ] )`
    ///
    /// The reduce method traverses left to right starting from the first defined value in the array,
    /// accumulating a value using a given callback function. It returns the accumulated value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.reduce
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/reduce
    pub(crate) fn reduce(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let this: Value = this.to_object(context)?.into();
        let callback = match args.get(0) {
            Some(value) if value.is_function() => value,
            _ => return context.throw_type_error("Reduce was called without a callback"),
        };
        let initial_value = args.get(1).cloned().unwrap_or_else(Value::undefined);
        let mut length = this.get_field("length", context)?.to_length(context)?;
        if length == 0 && initial_value.is_undefined() {
            return context
                .throw_type_error("Reduce was called on an empty array and with no initial value");
        }
        let mut k = 0;
        let mut accumulator = if initial_value.is_undefined() {
            let mut k_present = false;
            while k < length {
                if this.has_field(k) {
                    k_present = true;
                    break;
                }
                k += 1;
            }
            if !k_present {
                return context.throw_type_error(
                    "Reduce was called on an empty array and with no initial value",
                );
            }
            let result = this.get_field(k, context)?;
            k += 1;
            result
        } else {
            initial_value
        };
        while k < length {
            if this.has_field(k) {
                let arguments = [
                    accumulator,
                    this.get_field(k, context)?,
                    Value::from(k),
                    this.clone(),
                ];
                accumulator = context.call(&callback, &Value::undefined(), &arguments)?;
                /* We keep track of possibly shortened length in order to prevent unnecessary iteration.
                It may also be necessary to do this since shortening the array length does not
                delete array elements. See: https://github.com/boa-dev/boa/issues/557 */
                length = min(
                    length,
                    this.get_field("length", context)?.to_length(context)?,
                );
            }
            k += 1;
        }
        Ok(accumulator)
    }

    /// `Array.prototype.reduceRight( callbackFn [ , initialValue ] )`
    ///
    /// The reduceRight method traverses right to left starting from the last defined value in the array,
    /// accumulating a value using a given callback function. It returns the accumulated value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.reduceright
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/reduceRight
    pub(crate) fn reduce_right(
        this: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        let this: Value = this.to_object(context)?.into();
        let callback = match args.get(0) {
            Some(value) if value.is_function() => value,
            _ => return context.throw_type_error("reduceRight was called without a callback"),
        };
        let initial_value = args.get(1).cloned().unwrap_or_else(Value::undefined);
        let mut length = this.get_field("length", context)?.to_length(context)?;
        if length == 0 {
            return if initial_value.is_undefined() {
                context.throw_type_error(
                    "reduceRight was called on an empty array and with no initial value",
                )
            } else {
                // early return to prevent usize subtraction errors
                Ok(initial_value)
            };
        }
        let mut k = length - 1;
        let mut accumulator = if initial_value.is_undefined() {
            let mut k_present = false;
            loop {
                if this.has_field(k) {
                    k_present = true;
                    break;
                }
                // check must be done at the end to prevent usize subtraction error
                if k == 0 {
                    break;
                }
                k -= 1;
            }
            if !k_present {
                return context.throw_type_error(
                    "reduceRight was called on an empty array and with no initial value",
                );
            }
            let result = this.get_field(k, context)?;
            k = k.overflowing_sub(1).0;
            result
        } else {
            initial_value
        };
        // usize::MAX is bigger than the maximum array size so we can use it check for integer undeflow
        while k != usize::MAX {
            if this.has_field(k) {
                let arguments = [
                    accumulator,
                    this.get_field(k, context)?,
                    Value::from(k),
                    this.clone(),
                ];
                accumulator = context.call(&callback, &Value::undefined(), &arguments)?;
                /* We keep track of possibly shortened length in order to prevent unnecessary iteration.
                It may also be necessary to do this since shortening the array length does not
                delete array elements. See: https://github.com/boa-dev/boa/issues/557 */
                length = min(
                    length,
                    this.get_field("length", context)?.to_length(context)?,
                );

                // move k to the last defined element if necessary or return if the length was set to 0
                if k >= length {
                    if length == 0 {
                        return Ok(accumulator);
                    } else {
                        k = length - 1;
                        continue;
                    }
                }
            }
            if k == 0 {
                break;
            }
            k = k.overflowing_sub(1).0;
        }
        Ok(accumulator)
    }

    /// `Array.prototype.values( )`
    ///
    /// The values method returns an iterable that iterates over the values in the array.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.values
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/values
    pub(crate) fn values(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Ok(ArrayIterator::create_array_iterator(
            context,
            this.clone(),
            ArrayIterationKind::Value,
        ))
    }

    /// `Array.prototype.keys( )`
    ///
    /// The keys method returns an iterable that iterates over the indexes in the array.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.values
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/values
    pub(crate) fn keys(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Ok(ArrayIterator::create_array_iterator(
            context,
            this.clone(),
            ArrayIterationKind::Key,
        ))
    }

    /// `Array.prototype.entries( )`
    ///
    /// The entries method returns an iterable that iterates over the key-value pairs in the array.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.values
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/values
    pub(crate) fn entries(this: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        Ok(ArrayIterator::create_array_iterator(
            context,
            this.clone(),
            ArrayIterationKind::KeyAndValue,
        ))
    }

    /// Represents the algorithm to calculate `relativeStart` (or `k`) in array functions.
    pub(super) fn get_relative_start(
        context: &mut Context,
        arg: Option<&Value>,
        len: usize,
    ) -> Result<usize> {
        let default_value = Value::undefined();
        // 1. Let relativeStart be ? ToIntegerOrInfinity(start).
        let relative_start = arg
            .unwrap_or(&default_value)
            .to_integer_or_infinity(context)?;
        match relative_start {
            // 2. If relativeStart is -∞, let k be 0.
            IntegerOrInfinity::NegativeInfinity => Ok(0),
            // 3. Else if relativeStart < 0, let k be max(len + relativeStart, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => Self::offset(len as u64, i)
                .try_into()
                .map_err(interror_to_value),
            // 4. Else, let k be min(relativeStart, len).
            // Both `as` casts are safe as both variables are non-negative
            IntegerOrInfinity::Integer(i) => (i as u64)
                .min(len as u64)
                .try_into()
                .map_err(interror_to_value),

            // Special case - postive infinity. `len` is always smaller than +inf, thus from (4)
            IntegerOrInfinity::PositiveInfinity => Ok(len),
        }
    }

    /// Represents the algorithm to calculate `relativeEnd` (or `final`) in array functions.
    pub(super) fn get_relative_end(
        context: &mut Context,
        arg: Option<&Value>,
        len: usize,
    ) -> Result<usize> {
        let default_value = Value::undefined();
        let value = arg.unwrap_or(&default_value);
        // 1. If end is undefined, let relativeEnd be len [and return it]
        if value.is_undefined() {
            Ok(len)
        } else {
            // 1. cont, else let relativeEnd be ? ToIntegerOrInfinity(end).
            let relative_end = value.to_integer_or_infinity(context)?;
            match relative_end {
                // 2. If relativeEnd is -∞, let final be 0.
                IntegerOrInfinity::NegativeInfinity => Ok(0),
                // 3. Else if relativeEnd < 0, let final be max(len + relativeEnd, 0).
                IntegerOrInfinity::Integer(i) if i < 0 => Self::offset(len as u64, i)
                    .try_into()
                    .map_err(interror_to_value),
                // 4. Else, let final be min(relativeEnd, len).
                // Both `as` casts are safe as both variables are non-negative
                IntegerOrInfinity::Integer(i) => (i as u64)
                    .min(len as u64)
                    .try_into()
                    .map_err(interror_to_value),

                // Special case - postive infinity. `len` is always smaller than +inf, thus from (4)
                IntegerOrInfinity::PositiveInfinity => Ok(len),
            }
        }
    }

    fn offset(len: u64, i: i64) -> u64 {
        // `Number::MIN_SAFE_INTEGER > i64::MIN` so this should always hold
        debug_assert!(i < 0 && i != i64::MIN);
        // `i.staurating_neg()` will always be less than `u64::MAX`
        len.saturating_sub(i.saturating_neg() as u64)
    }
}

fn f64_to_isize(v: f64) -> Result<isize> {
    v.to_isize()
        .ok_or_else(|| Value::string("cannot convert f64 to isize - out of range"))
}

fn f64_to_usize(v: f64) -> Result<usize> {
    v.to_usize()
        .ok_or_else(|| Value::string("cannot convert f64 to usize - out of range"))
}

fn interror_to_value(err: std::num::TryFromIntError) -> Value {
    Value::string(format!("{}", err))
}
