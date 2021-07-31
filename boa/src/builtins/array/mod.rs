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
    builtins::Number,
    object::{ConstructorBuilder, FunctionBuilder, GcObject, ObjectData, PROTOTYPE},
    property::{Attribute, DataDescriptor},
    symbol::WellKnownSymbols,
    value::{IntegerOrInfinity, Value},
    BoaProfiler, Context, JsString, Result,
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

        let symbol_iterator = WellKnownSymbols::iterator();

        let get_species = FunctionBuilder::native(context, Self::get_species)
            .name("get [Symbol.species]")
            .constructable(false)
            .build();

        let values_function = FunctionBuilder::native(context, Self::values)
            .name("values")
            .length(0)
            .constructable(false)
            .build();

        let array = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().array_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .static_accessor(
            WellKnownSymbols::species(),
            Some(get_species),
            None,
            Attribute::CONFIGURABLE,
        )
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
        .method(Self::flat, "flat", 0)
        .method(Self::flat_map, "flatMap", 1)
        .method(Self::slice, "slice", 2)
        .method(Self::some, "some", 2)
        .method(Self::reduce, "reduce", 2)
        .method(Self::reduce_right, "reduceRight", 2)
        .method(Self::keys, "keys", 0)
        .method(Self::entries, "entries", 0)
        .method(Self::copy_within, "copyWithin", 3)
        // Static Methods
        .static_method(Self::is_array, "isArray", 1)
        .static_method(Self::of, "of", 0)
        .build();

        (Self::NAME, array.into(), Self::attribute())
    }
}

impl Array {
    const LENGTH: usize = 1;

    fn constructor(new_target: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // If NewTarget is undefined, let newTarget be the active function object; else let newTarget be NewTarget.
        // 2. Let proto be ? GetPrototypeFromConstructor(newTarget, "%Array.prototype%").
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.__get__(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| context.standard_objects().array_object().prototype());

        // 3. Let numberOfArgs be the number of elements in values.
        let number_of_args = args.len();

        // 4. If numberOfArgs = 0, then
        if number_of_args == 0 {
            // 4.a. Return ! ArrayCreate(0, proto).
            Ok(Array::array_create(0, Some(prototype), context)
                .unwrap()
                .into())
        // 5. Else if numberOfArgs = 1, then
        } else if number_of_args == 1 {
            // a. Let len be values[0].
            let len = &args[0];
            // b. Let array be ! ArrayCreate(0, proto).
            let array = Array::array_create(0, Some(prototype), context).unwrap();
            // c. If Type(len) is not Number, then
            let int_len = if !len.is_number() {
                // i. Perform ! CreateDataPropertyOrThrow(array, "0", len).
                array
                    .create_data_property_or_throw(0, len, context)
                    .unwrap();
                // ii. Let intLen be 1𝔽.
                1
            // d. Else,
            } else {
                // i. Let intLen be ! ToUint32(len).
                let int_len = len.to_u32(context).unwrap();
                // ii. If SameValueZero(intLen, len) is false, throw a RangeError exception.
                if !Value::same_value_zero(&int_len.into(), len) {
                    return Err(context.construct_range_error("invalid array length"));
                }
                int_len
            };
            // e. Perform ! Set(array, "length", intLen, true).
            array.set("length", int_len, true, context).unwrap();
            // f. Return array.
            Ok(array.into())
        // 6. Else,
        } else {
            // 6.a. Assert: numberOfArgs ≥ 2.
            debug_assert!(number_of_args >= 2);

            // b. Let array be ? ArrayCreate(numberOfArgs, proto).
            let array = Array::array_create(number_of_args, Some(prototype), context)?;
            // c. Let k be 0.
            // d. Repeat, while k < numberOfArgs,
            for (i, item) in args.iter().cloned().enumerate() {
                // i. Let Pk be ! ToString(𝔽(k)).
                // ii. Let itemK be values[k].
                // iii. Perform ! CreateDataPropertyOrThrow(array, Pk, itemK).
                array
                    .create_data_property_or_throw(i, item, context)
                    .unwrap();
                // iv. Set k to k + 1.
            }
            // e. Assert: The mathematical value of array's "length" property is numberOfArgs.
            // f. Return array.
            Ok(array.into())
        }
    }

    /// Utility for constructing `Array` objects.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraycreate
    pub(crate) fn array_create(
        length: usize,
        prototype: Option<GcObject>,
        context: &mut Context,
    ) -> Result<GcObject> {
        // 1. If length > 2^32 - 1, throw a RangeError exception.
        if length > 2usize.pow(32) - 1 {
            return Err(context.construct_range_error("array exceeded max size"));
        }
        // 7. Return A.
        // 2. If proto is not present, set proto to %Array.prototype%.
        // 3. Let A be ! MakeBasicObject(« [[Prototype]], [[Extensible]] »).
        // 4. Set A.[[Prototype]] to proto.
        // 5. Set A.[[DefineOwnProperty]] as specified in 10.4.2.1.
        let prototype = match prototype {
            Some(prototype) => prototype,
            None => context.standard_objects().array_object().prototype(),
        };
        let array = context.construct_object();

        array.set_prototype_instance(prototype.into());
        // This value is used by console.log and other routines to match Object type
        // to its Javascript Identifier (global constructor method name)
        array.borrow_mut().data = ObjectData::Array;

        // 6. Perform ! OrdinaryDefineOwnProperty(A, "length", PropertyDescriptor { [[Value]]: 𝔽(length), [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: false }).
        let length = DataDescriptor::new(
            length as f64,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
        );
        array.ordinary_define_own_property("length".into(), length.into());

        Ok(array)
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

    /// Utility function for concatenating array objects.
    ///
    /// Returns a Boolean valued property that if `true` indicates that
    /// an object should be flattened to its array elements
    /// by `Array.prototype.concat`.
    fn is_concat_spreadable(this: &Value, context: &mut Context) -> Result<bool> {
        // 1. If Type(O) is not Object, return false.
        if !this.is_object() {
            return Ok(false);
        }
        // 2. Let spreadable be ? Get(O, @@isConcatSpreadable).
        let spreadable = this.get_field(WellKnownSymbols::is_concat_spreadable(), context)?;

        // 3. If spreadable is not undefined, return ! ToBoolean(spreadable).
        if !spreadable.is_undefined() {
            return Ok(spreadable.to_boolean());
        }
        // 4. Return ? IsArray(O).
        match this.as_object() {
            Some(obj) => Ok(obj.is_array()),
            _ => Ok(false),
        }
    }

    /// `get Array [ @@species ]`
    ///
    /// The `Array [ @@species ]` accessor property returns the Array constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-array-@@species
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/@@species
    fn get_species(this: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// Utility function used to specify the creation of a new Array object using a constructor
    /// function that is derived from original_array.
    ///
    /// see: <https://tc39.es/ecma262/#sec-arrayspeciescreate>
    pub(crate) fn array_species_create(
        original_array: &GcObject,
        length: usize,
        context: &mut Context,
    ) -> Result<GcObject> {
        // 1. Let isArray be ? IsArray(originalArray).
        // 2. If isArray is false, return ? ArrayCreate(length).
        if !original_array.is_array() {
            return Self::array_create(length, None, context);
        }
        // 3. Let C be ? Get(originalArray, "constructor").
        let c = original_array.get("constructor", context)?;

        // 4. If IsConstructor(C) is true, then
        //     a. Let thisRealm be the current Realm Record.
        //     b. Let realmC be ? GetFunctionRealm(C).
        //     c. If thisRealm and realmC are not the same Realm Record, then
        //         i. If SameValue(C, realmC.[[Intrinsics]].[[%Array%]]) is true, set C to undefined.
        // TODO: Step 4 is ignored, as there are no different realms for now

        // 5. If Type(C) is Object, then
        let c = if let Some(c) = c.as_object() {
            // 5.a. Set C to ? Get(C, @@species).
            let c = c.get(WellKnownSymbols::species(), context)?;
            // 5.b. If C is null, set C to undefined.
            if c.is_null_or_undefined() {
                Value::undefined()
            } else {
                c
            }
        } else {
            c
        };

        // 6. If C is undefined, return ? ArrayCreate(length).
        if c.is_undefined() {
            return Self::array_create(length, None, context);
        }

        // 7. If IsConstructor(C) is false, throw a TypeError exception.
        if let Some(c) = c.as_object() {
            if !c.is_constructable() {
                return Err(context.construct_type_error("Symbol.species must be a constructor"));
            }
            // 8. Return ? Construct(C, « 𝔽(length) »).
            Ok(
                c.construct(&[Value::from(length)], &c.clone().into(), context)?
                    .as_object()
                    .unwrap(),
            )
        } else {
            Err(context.construct_type_error("Symbol.species must be a constructor"))
        }
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
            false,
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

    /// `Array.of(...items)`
    ///
    /// The Array.of method creates a new Array instance from a variable number of arguments,
    /// regardless of the number or type of arguments.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.of
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/of
    pub(crate) fn of(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let len be the number of elements in items.
        // 2. Let lenNumber be 𝔽(len).
        let len = args.len();

        // 3. Let C be the this value.
        // 4. If IsConstructor(C) is true, then
        //     a. Let A be ? Construct(C, « lenNumber »).
        // 5. Else,
        //     a. Let A be ? ArrayCreate(len).
        let a = match this.as_object() {
            Some(object) if object.is_constructable() => object
                .construct(&[len.into()], this, context)?
                .as_object()
                .unwrap(),
            _ => Array::array_create(len, None, context)?,
        };

        // 6. Let k be 0.
        // 7. Repeat, while k < len,
        for (k, value) in args.iter().enumerate() {
            // a. Let kValue be items[k].
            // b. Let Pk be ! ToString(𝔽(k)).
            // c. Perform ? CreateDataPropertyOrThrow(A, Pk, kValue).
            a.create_data_property_or_throw(k, value, context)?;
            // d. Set k to k + 1.
        }

        // 8. Perform ? Set(A, "length", lenNumber, true).
        a.set("length", len, true, context)?;

        // 9. Return A.
        Ok(a.into())
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
        // 1. Let O be ? ToObject(this value).
        let obj = this.to_object(context)?;
        // 2. Let A be ? ArraySpeciesCreate(O, 0).
        let arr = Self::array_species_create(&obj, 0, context)?;
        // 3. Let n be 0.
        let mut n = 0;
        // 4. Prepend O to items.
        // 5. For each element E of items, do
        for item in [Value::from(obj)].iter().chain(args.iter()) {
            // a. Let spreadable be ? IsConcatSpreadable(E).
            let spreadable = Self::is_concat_spreadable(item, context)?;
            // b. If spreadable is true, then
            if spreadable {
                // item is guaranteed to be an object since is_concat_spreadable checks it,
                // so we can call `.unwrap()`
                let item = item.as_object().unwrap();
                // i. Let k be 0.
                // ii. Let len be ? LengthOfArrayLike(E).
                let len = item.length_of_array_like(context)?;
                // iii. If n + len > 2^53 - 1, throw a TypeError exception.
                if n + len > Number::MAX_SAFE_INTEGER as usize {
                    return context.throw_type_error(
                        "length + number of arguments exceeds the max safe integer limit",
                    );
                }
                // iv. Repeat, while k < len,
                for k in 0..len {
                    // 1. Let P be ! ToString(𝔽(k)).
                    // 2. Let exists be ? HasProperty(E, P).
                    let exists = item.has_property(k, context)?;
                    // 3. If exists is true, then
                    if exists {
                        // a. Let subElement be ? Get(E, P).
                        let sub_element = item.get(k, context)?;
                        // b. Perform ? CreateDataPropertyOrThrow(A, ! ToString(𝔽(n)), subElement).
                        arr.create_data_property_or_throw(n, sub_element, context)?;
                    }
                    // 4. Set n to n + 1.
                    n += 1;
                    // 5. Set k to k + 1.
                }
            }
            // c. Else,
            else {
                // i. NOTE: E is added as a single item rather than spread.
                // ii. If n ≥ 253 - 1, throw a TypeError exception.
                if n >= Number::MAX_SAFE_INTEGER as usize {
                    return context.throw_type_error("length exceeds the max safe integer limit");
                }
                // iii. Perform ? CreateDataPropertyOrThrow(A, ! ToString(𝔽(n)), E).
                arr.create_data_property_or_throw(n, item, context)?;
                // iv. Set n to n + 1.
                n += 1
            }
        }
        // 6. Perform ? Set(A, "length", 𝔽(n), true).
        arr.set("length", n, true, context)?;

        // 7. Return A.
        Ok(Value::from(arr))
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
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;
        // 2. Let len be ? LengthOfArrayLike(O).
        let mut len = o.length_of_array_like(context)? as u64;
        // 3. Let argCount be the number of elements in items.
        let arg_count = args.len() as u64;
        // 4. If len + argCount > 2^53 - 1, throw a TypeError exception.
        if len + arg_count > 2u64.pow(53) - 1 {
            return context.throw_type_error(
                "the length + the number of arguments exceed the maximum safe integer limit",
            );
        }
        // 5. For each element E of items, do
        for element in args.iter().cloned() {
            // a. Perform ? Set(O, ! ToString(𝔽(len)), E, true).
            o.set(len, element, true, context)?;
            // b. Set len to len + 1.
            len += 1;
        }
        // 6. Perform ? Set(O, "length", 𝔽(len), true).
        o.set("length", len, true, context)?;
        // 7. Return 𝔽(len).
        Ok(len.into())
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
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;
        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o.length_of_array_like(context)?;
        // 3. If len = 0, then
        if len == 0 {
            // a. Perform ? Set(O, "length", +0𝔽, true).
            o.set("length", 0, true, context)?;
            // b. Return undefined.
            Ok(Value::undefined())
        // 4. Else,
        } else {
            // a. Assert: len > 0.
            // b. Let newLen be 𝔽(len - 1).
            let new_len = len - 1;
            // c. Let index be ! ToString(newLen).
            let index = new_len;
            // d. Let element be ? Get(O, index).
            let element = o.get(index, context)?;
            // e. Perform ? DeletePropertyOrThrow(O, index).
            o.delete_property_or_throw(index, context)?;
            // f. Perform ? Set(O, "length", newLen, true).
            o.set("length", new_len, true, context)?;
            // g. Return element.
            Ok(element)
        }
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
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;
        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o.length_of_array_like(context)?;
        // 3. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback = if let Some(arg) = args
            .get(0)
            .and_then(Value::as_object)
            .filter(GcObject::is_callable)
        {
            arg
        } else {
            return context.throw_type_error("Array.prototype.forEach: invalid callback function");
        };
        // 4. Let k be 0.
        // 5. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            let pk = k;
            // b. Let kPresent be ? HasProperty(O, Pk).
            let present = o.has_property(pk, context)?;
            // c. If kPresent is true, then
            if present {
                // i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(pk, context)?;
                // ii. Perform ? Call(callbackfn, thisArg, « kValue, 𝔽(k), O »).
                let this_arg = args.get(1).cloned().unwrap_or_else(Value::undefined);
                callback.call(&this_arg, &[k_value, k.into(), o.clone().into()], context)?;
            }
            // d. Set k to k + 1.
        }
        // 6. Return undefined.
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
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;
        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o.length_of_array_like(context)?;
        // 3. If separator is undefined, let sep be the single-element String ",".
        // 4. Else, let sep be ? ToString(separator).
        let separator = if let Some(separator) = args.get(0) {
            separator.to_string(context)?
        } else {
            JsString::new(",")
        };

        // 5. Let R be the empty String.
        let mut r = String::new();
        // 6. Let k be 0.
        // 7. Repeat, while k < len,
        for k in 0..len {
            // a. If k > 0, set R to the string-concatenation of R and sep.
            if k > 0 {
                r.push_str(&separator);
            }
            // b. Let element be ? Get(O, ! ToString(𝔽(k))).
            let element = o.get(k, context)?;
            // c. If element is undefined or null, let next be the empty String; otherwise, let next be ? ToString(element).
            let next = if element.is_null_or_undefined() {
                JsString::new("")
            } else {
                element.to_string(context)?
            };
            // d. Set R to the string-concatenation of R and next.
            r.push_str(&next);
            // e. Set k to k + 1.
        }
        // 8. Return R.
        Ok(r.into())
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
        // 1. Let array be ? ToObject(this value).
        let array = this.to_object(context)?;
        // 2. Let func be ? Get(array, "join").
        let func = array.get("join", context)?;
        // 3. If IsCallable(func) is false, set func to the intrinsic function %Object.prototype.toString%.
        // 4. Return ? Call(func, array).
        if let Some(func) = func.as_object().filter(GcObject::is_callable) {
            func.call(&array.into(), &[], context)
        } else {
            crate::builtins::object::Object::to_string(&array.into(), &[], context)
        }
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
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;
        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o.length_of_array_like(context)?;
        // 3. Let middle be floor(len / 2).
        let middle = len / 2;
        // 4. Let lower be 0.
        let mut lower = 0;
        // 5. Repeat, while lower ≠ middle,
        while lower != middle {
            // a. Let upper be len - lower - 1.
            let upper = len - lower - 1;
            // Skiped: b. Let upperP be ! ToString(𝔽(upper)).
            // Skiped: c. Let lowerP be ! ToString(𝔽(lower)).
            // d. Let lowerExists be ? HasProperty(O, lowerP).
            let lower_exists = o.has_property(lower, context)?;
            // e. If lowerExists is true, then
            let mut lower_value = Value::undefined();
            if lower_exists {
                // i. Let lowerValue be ? Get(O, lowerP).
                lower_value = o.get(lower, context)?;
            }
            // f. Let upperExists be ? HasProperty(O, upperP).
            let upper_exists = o.has_property(upper, context)?;
            // g. If upperExists is true, then
            let mut upper_value = Value::undefined();
            if upper_exists {
                // i. Let upperValue be ? Get(O, upperP).
                upper_value = o.get(upper, context)?;
            }
            match (lower_exists, upper_exists) {
                // h. If lowerExists is true and upperExists is true, then
                (true, true) => {
                    // i. Perform ? Set(O, lowerP, upperValue, true).
                    o.set(lower, upper_value, true, context)?;
                    // ii. Perform ? Set(O, upperP, lowerValue, true).
                    o.set(upper, lower_value, true, context)?;
                }
                // i. Else if lowerExists is false and upperExists is true, then
                (false, true) => {
                    // i. Perform ? Set(O, lowerP, upperValue, true).
                    o.set(lower, upper_value, true, context)?;
                    // ii. Perform ? DeletePropertyOrThrow(O, upperP).
                    o.delete_property_or_throw(upper, context)?;
                }
                // j. Else if lowerExists is true and upperExists is false, then
                (true, false) => {
                    // i. Perform ? DeletePropertyOrThrow(O, lowerP).
                    o.delete_property_or_throw(lower, context)?;
                    // ii. Perform ? Set(O, upperP, lowerValue, true).
                    o.set(upper, lower_value, true, context)?;
                }
                // k. Else,
                (false, false) => {
                    // i. Assert: lowerExists and upperExists are both false.
                    // ii. No action is required.
                }
            }

            // l. Set lower to lower + 1.
            lower += 1;
        }
        // 6. Return O.
        Ok(o.into())
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
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;
        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o.length_of_array_like(context)?;
        // 3. If len = 0, then
        if len == 0 {
            // a. Perform ? Set(O, "length", +0𝔽, true).
            o.set("length", 0, true, context)?;
            // b. Return undefined.
            return Ok(Value::undefined());
        }
        // 4. Let first be ? Get(O, "0").
        let first = o.get(0, context)?;
        // 5. Let k be 1.
        // 6. Repeat, while k < len,
        for k in 1..len {
            // a. Let from be ! ToString(𝔽(k)).
            let from = k;
            // b. Let to be ! ToString(𝔽(k - 1)).
            let to = k - 1;
            // c. Let fromPresent be ? HasProperty(O, from).
            let from_present = o.has_property(from, context)?;
            // d. If fromPresent is true, then
            if from_present {
                // i. Let fromVal be ? Get(O, from).
                let from_val = o.get(from, context)?;
                // ii. Perform ? Set(O, to, fromVal, true).
                o.set(to, from_val, true, context)?;
            // e. Else,
            } else {
                // i. Assert: fromPresent is false.
                // ii. Perform ? DeletePropertyOrThrow(O, to).
                o.delete_property_or_throw(to, context)?;
            }
            // f. Set k to k + 1.
        }
        // 7. Perform ? DeletePropertyOrThrow(O, ! ToString(𝔽(len - 1))).
        o.delete_property_or_throw(len - 1, context)?;
        // 8. Perform ? Set(O, "length", 𝔽(len - 1), true).
        o.set("length", len - 1, true, context)?;
        // 9. Return first.
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
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;
        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o.length_of_array_like(context)? as u64;
        // 3. Let argCount be the number of elements in items.
        let arg_count = args.len() as u64;
        // 4. If argCount > 0, then
        if arg_count > 0 {
            // a. If len + argCount > 2^53 - 1, throw a TypeError exception.
            if len + arg_count > 2u64.pow(53) - 1 {
                return context.throw_type_error(
                    "length + number of arguments exceeds the max safe integer limit",
                );
            }
            // b. Let k be len.
            let mut k = len;
            // c. Repeat, while k > 0,
            while k > 0 {
                // i. Let from be ! ToString(𝔽(k - 1)).
                let from = k - 1;
                // ii. Let to be ! ToString(𝔽(k + argCount - 1)).
                let to = k + arg_count - 1;
                // iii. Let fromPresent be ? HasProperty(O, from).
                let from_present = o.has_property(from, context)?;
                // iv. If fromPresent is true, then
                if from_present {
                    // 1. Let fromValue be ? Get(O, from).
                    let from_value = o.get(from, context)?;
                    // 2. Perform ? Set(O, to, fromValue, true).
                    o.set(to, from_value, true, context)?;
                // v. Else,
                } else {
                    // 1. Assert: fromPresent is false.
                    // 2. Perform ? DeletePropertyOrThrow(O, to).
                    o.delete_property_or_throw(to, context)?;
                }
                // vi. Set k to k - 1.
                k -= 1;
            }
            // d. Let j be +0𝔽.
            // e. For each element E of items, do
            for (j, e) in args.iter().enumerate() {
                // i. Perform ? Set(O, ! ToString(j), E, true).
                o.set(j, e, true, context)?;
                // ii. Set j to j + 1𝔽.
            }
        }
        // 5. Perform ? Set(O, "length", 𝔽(len + argCount), true).
        o.set("length", len + arg_count, true, context)?;
        // 6. Return 𝔽(len + argCount).
        Ok((len + arg_count).into())
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
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;
        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o.length_of_array_like(context)?;
        // 3. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback = if let Some(arg) = args
            .get(0)
            .and_then(Value::as_object)
            .filter(GcObject::is_callable)
        {
            arg
        } else {
            return context.throw_type_error("Array.prototype.every: callback is not callable");
        };

        // 4. Let k be 0.
        // 5. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.has_property(k, context)?;
            // c. If kPresent is true, then
            if k_present {
                // i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(k, context)?;
                // ii. Let testResult be ! ToBoolean(? Call(callbackfn, thisArg, « kValue, 𝔽(k), O »)).
                let this_arg = args.get(1).cloned().unwrap_or_default();
                let test_result = callback
                    .call(&this_arg, &[k_value, k.into(), o.clone().into()], context)?
                    .to_boolean();
                // iii. If testResult is false, return false.
                if !test_result {
                    return Ok(Value::from(false));
                }
            }
            // d. Set k to k + 1.
        }
        // 6. Return true.
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
        // 1. Let O be ? ToObject(this value).
        let this_val = args.get(1).cloned().unwrap_or_else(Value::undefined);
        let obj = this.to_object(context)?;
        // 2. Let len be ? LengthOfArrayLike(O).
        let len = this.get_field("length", context)?.to_length(context)?;
        // 3. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback = args.get(0).cloned().unwrap_or_else(Value::undefined);
        if !callback.is_function() {
            return context.throw_type_error("Callbackfn is not callable");
        }
        // 4. Let A be ? ArraySpeciesCreate(O, len).
        let arr = Self::array_species_create(&obj, len, context)?;
        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let k_present be ? HasProperty(O, Pk).
            let k_present = this.has_field(k);
            // c. If k_present is true, then
            if k_present {
                // i. Let kValue be ? Get(O, Pk).
                let k_value = this.get_field(k, context)?;
                let args = [k_value, Value::from(k), this.into()];
                // ii. Let mappedValue be ? Call(callbackfn, thisArg, « kValue, 𝔽(k), O »).
                let value = context.call(&callback, &this_val, &args)?;
                // iii. Perform ? CreateDataPropertyOrThrow(A, Pk, mappedValue).
                arr.ordinary_define_own_property(
                    k.into(),
                    DataDescriptor::new(value, Attribute::all()).into(),
                );
            }
            // d. Set k to k + 1.
        }
        // 7. Return A.
        Ok(Value::from(arr))
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

    /// `Array.prototype.flat( [depth] )`
    ///
    /// This method creates a new array with all sub-array elements concatenated into it
    /// recursively up to the specified depth.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.flat
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flat
    pub(crate) fn flat(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let O be ToObject(this value)
        let this: Value = this.to_object(context)?.into();

        // 2. Let sourceLen be LengthOfArrayLike(O)
        let source_len = this.get_field("length", context)?.to_length(context)? as u32;

        // 3. Let depthNum be 1
        let depth = args.get(0);
        let default_depth = Value::Integer(1);

        // 4. If depth is not undefined, then set depthNum to IntegerOrInfinity(depth)
        // 4.a. Set depthNum to ToIntegerOrInfinity(depth)
        // 4.b. If depthNum < 0, set depthNum to 0
        let depth_num = match depth
            .unwrap_or(&default_depth)
            .to_integer_or_infinity(context)?
        {
            IntegerOrInfinity::Integer(i) if i < 0 => IntegerOrInfinity::Integer(0),
            num => num,
        };

        // 5. Let A be ArraySpeciesCreate(O, 0)
        let new_array = Self::new_array(context);

        // 6. Perform FlattenIntoArray(A, O, sourceLen, 0, depthNum)
        let len = Self::flatten_into_array(
            context,
            &new_array,
            &this,
            source_len,
            0,
            depth_num,
            &Value::undefined(),
            &Value::undefined(),
        )?;
        new_array.set_field("length", len.to_length(context)?, false, context)?;

        Ok(new_array)
    }

    /// `Array.prototype.flatMap( callback, [ thisArg ] )`
    ///
    /// This method returns a new array formed by applying a given callback function to
    /// each element of the array, and then flattening the result by one level. It is
    /// identical to a `map()` followed by a `flat()` of depth 1, but slightly more
    /// efficient than calling those two methods separately.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.flatMap
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flatMap
    pub(crate) fn flat_map(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        // 1. Let O be ToObject(this value)
        let o: Value = this.to_object(context)?.into();

        // 2. Let sourceLen be LengthOfArrayLike(O)
        let source_len = this.get_field("length", context)?.to_length(context)? as u32;

        // 3. If IsCallable(mapperFunction) is false, throw a TypeError exception
        let mapper_function = args.get(0).cloned().unwrap_or_else(Value::undefined);
        if !mapper_function.is_function() {
            return context.throw_type_error("flatMap mapper function is not callable");
        }
        let this_arg = args.get(1).cloned().unwrap_or(o);

        // 4. Let A be ArraySpeciesCreate(O, 0)
        let new_array = Self::new_array(context);

        // 5. Perform FlattenIntoArray(A, O, sourceLen, 0, 1, mapperFunction, thisArg)
        let depth = Value::Integer(1).to_integer_or_infinity(context)?;
        let len = Self::flatten_into_array(
            context,
            &new_array,
            this,
            source_len,
            0,
            depth,
            &mapper_function,
            &this_arg,
        )?;
        new_array.set_field("length", len.to_length(context)?, false, context)?;

        // 6. Return A
        Ok(new_array)
    }

    /// Abstract method `FlattenIntoArray`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-flattenintoarray
    #[allow(clippy::too_many_arguments)]
    fn flatten_into_array(
        context: &mut Context,
        target: &Value,
        source: &Value,
        source_len: u32,
        start: u32,
        depth: IntegerOrInfinity,
        mapper_function: &Value,
        this_arg: &Value,
    ) -> Result<Value> {
        // 1. Assert target is Object
        debug_assert!(target.is_object());

        // 2. Assert source is Object
        debug_assert!(source.is_object());

        // 3. Assert if mapper_function is present, then:
        // - IsCallable(mapper_function) is true
        // - thisArg is present
        // - depth is 1

        // 4. Let targetIndex be start
        let mut target_index = start;

        // 5. Let sourceIndex be 0
        let mut source_index = 0;

        // 6. Repeat, while R(sourceIndex) < sourceLen
        while source_index < source_len {
            // a. Let P be ToString(sourceIndex)
            // b. Let exists be HasProperty(source, P)
            // c. If exists is true, then
            if source.has_field(source_index) {
                // i. Let element be Get(source, P)
                let mut element = source.get_field(source_index, context)?;

                // ii. If mapperFunction is present, then
                if !mapper_function.is_undefined() {
                    // 1. Set element to Call(mapperFunction, thisArg, <<element, sourceIndex, source>>)
                    let args = [element, Value::from(source_index), target.clone()];
                    element = context.call(mapper_function, this_arg, &args)?;
                }
                let element_as_object = element.as_object();

                // iii. Let shouldFlatten be false
                let mut should_flatten = false;

                // iv. If depth > 0, then
                let depth_is_positive = match depth {
                    IntegerOrInfinity::PositiveInfinity => true,
                    IntegerOrInfinity::NegativeInfinity => false,
                    IntegerOrInfinity::Integer(i) => i > 0,
                };
                if depth_is_positive {
                    // 1. Set shouldFlatten is IsArray(element)
                    should_flatten = match element_as_object {
                        Some(obj) => obj.is_array(),
                        _ => false,
                    };
                }
                // v. If shouldFlatten is true
                if should_flatten {
                    // 1. If depth is +Infinity let newDepth be +Infinity
                    // 2. Else, let newDepth be depth - 1
                    let new_depth = match depth {
                        IntegerOrInfinity::PositiveInfinity => IntegerOrInfinity::PositiveInfinity,
                        IntegerOrInfinity::Integer(d) => IntegerOrInfinity::Integer(d - 1),
                        IntegerOrInfinity::NegativeInfinity => IntegerOrInfinity::NegativeInfinity,
                    };

                    // 3. Let elementLen be LengthOfArrayLike(element)
                    let element_len =
                        element.get_field("length", context)?.to_length(context)? as u32;

                    // 4. Set targetIndex to FlattenIntoArray(target, element, elementLen, targetIndex, newDepth)
                    target_index = Self::flatten_into_array(
                        context,
                        target,
                        &element,
                        element_len,
                        target_index,
                        new_depth,
                        &Value::undefined(),
                        &Value::undefined(),
                    )?
                    .to_u32(context)?;

                // vi. Else
                } else {
                    // 1. If targetIndex >= 2^53 - 1, throw a TypeError exception
                    if target_index.to_f64().ok_or(0)? >= Number::MAX_SAFE_INTEGER {
                        return context
                            .throw_type_error("Target index exceeded max safe integer value");
                    }

                    // 2. Perform CreateDataPropertyOrThrow(target, targetIndex, element)
                    target
                        .set_property(target_index, DataDescriptor::new(element, Attribute::all()));

                    // 3. Set targetIndex to targetIndex + 1
                    target_index = target_index.saturating_add(1);
                }
            }
            // d. Set sourceIndex to sourceIndex + 1
            source_index = source_index.saturating_add(1);
        }

        // 7. Return targetIndex
        Ok(Value::Integer(target_index.try_into().unwrap_or(0)))
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

        let from_index = args.get(1).cloned().unwrap_or_else(|| Value::from(0));

        let length = this.get_field("length", context)?.to_length(context)?;

        if length == 0 {
            return Ok(Value::from(false));
        }

        let n = match from_index.to_integer_or_infinity(context)? {
            IntegerOrInfinity::NegativeInfinity => 0,
            IntegerOrInfinity::PositiveInfinity => return Ok(Value::from(false)),
            IntegerOrInfinity::Integer(i) => i,
        };

        let k = match n {
            num if num >= 0 => num as usize,    // if n>=0 -> k=n
            num if -num as usize > length => 0, // if n<0 -> k= max(length + n, 0)
            _ => length - (-n as usize), // this is `length + n` but is necessary for typing reasons
        };

        for idx in k..length {
            let check_element = this.get_field(idx, context)?.clone();

            if Value::same_value_zero(&check_element, &search_element) {
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
        new_array.set_field("length", Value::from(new_array_len), true, context)?;
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
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;

        // 2. Let len be ? LengthOfArrayLike(O).
        let length = o.length_of_array_like(context)?;

        // 3. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback = args
            .get(0)
            .map(|a| a.to_object(context))
            .transpose()?
            .ok_or_else(|| {
                context.construct_type_error(
                    "missing argument 0 when calling function Array.prototype.filter",
                )
            })?;
        let this_val = args.get(1).cloned().unwrap_or_else(Value::undefined);

        if !callback.is_callable() {
            return context.throw_type_error("the callback must be callable");
        }

        // 4. Let A be ? ArraySpeciesCreate(O, 0).
        let a = Self::array_species_create(&o, 0, context)?;

        // 5. Let k be 0.
        // 6. Let to be 0.
        let mut to = 0u32;
        // 7. Repeat, while k < len,
        for idx in 0..length {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kPresent be ? HasProperty(O, Pk).
            // c. If kPresent is true, then
            if o.has_property(idx, context)? {
                // i. Let kValue be ? Get(O, Pk).
                let element = o.get(idx, context)?;

                let args = [element.clone(), Value::from(idx), Value::from(o.clone())];

                // ii. Let selected be ! ToBoolean(? Call(callbackfn, thisArg, « kValue, 𝔽(k), O »)).
                let selected = callback.call(&this_val, &args, context)?.to_boolean();

                // iii. If selected is true, then
                if selected {
                    // 1. Perform ? CreateDataPropertyOrThrow(A, ! ToString(𝔽(to)), kValue).
                    a.create_data_property_or_throw(to, element, context)?;
                    // 2. Set to to to + 1.
                    to += 1;
                }
            }
        }

        // 8. Return A.
        Ok(a.into())
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
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;
        // 2. Let len be ? LengthOfArrayLike(O).
        let len = o.length_of_array_like(context)?;
        // 3. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback = if let Some(arg) = args
            .get(0)
            .and_then(Value::as_object)
            .filter(GcObject::is_callable)
        {
            arg
        } else {
            return context.throw_type_error("Array.prototype.some: callback is not callable");
        };

        // 4. Let k be 0.
        // 5. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kPresent be ? HasProperty(O, Pk).
            let k_present = o.has_property(k, context)?;
            // c. If kPresent is true, then
            if k_present {
                // i. Let kValue be ? Get(O, Pk).
                let k_value = o.get(k, context)?;
                // ii. Let testResult be ! ToBoolean(? Call(callbackfn, thisArg, « kValue, 𝔽(k), O »)).
                let this_arg = args.get(1).cloned().unwrap_or_default();
                let test_result = callback
                    .call(&this_arg, &[k_value, k.into(), o.clone().into()], context)?
                    .to_boolean();
                // iii. If testResult is true, return true.
                if test_result {
                    return Ok(Value::from(true));
                }
            }
            // d. Set k to k + 1.
        }
        // 6. Return false.
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
                accumulator = context.call(callback, &Value::undefined(), &arguments)?;
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
                accumulator = context.call(callback, &Value::undefined(), &arguments)?;
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

    /// `Array.prototype.copyWithin ( target, start [ , end ] )`
    ///
    /// The copyWithin() method shallow copies part of an array to another location
    /// in the same array and returns it without modifying its length.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array.prototype.copywithin
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/copyWithin
    pub(crate) fn copy_within(
        this: &Value,
        args: &[Value],
        context: &mut Context,
    ) -> Result<Value> {
        enum Direction {
            Forward,
            Backward,
        }
        let this: Value = this.to_object(context)?.into();

        let length = this.get_field("length", context)?.to_length(context)?;

        let mut to = Self::get_relative_start(context, args.get(0), length)?;
        let mut from = Self::get_relative_start(context, args.get(1), length)?;
        let finale = Self::get_relative_end(context, args.get(2), length)?;

        // saturating sub accounts for the case from > finale, which would cause an overflow
        // can skip the check for length - to, because we assert to <= length in get_relative_start
        let count = (finale.saturating_sub(from)).min(length - to);

        let direction = if from < to && to < from + count {
            from += count - 1;
            to += count - 1;
            Direction::Backward
        } else {
            Direction::Forward
        };

        // the original spec uses a while-loop from count to 1,
        // but count is not used inside the loop, so we can safely replace it
        // with a for-loop from 0 to count - 1
        for _ in 0..count {
            if this.has_field(from) {
                let val = this.get_field(from, context)?;
                this.set_field(to, val, true, context)?;
            } else {
                this.remove_property(to);
            }
            match direction {
                Direction::Forward => {
                    from += 1;
                    to += 1;
                }
                Direction::Backward => {
                    from -= 1;
                    to -= 1;
                }
            }
        }

        Ok(this)
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
