//! This module implements the global `TypedArray` objects.
//!
//! A `TypedArray` object describes an array-like view of an underlying binary data buffer.
//! There is no global property named `TypedArray`, nor is there a directly visible `TypedArray` constructor.
//! Instead, there are a number of different global properties,
//! whose values are typed array constructors for specific element types.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-typedarray-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray

use crate::{
    builtins::{
        array_buffer::{ArrayBuffer, SharedMemoryOrder},
        iterable::iterable_to_list,
        typed_array::integer_indexed_object::{ContentType, IntegerIndexed},
        Array, ArrayIterator, BuiltIn, JsArgs,
    },
    context::intrinsics::{StandardConstructor, StandardConstructors},
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        JsObject, ObjectData,
    },
    property::{Attribute, PropertyNameKind},
    symbol::WellKnownSymbols,
    value::{IntegerOrInfinity, JsValue},
    Context, JsResult, JsString,
};
use boa_gc::{unsafe_empty_trace, Finalize, Trace};
use boa_profiler::Profiler;
use num_traits::{Signed, Zero};
use std::cmp::Ordering;

use tap::{Conv, Pipe};

pub mod integer_indexed_object;

macro_rules! typed_array {
    ($ty:ident, $variant:ident, $name:literal, $global_object_name:ident) => {
        #[doc = concat!("JavaScript `", $name, "` built-in implementation.")]
        #[derive(Debug, Clone, Copy)]
        pub struct $ty;

        impl BuiltIn for $ty {
            const NAME: &'static str = $name;

            const ATTRIBUTE: Attribute = Attribute::WRITABLE
                .union(Attribute::NON_ENUMERABLE)
                .union(Attribute::CONFIGURABLE);

            fn init(context: &mut Context) -> Option<JsValue> {
                let _timer = Profiler::global().start_event(Self::NAME, "init");

                let typed_array_constructor = context
                    .intrinsics()
                    .constructors()
                    .typed_array()
                    .constructor();
                let typed_array_constructor_proto = context
                    .intrinsics()
                    .constructors()
                    .typed_array()
                    .prototype();

                let get_species = FunctionBuilder::native(context, TypedArray::get_species)
                    .name("get [Symbol.species]")
                    .constructor(false)
                    .build();

                ConstructorBuilder::with_standard_constructor(
                    context,
                    Self::constructor,
                    context
                        .intrinsics()
                        .constructors()
                        .$global_object_name()
                        .clone(),
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
                    "BYTES_PER_ELEMENT",
                    TypedArrayKind::$variant.element_size(),
                    Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
                )
                .static_property(
                    "BYTES_PER_ELEMENT",
                    TypedArrayKind::$variant.element_size(),
                    Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
                )
                .custom_prototype(typed_array_constructor)
                .inherit(typed_array_constructor_proto)
                .build()
                .conv::<JsValue>()
                .pipe(Some)
            }
        }

        impl $ty {
            const LENGTH: usize = 3;

            /// `23.2.5.1 TypedArray ( ...args )`
            ///
            /// More information:
            ///  - [ECMAScript reference][spec]
            ///
            /// [spec]: https://tc39.es/ecma262/#sec-typedarray
            fn constructor(
                new_target: &JsValue,
                args: &[JsValue],
                context: &mut Context,
            ) -> JsResult<JsValue> {
                // 1. If NewTarget is undefined, throw a TypeError exception.
                if new_target.is_undefined() {
                    return context.throw_type_error(concat!(
                        "new target was undefined when constructing an ",
                        $name
                    ));
                }

                // 2. Let constructorName be the String value of the Constructor Name value specified in Table 72 for this TypedArray constructor.
                let constructor_name = TypedArrayKind::$variant;

                // 3. Let proto be "%TypedArray.prototype%".
                let proto = StandardConstructors::$global_object_name;

                // 4. Let numberOfArgs be the number of elements in args.
                let number_of_args = args.len();

                // 5. If numberOfArgs = 0, then
                if number_of_args == 0 {
                    // a. Return ? AllocateTypedArray(constructorName, NewTarget, proto, 0).
                    return Ok(TypedArray::allocate(
                        constructor_name,
                        new_target,
                        proto,
                        Some(0),
                        context,
                    )?
                    .into());
                }
                // 6. Else,

                // a. Let firstArgument be args[0].
                let first_argument = &args[0];

                // b. If Type(firstArgument) is Object, then
                if let Some(first_argument) = first_argument.as_object() {
                    // i. Let O be ? AllocateTypedArray(constructorName, NewTarget, proto).
                    let o =
                        TypedArray::allocate(constructor_name, new_target, proto, None, context)?;

                    // ii. If firstArgument has a [[TypedArrayName]] internal slot, then
                    if first_argument.is_typed_array() {
                        // 1. Perform ? InitializeTypedArrayFromTypedArray(O, firstArgument).
                        TypedArray::initialize_from_typed_array(&o, first_argument, context)?;
                    } else if first_argument.is_array_buffer() {
                        // iii. Else if firstArgument has an [[ArrayBufferData]] internal slot, then

                        // 1. If numberOfArgs > 1, let byteOffset be args[1]; else let byteOffset be undefined.
                        let byte_offset = args.get_or_undefined(1);

                        // 2. If numberOfArgs > 2, let length be args[2]; else let length be undefined.
                        let length = args.get_or_undefined(2);

                        // 3. Perform ? InitializeTypedArrayFromArrayBuffer(O, firstArgument, byteOffset, length).
                        TypedArray::initialize_from_array_buffer(
                            &o,
                            first_argument.clone(),
                            byte_offset,
                            length,
                            context,
                        )?;
                    } else {
                        // iv. Else,

                        // 1. Assert: Type(firstArgument) is Object and firstArgument does not have
                        // either a [[TypedArrayName]] or an [[ArrayBufferData]] internal slot.

                        // 2. Let usingIterator be ? GetMethod(firstArgument, @@iterator).

                        let first_argument_v = JsValue::from(first_argument.clone());
                        let using_iterator =
                            first_argument_v.get_method(WellKnownSymbols::replace(), context)?;

                        // 3. If usingIterator is not undefined, then
                        if let Some(using_iterator) = using_iterator {
                            // a. Let values be ? IterableToList(firstArgument, usingIterator).
                            let values = iterable_to_list(
                                context,
                                &first_argument_v,
                                Some(using_iterator.into()),
                            )?;

                            // b. Perform ? InitializeTypedArrayFromList(O, values).
                            TypedArray::initialize_from_list(&o, values, context)?;
                        } else {
                            // 4. Else,

                            // a. NOTE: firstArgument is not an Iterable so assume it is already an array-like object.
                            // b. Perform ? InitializeTypedArrayFromArrayLike(O, firstArgument).
                            TypedArray::initialize_from_array_like(&o, &first_argument, context)?;
                        }
                    }

                    // v. Return O.
                    Ok(o.into())
                } else {
                    // c. Else,

                    // i. Assert: firstArgument is not an Object.
                    assert!(!first_argument.is_object(), "firstArgument was an object");

                    // ii. Let elementLength be ? ToIndex(firstArgument).
                    let element_length = first_argument.to_index(context)?;

                    // iii. Return ? AllocateTypedArray(constructorName, NewTarget, proto, elementLength).
                    Ok(TypedArray::allocate(
                        constructor_name,
                        new_target,
                        proto,
                        Some(element_length),
                        context,
                    )?
                    .into())
                }
            }
        }
    };
}

/// The JavaScript `%TypedArray%` object.
///
/// <https://tc39.es/ecma262/#sec-%typedarray%-intrinsic-object>
#[derive(Debug, Clone, Copy)]
pub(crate) struct TypedArray;

impl BuiltIn for TypedArray {
    const NAME: &'static str = "TypedArray";
    fn init(context: &mut Context) -> Option<JsValue> {
        let get_species = FunctionBuilder::native(context, Self::get_species)
            .name("get [Symbol.species]")
            .constructor(false)
            .build();

        let get_buffer = FunctionBuilder::native(context, Self::buffer)
            .name("get buffer")
            .constructor(false)
            .build();

        let get_byte_length = FunctionBuilder::native(context, Self::byte_length)
            .name("get byteLength")
            .constructor(false)
            .build();

        let get_byte_offset = FunctionBuilder::native(context, Self::byte_offset)
            .name("get byteOffset")
            .constructor(false)
            .build();

        let get_length = FunctionBuilder::native(context, Self::length)
            .name("get length")
            .constructor(false)
            .build();

        let get_to_string_tag = FunctionBuilder::native(context, Self::to_string_tag)
            .name("get [Symbol.toStringTag]")
            .constructor(false)
            .build();

        let values_function = FunctionBuilder::native(context, Self::values)
            .name("values")
            .length(0)
            .constructor(false)
            .build();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().typed_array().clone(),
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
            WellKnownSymbols::iterator(),
            values_function,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .accessor(
            "buffer",
            Some(get_buffer),
            None,
            Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .accessor(
            "byteLength",
            Some(get_byte_length),
            None,
            Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .accessor(
            "byteOffset",
            Some(get_byte_offset),
            None,
            Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .accessor(
            "length",
            Some(get_length),
            None,
            Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .accessor(
            WellKnownSymbols::to_string_tag(),
            Some(get_to_string_tag),
            None,
            Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
        )
        .static_method(Self::from, "from", 1)
        .static_method(Self::of, "of", 0)
        .method(Self::at, "at", 1)
        .method(Self::copy_within, "copyWithin", 2)
        .method(Self::entries, "entries", 0)
        .method(Self::every, "every", 1)
        .method(Self::fill, "fill", 1)
        .method(Self::filter, "filter", 1)
        .method(Self::find, "find", 1)
        .method(Self::findindex, "findIndex", 1)
        .method(Self::foreach, "forEach", 1)
        .method(Self::includes, "includes", 1)
        .method(Self::index_of, "indexOf", 1)
        .method(Self::join, "join", 1)
        .method(Self::keys, "keys", 0)
        .method(Self::last_index_of, "lastIndexOf", 1)
        .method(Self::map, "map", 1)
        .method(Self::reduce, "reduce", 1)
        .method(Self::reduceright, "reduceRight", 1)
        .method(Self::reverse, "reverse", 0)
        .method(Self::set, "set", 1)
        .method(Self::slice, "slice", 2)
        .method(Self::some, "some", 1)
        .method(Self::sort, "sort", 1)
        .method(Self::subarray, "subarray", 2)
        .method(Self::values, "values", 0)
        // 23.2.3.29 %TypedArray%.prototype.toString ( )
        // The initial value of the %TypedArray%.prototype.toString data property is the same
        // built-in function object as the Array.prototype.toString method defined in 23.1.3.30.
        .method(Array::to_string, "toString", 0)
        .build();

        None
    }
}
impl TypedArray {
    const LENGTH: usize = 0;

    /// `23.2.1.1 %TypedArray% ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%
    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Throw a TypeError exception.
        context.throw_type_error("the TypedArray constructor should never be called directly")
    }

    /// `23.2.2.1 %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.from
    fn from(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let C be the this value.
        // 2. If IsConstructor(C) is false, throw a TypeError exception.
        let constructor = match this.as_object() {
            Some(obj) if obj.is_constructor() => obj,
            _ => {
                return context
                    .throw_type_error("TypedArray.from called on non-constructable value")
            }
        };

        let mapping = match args.get(1) {
            // 3. If mapfn is undefined, let mapping be false.
            None | Some(JsValue::Undefined) => None,
            // 4. Else,
            Some(v) => match v.as_object() {
                // b. Let mapping be true.
                Some(obj) if obj.is_callable() => Some(obj),
                // a. If IsCallable(mapfn) is false, throw a TypeError exception.
                _ => {
                    return context
                        .throw_type_error("TypedArray.from called with non-callable mapfn")
                }
            },
        };

        // 5. Let usingIterator be ? GetMethod(source, @@iterator).
        let source = args.get_or_undefined(0);
        let using_iterator = source.get_method(WellKnownSymbols::iterator(), context)?;

        let this_arg = args.get_or_undefined(2);

        // 6. If usingIterator is not undefined, then
        if let Some(using_iterator) = using_iterator {
            // a. Let values be ? IterableToList(source, usingIterator).
            let values = iterable_to_list(context, source, Some(using_iterator.into()))?;

            // b. Let len be the number of elements in values.
            // c. Let targetObj be ? TypedArrayCreate(C, « 𝔽(len) »).
            let target_obj = Self::create(constructor, &[values.len().into()], context)?;

            // d. Let k be 0.
            // e. Repeat, while k < len,
            for (k, k_value) in values.iter().enumerate() {
                // i. Let Pk be ! ToString(𝔽(k)).
                // ii. Let kValue be the first element of values and remove that element from values.
                // iii. If mapping is true, then
                let mapped_value = if let Some(map_fn) = &mapping {
                    // 1. Let mappedValue be ? Call(mapfn, thisArg, « kValue, 𝔽(k) »).
                    map_fn.call(this_arg, &[k_value.clone(), k.into()], context)?
                }
                // iv. Else, let mappedValue be kValue.
                else {
                    k_value.clone()
                };

                // v. Perform ? Set(targetObj, Pk, mappedValue, true).
                target_obj.set(k, mapped_value, true, context)?;
            }

            // f. Assert: values is now an empty List.
            // g. Return targetObj.
            return Ok(target_obj.into());
        }

        // 7. NOTE: source is not an Iterable so assume it is already an array-like object.
        // 8. Let arrayLike be ! ToObject(source).
        let array_like = source
            .to_object(context)
            .expect("ToObject cannot fail here");

        // 9. Let len be ? LengthOfArrayLike(arrayLike).
        let len = array_like.length_of_array_like(context)?;

        // 10. Let targetObj be ? TypedArrayCreate(C, « 𝔽(len) »).
        let target_obj = Self::create(constructor, &[len.into()], context)?;

        // 11. Let k be 0.
        // 12. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ? Get(arrayLike, Pk).
            let k_value = array_like.get(k, context)?;

            // c. If mapping is true, then
            let mapped_value = if let Some(map_fn) = &mapping {
                // i. Let mappedValue be ? Call(mapfn, thisArg, « kValue, 𝔽(k) »).
                map_fn.call(this_arg, &[k_value, k.into()], context)?
            }
            // d. Else, let mappedValue be kValue.
            else {
                k_value
            };

            // e. Perform ? Set(targetObj, Pk, mappedValue, true).
            target_obj.set(k, mapped_value, true, context)?;
        }

        // 13. Return targetObj.
        Ok(target_obj.into())
    }

    /// `23.2.2.2 %TypedArray%.of ( ...items )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.of
    fn of(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let len be the number of elements in items.

        // 2. Let C be the this value.
        // 3. If IsConstructor(C) is false, throw a TypeError exception.
        let constructor = match this.as_object() {
            Some(obj) if obj.is_constructor() => obj,
            _ => {
                return context.throw_type_error("TypedArray.of called on non-constructable value")
            }
        };

        // 4. Let newObj be ? TypedArrayCreate(C, « 𝔽(len) »).
        let new_obj = Self::create(constructor, &[args.len().into()], context)?;

        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for (k, k_value) in args.iter().enumerate() {
            // a. Let kValue be items[k].
            // b. Let Pk be ! ToString(𝔽(k)).
            // c. Perform ? Set(newObj, Pk, kValue, true).
            new_obj.set(k, k_value, true, context)?;
        }

        // 7. Return newObj.
        Ok(new_obj.into())
    }

    /// `23.2.2.4 get %TypedArray% [ @@species ]`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-%typedarray%-@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `23.2.3.1 %TypedArray%.prototype.at ( index )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.at
    fn at(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        // 4. Let relativeIndex be ? ToIntegerOrInfinity(index).
        let relative_index = args.get_or_undefined(0).to_integer_or_infinity(context)?;

        let k = match relative_index {
            // Note: Early undefined return on infinity.
            IntegerOrInfinity::PositiveInfinity | IntegerOrInfinity::NegativeInfinity => {
                return Ok(JsValue::undefined())
            }
            // 5. If relativeIndex ≥ 0, then
            // a. Let k be relativeIndex.
            IntegerOrInfinity::Integer(i) if i >= 0 => i,
            // 6. Else,
            // a. Let k be len + relativeIndex.
            IntegerOrInfinity::Integer(i) => len + i,
        };

        // 7. If k < 0 or k ≥ len, return undefined.
        if k < 0 || k >= len {
            return Ok(JsValue::undefined());
        }

        // 8. Return ! Get(O, ! ToString(𝔽(k))).
        Ok(obj.get(k, context).expect("Get cannot fail here"))
    }

    /// `23.2.3.2 get %TypedArray%.prototype.buffer`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-%typedarray%.prototype.buffer
    fn buffer(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let typed_array = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;

        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        // 5. Return buffer.
        Ok(typed_array
            .viewed_array_buffer()
            .map_or_else(JsValue::undefined, |buffer| buffer.clone().into()))
    }

    /// `23.2.3.3 get %TypedArray%.prototype.byteLength`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-%typedarray%.prototype.bytelength
    fn byte_length(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let typed_array = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;

        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        // 5. If IsDetachedBuffer(buffer) is true, return +0𝔽.
        // 6. Let size be O.[[ByteLength]].
        // 7. Return 𝔽(size).
        if typed_array.is_detached() {
            Ok(0.into())
        } else {
            Ok(typed_array.byte_length().into())
        }
    }

    /// `23.2.3.4 get %TypedArray%.prototype.byteOffset`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-%typedarray%.prototype.byteoffset
    fn byte_offset(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let typed_array = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;

        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        // 5. If IsDetachedBuffer(buffer) is true, return +0𝔽.
        // 6. Let offset be O.[[ByteOffset]].
        // 7. Return 𝔽(offset).
        if typed_array.is_detached() {
            Ok(0.into())
        } else {
            Ok(typed_array.byte_offset().into())
        }
    }

    /// `23.2.3.6 %TypedArray%.prototype.copyWithin ( target, start [ , end ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.copywithin
    fn copy_within(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;

        let len = {
            let obj_borrow = obj.borrow();
            let o = obj_borrow
                .as_typed_array()
                .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;

            // 2. Perform ? ValidateTypedArray(O).
            if o.is_detached() {
                return context.throw_type_error("Buffer of the typed array is detached");
            }

            // 3. Let len be O.[[ArrayLength]].
            o.array_length() as i64
        };

        // 4. Let relativeTarget be ? ToIntegerOrInfinity(target).
        let relative_target = args.get_or_undefined(0).to_integer_or_infinity(context)?;

        let to = match relative_target {
            // 5. If relativeTarget is -∞, let to be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 6. Else if relativeTarget < 0, let to be max(len + relativeTarget, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(len + i, 0),
            // 7. Else, let to be min(relativeTarget, len).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 8. Let relativeStart be ? ToIntegerOrInfinity(start).
        let relative_start = args.get_or_undefined(1).to_integer_or_infinity(context)?;

        let from = match relative_start {
            // 9. If relativeStart is -∞, let from be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 10. Else if relativeStart < 0, let from be max(len + relativeStart, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(len + i, 0),
            // 11. Else, let from be min(relativeStart, len).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 12. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
        let end = args.get_or_undefined(2);
        let relative_end = if end.is_undefined() {
            IntegerOrInfinity::Integer(len)
        } else {
            end.to_integer_or_infinity(context)?
        };

        let r#final = match relative_end {
            // 13. If relativeEnd is -∞, let final be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 14. Else if relativeEnd < 0, let final be max(len + relativeEnd, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(len + i, 0),
            // 15. Else, let final be min(relativeEnd, len).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 16. Let count be min(final - from, len - to).
        let count = std::cmp::min(r#final - from, len - to);

        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;

        // 17. If count > 0, then
        if count > 0 {
            // a. NOTE: The copying must be performed in a manner that preserves the bit-level encoding of the source data.
            // b. Let buffer be O.[[ViewedArrayBuffer]].
            // c. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
            if o.is_detached() {
                return context.throw_type_error("Buffer of the typed array is detached");
            }

            // d. Let typedArrayName be the String value of O.[[TypedArrayName]].
            let typed_array_name = o.typed_array_name();

            // e. Let elementSize be the Element Size value specified in Table 73 for typedArrayName.
            let element_size = typed_array_name.element_size() as i64;

            // f. Let byteOffset be O.[[ByteOffset]].
            let byte_offset = o.byte_offset() as i64;

            // g. Let toByteIndex be to × elementSize + byteOffset.
            let mut to_byte_index = to * element_size + byte_offset;

            // h. Let fromByteIndex be from × elementSize + byteOffset.
            let mut from_byte_index = from * element_size + byte_offset;

            // i. Let countBytes be count × elementSize.
            let mut count_bytes = count * element_size;

            // j. If fromByteIndex < toByteIndex and toByteIndex < fromByteIndex + countBytes, then
            let direction = if from_byte_index < to_byte_index
                && to_byte_index < from_byte_index + count_bytes
            {
                // ii. Set fromByteIndex to fromByteIndex + countBytes - 1.
                from_byte_index = from_byte_index + count_bytes - 1;

                // iii. Set toByteIndex to toByteIndex + countBytes - 1.
                to_byte_index = to_byte_index + count_bytes - 1;

                // i. Let direction be -1.
                -1
            }
            // k. Else,
            else {
                // i. Let direction be 1.
                1
            };

            let buffer_obj = o
                .viewed_array_buffer()
                .expect("Already checked for detached buffer");
            let mut buffer_obj_borrow = buffer_obj.borrow_mut();
            let buffer = buffer_obj_borrow
                .as_array_buffer_mut()
                .expect("Already checked for detached buffer");

            // l. Repeat, while countBytes > 0,
            while count_bytes > 0 {
                // i. Let value be GetValueFromBuffer(buffer, fromByteIndex, Uint8, true, Unordered).
                let value = buffer.get_value_from_buffer(
                    from_byte_index as usize,
                    TypedArrayKind::Uint8,
                    true,
                    SharedMemoryOrder::Unordered,
                    None,
                );

                // ii. Perform SetValueInBuffer(buffer, toByteIndex, Uint8, value, true, Unordered).
                buffer.set_value_in_buffer(
                    to_byte_index as usize,
                    TypedArrayKind::Uint8,
                    &value,
                    SharedMemoryOrder::Unordered,
                    None,
                    context,
                )?;

                // iii. Set fromByteIndex to fromByteIndex + direction.
                from_byte_index += direction;

                // iv. Set toByteIndex to toByteIndex + direction.
                to_byte_index += direction;

                // v. Set countBytes to countBytes - 1.
                count_bytes -= 1;
            }
        }

        // 18. Return O.
        Ok(this.clone())
    }

    /// `23.2.3.7 %TypedArray%.prototype.entries ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.entries
    fn entries(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let o = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.borrow()
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?
            .is_detached()
        {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Return CreateArrayIterator(O, key+value).
        Ok(ArrayIterator::create_array_iterator(
            o.clone(),
            PropertyNameKind::KeyAndValue,
            context,
        ))
    }

    /// `23.2.3.8 %TypedArray%.prototype.every ( callbackfn [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.every
    fn every(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => {
                return context.throw_type_error(
                    "TypedArray.prototype.every called with non-callable callback function",
                )
            }
        };

        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ! Get(O, Pk).
            let k_value = obj.get(k, context)?;

            // c. Let testResult be ! ToBoolean(? Call(callbackfn, thisArg, « kValue, 𝔽(k), O »)).
            let test_result = callback_fn
                .call(
                    args.get_or_undefined(1),
                    &[k_value, k.into(), this.clone()],
                    context,
                )?
                .to_boolean();

            // d. If testResult is false, return false.
            if !test_result {
                return Ok(false.into());
            }
        }

        // 7. Return true.
        Ok(true.into())
    }

    /// `23.2.3.9 %TypedArray%.prototype.fill ( value [ , start [ , end ] ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.fill
    fn fill(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        // 4. If O.[[ContentType]] is BigInt, set value to ? ToBigInt(value).
        let value: JsValue = if o.typed_array_name().content_type() == ContentType::BigInt {
            args.get_or_undefined(0).to_bigint(context)?.into()
        // 5. Otherwise, set value to ? ToNumber(value).
        } else {
            args.get_or_undefined(0).to_number(context)?.into()
        };

        // 6. Let relativeStart be ? ToIntegerOrInfinity(start).
        let mut k = match args.get_or_undefined(1).to_integer_or_infinity(context)? {
            // 7. If relativeStart is -∞, let k be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 8. Else if relativeStart < 0, let k be max(len + relativeStart, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(len + i, 0),
            // 9. Else, let k be min(relativeStart, len).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 10. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
        let end = args.get_or_undefined(2);
        let relative_end = if end.is_undefined() {
            IntegerOrInfinity::Integer(len)
        } else {
            end.to_integer_or_infinity(context)?
        };

        let r#final = match relative_end {
            // 11. If relativeEnd is -∞, let final be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 12. Else if relativeEnd < 0, let final be max(len + relativeEnd, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(len + i, 0),
            // 13. Else, let final be min(relativeEnd, len).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 14. If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 15. Repeat, while k < final,
        while k < r#final {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Perform ! Set(O, Pk, value, true).
            obj.set(k, value.clone(), true, context)
                .expect("Set cannot fail here");

            // c. Set k to k + 1.
            k += 1;
        }

        // 16. Return O.
        Ok(this.clone())
    }

    /// `23.2.3.10 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.filter
    fn filter(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => {
                return context.throw_type_error(
                    "TypedArray.prototype.filter called with non-callable callback function",
                )
            }
        };

        // 5. Let kept be a new empty List.
        let mut kept = Vec::new();

        // 6. Let k be 0.
        // 7. Let captured be 0.
        let mut captured = 0;

        // 8. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ! Get(O, Pk).
            let k_value = obj.get(k, context).expect("Get cannot fail here");

            // c. Let selected be ! ToBoolean(? Call(callbackfn, thisArg, « kValue, 𝔽(k), O »)).#
            let selected = callback_fn
                .call(
                    args.get_or_undefined(1),
                    &[k_value.clone(), k.into(), this.clone()],
                    context,
                )?
                .to_boolean();

            // d. If selected is true, then
            if selected {
                // i. Append kValue to the end of kept.
                kept.push(k_value);

                // ii. Set captured to captured + 1.
                captured += 1;
            }
        }

        // 9. Let A be ? TypedArraySpeciesCreate(O, « 𝔽(captured) »).
        let a = Self::species_create(obj, o.typed_array_name(), &[captured.into()], context)?;

        // 10. Let n be 0.
        // 11. For each element e of kept, do
        for (n, e) in kept.iter().enumerate() {
            // a. Perform ! Set(A, ! ToString(𝔽(n)), e, true).
            a.set(n, e.clone(), true, context)
                .expect("Set cannot fail here");
            // b. Set n to n + 1.
        }

        // 12. Return A.
        Ok(a.into())
    }

    /// `23.2.3.11 %TypedArray%.prototype.find ( predicate [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.find
    fn find(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        // 4. If IsCallable(predicate) is false, throw a TypeError exception.
        let predicate = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => {
                return context.throw_type_error(
                    "TypedArray.prototype.find called with non-callable predicate function",
                )
            }
        };

        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ! Get(O, Pk).
            let k_value = obj.get(k, context).expect("Get cannot fail here");

            // c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
            // d. If testResult is true, return kValue.
            if predicate
                .call(
                    args.get_or_undefined(1),
                    &[k_value.clone(), k.into(), this.clone()],
                    context,
                )?
                .to_boolean()
            {
                return Ok(k_value);
            }
        }

        // 7. Return undefined.
        Ok(JsValue::undefined())
    }

    /// `23.2.3.12 %TypedArray%.prototype.findIndex ( predicate [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.findindex
    fn findindex(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        // 4. If IsCallable(predicate) is false, throw a TypeError exception.
        let predicate =
            match args.get_or_undefined(0).as_object() {
                Some(obj) if obj.is_callable() => obj,
                _ => return context.throw_type_error(
                    "TypedArray.prototype.findindex called with non-callable predicate function",
                ),
            };

        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ! Get(O, Pk).
            let k_value = obj.get(k, context).expect("Get cannot fail here");

            // c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
            // d. If testResult is true, return 𝔽(k).
            if predicate
                .call(
                    args.get_or_undefined(1),
                    &[k_value.clone(), k.into(), this.clone()],
                    context,
                )?
                .to_boolean()
            {
                return Ok(k.into());
            }
        }

        // 7. Return -1𝔽.
        Ok((-1).into())
    }

    /// `23.2.3.13 %TypedArray%.prototype.forEach ( callbackfn [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.foreach
    fn foreach(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => {
                return context.throw_type_error(
                    "TypedArray.prototype.foreach called with non-callable callback function",
                )
            }
        };

        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ! Get(O, Pk).
            let k_value = obj.get(k, context).expect("Get cannot fail here");

            // c. Perform ? Call(callbackfn, thisArg, « kValue, 𝔽(k), O »).
            callback_fn.call(
                args.get_or_undefined(1),
                &[k_value, k.into(), this.clone()],
                context,
            )?;
        }

        // 7. Return undefined.
        Ok(JsValue::undefined())
    }

    /// `23.2.3.14 %TypedArray%.prototype.includes ( searchElement [ , fromIndex ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.includes
    fn includes(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        // 4. If len is 0, return false.
        if len == 0 {
            return Ok(false.into());
        }

        // 5. Let n be ? ToIntegerOrInfinity(fromIndex).
        // 6. Assert: If fromIndex is undefined, then n is 0.
        let n = args.get_or_undefined(1).to_integer_or_infinity(context)?;

        let n = match n {
            // 7. If n is +∞, return false.
            IntegerOrInfinity::PositiveInfinity => return Ok(false.into()),
            // 8. Else if n is -∞, set n to 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            IntegerOrInfinity::Integer(i) => i,
        };

        // 9. If n ≥ 0, then
        let mut k = if n >= 0 {
            // a. Let k be n.
            n
        // 10. Else,
        } else {
            // a. Let k be len + n.
            // b. If k < 0, set k to 0.
            if len + n < 0 {
                0
            } else {
                len + n
            }
        };

        // 11. Repeat, while k < len,
        while k < len {
            // a. Let elementK be ! Get(O, ! ToString(𝔽(k))).
            let element_k = obj.get(k, context).expect("Get cannot fail here");

            // b. If SameValueZero(searchElement, elementK) is true, return true.
            if JsValue::same_value_zero(args.get_or_undefined(0), &element_k) {
                return Ok(true.into());
            }

            // c. Set k to k + 1.
            k += 1;
        }

        // 12. Return false.
        Ok(false.into())
    }

    /// `23.2.3.15 %TypedArray%.prototype.indexOf ( searchElement [ , fromIndex ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.indexof
    fn index_of(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        // 4. If len is 0, return -1𝔽.
        if len == 0 {
            return Ok((-1).into());
        }

        // 5. Let n be ? ToIntegerOrInfinity(fromIndex).
        // 6. Assert: If fromIndex is undefined, then n is 0.
        let n = args.get_or_undefined(1).to_integer_or_infinity(context)?;

        let n = match n {
            // 7. If n is +∞, return -1𝔽.
            IntegerOrInfinity::PositiveInfinity => return Ok((-1).into()),
            // 8. Else if n is -∞, set n to 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            IntegerOrInfinity::Integer(i) => i,
        };

        // 9. If n ≥ 0, then
        let mut k = if n >= 0 {
            // a. Let k be n.
            n
        // 10. Else,
        } else {
            // a. Let k be len + n.
            // b. If k < 0, set k to 0.
            if len + n < 0 {
                0
            } else {
                len + n
            }
        };

        // 11. Repeat, while k < len,
        while k < len {
            // a. Let kPresent be ! HasProperty(O, ! ToString(𝔽(k))).
            let k_present = obj
                .has_property(k, context)
                .expect("HasProperty cannot fail here");

            // b. If kPresent is true, then
            if k_present {
                // i. Let elementK be ! Get(O, ! ToString(𝔽(k))).
                let element_k = obj.get(k, context).expect("Get cannot fail here");

                // ii. Let same be IsStrictlyEqual(searchElement, elementK).
                // iii. If same is true, return 𝔽(k).
                if args.get_or_undefined(0).strict_equals(&element_k) {
                    return Ok(k.into());
                }
            }

            // c. Set k to k + 1.
            k += 1;
        }

        // 12. Return -1𝔽.
        Ok((-1).into())
    }

    /// `23.2.3.16 %TypedArray%.prototype.join ( separator )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.join
    fn join(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        // 4. If separator is undefined, let sep be the single-element String ",".
        let separator = args.get_or_undefined(0);
        let sep = if separator.is_undefined() {
            JsString::new(",")
        // 5. Else, let sep be ? ToString(separator).
        } else {
            separator.to_string(context)?
        };

        // 6. Let R be the empty String.
        let mut r = JsString::new("");

        // 7. Let k be 0.
        // 8. Repeat, while k < len,
        for k in 0..len {
            // a. If k > 0, set R to the string-concatenation of R and sep.
            if k > 0 {
                r = JsString::concat(r, sep.clone());
            }

            // b. Let element be ! Get(O, ! ToString(𝔽(k))).
            let element = obj.get(k, context).expect("Get cannot fail here");

            // c. If element is undefined, let next be the empty String; otherwise, let next be ! ToString(element).
            // d. Set R to the string-concatenation of R and next.
            if !element.is_undefined() {
                r = JsString::concat(r, element.to_string(context)?);
            }
        }

        // 9. Return R.
        Ok(r.into())
    }

    /// `23.2.3.17 %TypedArray%.prototype.keys ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.keys
    fn keys(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let o = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.borrow()
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?
            .is_detached()
        {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Return CreateArrayIterator(O, key).
        Ok(ArrayIterator::create_array_iterator(
            o.clone(),
            PropertyNameKind::Key,
            context,
        ))
    }

    /// `23.2.3.18 %TypedArray%.prototype.lastIndexOf ( searchElement [ , fromIndex ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.lastindexof
    fn last_index_of(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        // 4. If len is 0, return -1𝔽.
        if len == 0 {
            return Ok((-1).into());
        }

        // 5. If fromIndex is present, let n be ? ToIntegerOrInfinity(fromIndex); else let n be len - 1.
        let n = if let Some(n) = args.get(1) {
            n.to_integer_or_infinity(context)?
        } else {
            IntegerOrInfinity::Integer(len - 1)
        };

        let mut k = match n {
            // 6. If n is -∞, return -1𝔽.
            IntegerOrInfinity::NegativeInfinity => return Ok((-1).into()),
            // 7. If n ≥ 0, then
            // a. Let k be min(n, len - 1).
            IntegerOrInfinity::Integer(i) if i >= 0 => std::cmp::min(i, len - 1),
            IntegerOrInfinity::PositiveInfinity => len - 1,
            // 8. Else,
            // a. Let k be len + n.
            IntegerOrInfinity::Integer(i) => len + i,
        };

        // 9. Repeat, while k ≥ 0,
        while k >= 0 {
            // a. Let kPresent be ! HasProperty(O, ! ToString(𝔽(k))).
            let k_present = obj
                .has_property(k, context)
                .expect("HasProperty cannot fail here");

            // b. If kPresent is true, then
            if k_present {
                // i. Let elementK be ! Get(O, ! ToString(𝔽(k))).
                let element_k = obj.get(k, context).expect("Get cannot fail here");

                // ii. Let same be IsStrictlyEqual(searchElement, elementK).
                // iii. If same is true, return 𝔽(k).
                if args.get_or_undefined(0).strict_equals(&element_k) {
                    return Ok(k.into());
                }
            }

            // c. Set k to k - 1.
            k -= 1;
        }

        // 10. Return -1𝔽.
        Ok((-1).into())
    }

    /// `23.2.3.19 get %TypedArray%.prototype.length`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-%typedarray%.prototype.length
    fn length(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
        // 3. Assert: O has [[ViewedArrayBuffer]] and [[ArrayLength]] internal slots.
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let typed_array = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;

        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        // 5. If IsDetachedBuffer(buffer) is true, return +0𝔽.
        // 6. Let length be O.[[ArrayLength]].
        // 7. Return 𝔽(length).
        if typed_array.is_detached() {
            Ok(0.into())
        } else {
            Ok(typed_array.array_length().into())
        }
    }

    /// `23.2.3.20 %TypedArray%.prototype.map ( callbackfn [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.map
    fn map(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => {
                return context.throw_type_error(
                    "TypedArray.prototype.map called with non-callable callback function",
                )
            }
        };

        // 5. Let A be ? TypedArraySpeciesCreate(O, « 𝔽(len) »).
        let a = Self::species_create(obj, o.typed_array_name(), &[len.into()], context)?;

        // 6. Let k be 0.
        // 7. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ! Get(O, Pk).
            let k_value = obj.get(k, context).expect("Get cannot fail here");

            // c. Let mappedValue be ? Call(callbackfn, thisArg, « kValue, 𝔽(k), O »).
            let mapped_value = callback_fn.call(
                args.get_or_undefined(1),
                &[k_value, k.into(), this.clone()],
                context,
            )?;

            // d. Perform ? Set(A, Pk, mappedValue, true).
            a.set(k, mapped_value, true, context)?;
        }

        // 8. Return A.
        Ok(a.into())
    }

    /// `23.2.3.21 %TypedArray%.prototype.reduce ( callbackfn [ , initialValue ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.reduce
    fn reduce(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => {
                return context.throw_type_error(
                    "TypedArray.prototype.reduce called with non-callable callback function",
                )
            }
        };

        // 5. If len = 0 and initialValue is not present, throw a TypeError exception.
        if len == 0 && args.get(1).is_none() {
            return context
                .throw_type_error("Typed array length is 0 and initial value is not present");
        }

        // 6. Let k be 0.
        let mut k = 0;

        // 7. Let accumulator be undefined.
        // 8. If initialValue is present, then
        let mut accumulator = if let Some(initial_value) = args.get(1) {
            // a. Set accumulator to initialValue.
            initial_value.clone()
        // 9. Else,
        } else {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Set accumulator to ! Get(O, Pk).
            // c. Set k to k + 1.
            k += 1;
            obj.get(0, context).expect("Get cannot fail here")
        };

        // 10. Repeat, while k < len,
        while k < len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ! Get(O, Pk).
            let k_value = obj.get(k, context).expect("Get cannot fail here");

            // c. Set accumulator to ? Call(callbackfn, undefined, « accumulator, kValue, 𝔽(k), O »).
            accumulator = callback_fn.call(
                &JsValue::undefined(),
                &[accumulator, k_value, k.into(), this.clone()],
                context,
            )?;

            // d. Set k to k + 1.
            k += 1;
        }

        // 11. Return accumulator.
        Ok(accumulator)
    }

    /// `23.2.3.22 %TypedArray%.prototype.reduceRight ( callbackfn [ , initialValue ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.reduceright
    fn reduceright(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn =
            match args.get_or_undefined(0).as_object() {
                Some(obj) if obj.is_callable() => obj,
                _ => return context.throw_type_error(
                    "TypedArray.prototype.reduceright called with non-callable callback function",
                ),
            };

        // 5. If len = 0 and initialValue is not present, throw a TypeError exception.
        if len == 0 && args.get(1).is_none() {
            return context
                .throw_type_error("Typed array length is 0 and initial value is not present");
        }

        // 6. Let k be len - 1.
        let mut k = len - 1;

        // 7. Let accumulator be undefined.
        // 8. If initialValue is present, then
        let mut accumulator = if let Some(initial_value) = args.get(1) {
            // a. Set accumulator to initialValue.
            initial_value.clone()
        // 9. Else,
        } else {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Set accumulator to ! Get(O, Pk).
            let accumulator = obj.get(k, context).expect("Get cannot fail here");

            // c. Set k to k - 1.
            k -= 1;

            accumulator
        };

        // 10. Repeat, while k ≥ 0,
        while k >= 0 {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ! Get(O, Pk).
            let k_value = obj.get(k, context).expect("Get cannot fail here");

            // c. Set accumulator to ? Call(callbackfn, undefined, « accumulator, kValue, 𝔽(k), O »).
            accumulator = callback_fn.call(
                &JsValue::undefined(),
                &[accumulator, k_value, k.into(), this.clone()],
                context,
            )?;

            // d. Set k to k - 1.
            k -= 1;
        }

        // 11. Return accumulator.
        Ok(accumulator)
    }

    /// `23.2.3.23 %TypedArray%.prototype.reverse ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.reverse
    #[allow(clippy::float_cmp)]
    fn reverse(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as f64;

        // 4. Let middle be floor(len / 2).
        let middle = (len / 2.0).floor();

        // 5. Let lower be 0.
        let mut lower = 0.0;
        // 6. Repeat, while lower ≠ middle,
        while lower != middle {
            // a. Let upper be len - lower - 1.
            let upper = len - lower - 1.0;

            // b. Let upperP be ! ToString(𝔽(upper)).
            // c. Let lowerP be ! ToString(𝔽(lower)).
            // d. Let lowerValue be ! Get(O, lowerP).
            let lower_value = obj.get(lower, context).expect("Get cannot fail here");
            // e. Let upperValue be ! Get(O, upperP).
            let upper_value = obj.get(upper, context).expect("Get cannot fail here");

            // f. Perform ! Set(O, lowerP, upperValue, true).
            obj.set(lower, upper_value, true, context)
                .expect("Set cannot fail here");
            // g. Perform ! Set(O, upperP, lowerValue, true).
            obj.set(upper, lower_value, true, context)
                .expect("Set cannot fail here");

            // h. Set lower to lower + 1.
            lower += 1.0;
        }

        // 7. Return O.
        Ok(this.clone())
    }

    /// `23.2.3.24 %TypedArray%.prototype.set ( source [ , offset ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.set
    fn set(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let target be the this value.
        // 2. Perform ? RequireInternalSlot(target, [[TypedArrayName]]).
        // 3. Assert: target has a [[ViewedArrayBuffer]] internal slot.
        let target = this.as_object().ok_or_else(|| {
            context.construct_type_error("TypedArray.set must be called on typed array object")
        })?;
        if !target.is_typed_array() {
            return context.throw_type_error("TypedArray.set must be called on typed array object");
        }

        // 4. Let targetOffset be ? ToIntegerOrInfinity(offset).
        let target_offset = args.get_or_undefined(1).to_integer_or_infinity(context)?;

        // 5. If targetOffset < 0, throw a RangeError exception.
        match target_offset {
            IntegerOrInfinity::Integer(i) if i < 0 => {
                return context.throw_range_error("TypedArray.set called with negative offset")
            }
            IntegerOrInfinity::NegativeInfinity => {
                return context.throw_range_error("TypedArray.set called with negative offset")
            }
            _ => {}
        }

        let source = args.get_or_undefined(0);
        match source {
            // 6. If source is an Object that has a [[TypedArrayName]] internal slot, then
            JsValue::Object(source) if source.is_typed_array() => {
                // a. Perform ? SetTypedArrayFromTypedArray(target, targetOffset, source).
                Self::set_typed_array_from_typed_array(target, target_offset, source, context)?;
            }
            // 7. Else,
            _ => {
                // a. Perform ? SetTypedArrayFromArrayLike(target, targetOffset, source).
                Self::set_typed_array_from_array_like(target, target_offset, source, context)?;
            }
        }

        // 8. Return undefined.
        Ok(JsValue::undefined())
    }

    /// `3.2.3.24.1 SetTypedArrayFromTypedArray ( target, targetOffset, source )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-settypedarrayfromtypedarray
    fn set_typed_array_from_typed_array(
        target: &JsObject,
        target_offset: IntegerOrInfinity,
        source: &JsObject,
        context: &mut Context,
    ) -> JsResult<()> {
        let target_borrow = target.borrow();
        let target_array = target_borrow
            .as_typed_array()
            .expect("Target must be a typed array");

        let source_borrow = source.borrow();
        let source_array = source_borrow
            .as_typed_array()
            .expect("Source must be a typed array");

        // 1. Let targetBuffer be target.[[ViewedArrayBuffer]].
        // 2. If IsDetachedBuffer(targetBuffer) is true, throw a TypeError exception.
        if target_array.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }
        let target_buffer_obj = target_array
            .viewed_array_buffer()
            .expect("Already checked for detached buffer");

        // 3. Let targetLength be target.[[ArrayLength]].
        let target_length = target_array.array_length();

        // 4. Let srcBuffer be source.[[ViewedArrayBuffer]].
        // 5. If IsDetachedBuffer(srcBuffer) is true, throw a TypeError exception.
        if source_array.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }
        let mut src_buffer_obj = source_array
            .viewed_array_buffer()
            .expect("Already checked for detached buffer")
            .clone();

        // 6. Let targetName be the String value of target.[[TypedArrayName]].
        // 7. Let targetType be the Element Type value in Table 73 for targetName.
        let target_name = target_array.typed_array_name();

        // 8. Let targetElementSize be the Element Size value specified in Table 73 for targetName.
        let target_element_size = target_name.element_size();

        // 9. Let targetByteOffset be target.[[ByteOffset]].
        let target_byte_offset = target_array.byte_offset();

        // 10. Let srcName be the String value of source.[[TypedArrayName]].
        // 11. Let srcType be the Element Type value in Table 73 for srcName.
        let src_name = source_array.typed_array_name();

        // 12. Let srcElementSize be the Element Size value specified in Table 73 for srcName.
        let src_element_size = src_name.element_size();

        // 13. Let srcLength be source.[[ArrayLength]].
        let src_length = source_array.array_length();

        // 14. Let srcByteOffset be source.[[ByteOffset]].
        let src_byte_offset = source_array.byte_offset();

        // 15. If targetOffset is +∞, throw a RangeError exception.
        let target_offset = match target_offset {
            IntegerOrInfinity::Integer(i) if i >= 0 => i as usize,
            IntegerOrInfinity::PositiveInfinity => {
                return context.throw_range_error("Target offset cannot be Infinity");
            }
            _ => unreachable!(),
        };

        // 16. If srcLength + targetOffset > targetLength, throw a RangeError exception.
        if src_length + target_offset > target_length {
            return context.throw_range_error(
                "Source typed array and target offset longer than target typed array",
            );
        }

        // 17. If target.[[ContentType]] ≠ source.[[ContentType]], throw a TypeError exception.
        if target_name.content_type() != src_name.content_type() {
            return context.throw_type_error(
                "Source typed array and target typed array have different content types",
            );
        }

        // TODO: Shared Array Buffer
        // 18. If both IsSharedArrayBuffer(srcBuffer) and IsSharedArrayBuffer(targetBuffer) are true, then

        // a. If srcBuffer.[[ArrayBufferData]] and targetBuffer.[[ArrayBufferData]] are the same Shared Data Block values, let same be true; else let same be false.

        // 19. Else, let same be SameValue(srcBuffer, targetBuffer).
        let same = JsObject::equals(&src_buffer_obj, target_buffer_obj);

        // 20. If same is true, then
        let mut src_byte_index = if same {
            // a. Let srcByteLength be source.[[ByteLength]].
            let src_byte_length = source_array.byte_length();

            // b. Set srcBuffer to ? CloneArrayBuffer(srcBuffer, srcByteOffset, srcByteLength, %ArrayBuffer%).
            // c. NOTE: %ArrayBuffer% is used to clone srcBuffer because is it known to not have any observable side-effects.
            let array_buffer_constructor = context
                .intrinsics()
                .constructors()
                .array_buffer()
                .constructor()
                .into();
            let s = src_buffer_obj
                .borrow()
                .as_array_buffer()
                .expect("Already checked for detached buffer")
                .clone_array_buffer(
                    src_byte_offset,
                    src_byte_length,
                    &array_buffer_constructor,
                    context,
                )?;
            src_buffer_obj = s;

            // d. Let srcByteIndex be 0.
            0
        }
        // 21. Else, let srcByteIndex be srcByteOffset.
        else {
            src_byte_offset
        };

        // 22. Let targetByteIndex be targetOffset × targetElementSize + targetByteOffset.
        let mut target_byte_index = target_offset * target_element_size + target_byte_offset;

        // 23. Let limit be targetByteIndex + targetElementSize × srcLength.
        let limit = target_byte_index + target_element_size * src_length;

        let src_buffer_obj_borrow = src_buffer_obj.borrow();
        let src_buffer = src_buffer_obj_borrow
            .as_array_buffer()
            .expect("Must be an array buffer");

        // 24. If srcType is the same as targetType, then
        if src_name == target_name {
            // a. NOTE: If srcType and targetType are the same, the transfer must be performed in a manner that preserves the bit-level encoding of the source data.
            // b. Repeat, while targetByteIndex < limit,
            while target_byte_index < limit {
                // i. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, Uint8, true, Unordered).
                let value = src_buffer.get_value_from_buffer(
                    src_byte_index,
                    TypedArrayKind::Uint8,
                    true,
                    SharedMemoryOrder::Unordered,
                    None,
                );

                // ii. Perform SetValueInBuffer(targetBuffer, targetByteIndex, Uint8, value, true, Unordered).
                target_buffer_obj
                    .borrow_mut()
                    .as_array_buffer_mut()
                    .expect("Must be an array buffer")
                    .set_value_in_buffer(
                        target_byte_index,
                        TypedArrayKind::Uint8,
                        &value,
                        SharedMemoryOrder::Unordered,
                        None,
                        context,
                    )?;

                // iii. Set srcByteIndex to srcByteIndex + 1.
                src_byte_index += 1;

                // iv. Set targetByteIndex to targetByteIndex + 1.
                target_byte_index += 1;
            }
        }
        // 25. Else,
        else {
            // a. Repeat, while targetByteIndex < limit,
            while target_byte_index < limit {
                // i. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, srcType, true, Unordered).
                let value = src_buffer.get_value_from_buffer(
                    src_byte_index,
                    src_name,
                    true,
                    SharedMemoryOrder::Unordered,
                    None,
                );

                // ii. Perform SetValueInBuffer(targetBuffer, targetByteIndex, targetType, value, true, Unordered).
                target_buffer_obj
                    .borrow_mut()
                    .as_array_buffer_mut()
                    .expect("Must be an array buffer")
                    .set_value_in_buffer(
                        target_byte_index,
                        target_name,
                        &value,
                        SharedMemoryOrder::Unordered,
                        None,
                        context,
                    )?;

                // iii. Set srcByteIndex to srcByteIndex + srcElementSize.
                src_byte_index += src_element_size;

                // iv. Set targetByteIndex to targetByteIndex + targetElementSize.
                target_byte_index += target_element_size;
            }
        }

        Ok(())
    }

    /// `23.2.3.24.2 SetTypedArrayFromArrayLike ( target, targetOffset, source )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-settypedarrayfromarraylike
    fn set_typed_array_from_array_like(
        target: &JsObject,
        target_offset: IntegerOrInfinity,
        source: &JsValue,
        context: &mut Context,
    ) -> JsResult<()> {
        let target_borrow = target.borrow();
        let target_array = target_borrow
            .as_typed_array()
            .expect("Target must be a typed array");

        // 1. Let targetBuffer be target.[[ViewedArrayBuffer]].
        // 2. If IsDetachedBuffer(targetBuffer) is true, throw a TypeError exception.
        if target_array.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let targetLength be target.[[ArrayLength]].
        let target_length = target_array.array_length();

        // 4. Let targetName be the String value of target.[[TypedArrayName]].
        // 6. Let targetType be the Element Type value in Table 73 for targetName.
        let target_name = target_array.typed_array_name();

        // 5. Let targetElementSize be the Element Size value specified in Table 73 for targetName.
        let target_element_size = target_name.element_size();

        // 7. Let targetByteOffset be target.[[ByteOffset]].
        let target_byte_offset = target_array.byte_offset();

        // 8. Let src be ? ToObject(source).
        let src = source.to_object(context)?;

        // 9. Let srcLength be ? LengthOfArrayLike(src).
        let src_length = src.length_of_array_like(context)?;

        let target_offset = match target_offset {
            // 10. If targetOffset is +∞, throw a RangeError exception.
            IntegerOrInfinity::PositiveInfinity => {
                return context.throw_range_error("Target offset cannot be Infinity")
            }
            IntegerOrInfinity::Integer(i) if i >= 0 => i as usize,
            _ => unreachable!(),
        };

        // 11. If srcLength + targetOffset > targetLength, throw a RangeError exception.
        if src_length + target_offset > target_length {
            return context.throw_range_error(
                "Source object and target offset longer than target typed array",
            );
        }

        // 12. Let targetByteIndex be targetOffset × targetElementSize + targetByteOffset.
        let mut target_byte_index = target_offset * target_element_size + target_byte_offset;

        // 13. Let k be 0.
        let mut k = 0;

        // 14. Let limit be targetByteIndex + targetElementSize × srcLength.
        let limit = target_byte_index + target_element_size * src_length;

        // 15. Repeat, while targetByteIndex < limit,
        while target_byte_index < limit {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let value be ? Get(src, Pk).
            let value = src.get(k, context)?;

            // c. If target.[[ContentType]] is BigInt, set value to ? ToBigInt(value).
            // d. Otherwise, set value to ? ToNumber(value).
            let value = if target_name.content_type() == ContentType::BigInt {
                value.to_bigint(context)?.into()
            } else {
                value.to_number(context)?.into()
            };

            let target_buffer_obj = target_array
                .viewed_array_buffer()
                .expect("Already checked for detached buffer");
            let mut target_buffer_obj_borrow = target_buffer_obj.borrow_mut();
            let target_buffer = target_buffer_obj_borrow
                .as_array_buffer_mut()
                .expect("Already checked for detached buffer");

            // e. If IsDetachedBuffer(targetBuffer) is true, throw a TypeError exception.
            if target_buffer.is_detached_buffer() {
                return context.throw_type_error("Cannot set value on detached array buffer");
            }

            // f. Perform SetValueInBuffer(targetBuffer, targetByteIndex, targetType, value, true, Unordered).
            target_buffer.set_value_in_buffer(
                target_byte_index,
                target_name,
                &value,
                SharedMemoryOrder::Unordered,
                None,
                context,
            )?;

            // g. Set k to k + 1.
            k += 1;

            // h. Set targetByteIndex to targetByteIndex + targetElementSize.
            target_byte_index += target_element_size;
        }

        Ok(())
    }

    /// `23.2.3.25 %TypedArray%.prototype.slice ( start, end )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.slice
    fn slice(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        // 4. Let relativeStart be ? ToIntegerOrInfinity(start).
        let mut k = match args.get_or_undefined(0).to_integer_or_infinity(context)? {
            // 5. If relativeStart is -∞, let k be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 6. Else if relativeStart < 0, let k be max(len + relativeStart, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(len + i, 0),
            // 7. Else, let k be min(relativeStart, len).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 8. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
        let end = args.get_or_undefined(1);
        let relative_end = if end.is_undefined() {
            IntegerOrInfinity::Integer(len)
        } else {
            end.to_integer_or_infinity(context)?
        };

        let r#final = match relative_end {
            // 9. If relativeEnd is -∞, let final be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 10. Else if relativeEnd < 0, let final be max(len + relativeEnd, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(len + i, 0),
            // 11. Else, let final be min(relativeEnd, len).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 12. Let count be max(final - k, 0).
        let count = std::cmp::max(r#final - k, 0) as usize;

        // 13. Let A be ? TypedArraySpeciesCreate(O, « 𝔽(count) »).
        let a = Self::species_create(obj, o.typed_array_name(), &[count.into()], context)?;
        let a_borrow = a.borrow();
        let a_array = a_borrow
            .as_typed_array()
            .expect("This must be a typed array");

        // 14. If count > 0, then
        if count > 0 {
            // a. If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
            if o.is_detached() {
                return context.throw_type_error("Buffer of the typed array is detached");
            }

            // b. Let srcName be the String value of O.[[TypedArrayName]].
            // c. Let srcType be the Element Type value in Table 73 for srcName.
            // d. Let targetName be the String value of A.[[TypedArrayName]].
            // e. Let targetType be the Element Type value in Table 73 for targetName.
            // f. If srcType is different from targetType, then
            #[allow(clippy::if_not_else)]
            if o.typed_array_name() != a_array.typed_array_name() {
                // i. Let n be 0.
                let mut n = 0;
                // ii. Repeat, while k < final,
                while k < r#final {
                    // 1. Let Pk be ! ToString(𝔽(k)).
                    // 2. Let kValue be ! Get(O, Pk).
                    let k_value = obj.get(k, context).expect("Get cannot fail here");

                    // 3. Perform ! Set(A, ! ToString(𝔽(n)), kValue, true).
                    a.set(n, k_value, true, context)
                        .expect("Set cannot fail here");

                    // 4. Set k to k + 1.
                    k += 1;

                    // 5. Set n to n + 1.
                    n += 1;
                }
            // g. Else,
            } else {
                // i. Let srcBuffer be O.[[ViewedArrayBuffer]].
                let src_buffer_obj = o.viewed_array_buffer().expect("Cannot be detached here");
                let src_buffer_obj_borrow = src_buffer_obj.borrow();
                let src_buffer = src_buffer_obj_borrow
                    .as_array_buffer()
                    .expect("Cannot be detached here");

                // ii. Let targetBuffer be A.[[ViewedArrayBuffer]].
                let target_buffer_obj = a_array
                    .viewed_array_buffer()
                    .expect("Cannot be detached here");
                let mut target_buffer_obj_borrow = target_buffer_obj.borrow_mut();
                let target_buffer = target_buffer_obj_borrow
                    .as_array_buffer_mut()
                    .expect("Cannot be detached here");

                // iii. Let elementSize be the Element Size value specified in Table 73 for Element Type srcType.
                let element_size = o.typed_array_name().element_size();

                // iv. NOTE: If srcType and targetType are the same, the transfer must be performed in a manner that preserves the bit-level encoding of the source data.

                // v. Let srcByteOffset be O.[[ByteOffset]].
                let src_byte_offset = o.byte_offset();

                // vi. Let targetByteIndex be A.[[ByteOffset]].
                let mut target_byte_index = a_array.byte_offset();

                // vii. Let srcByteIndex be (k × elementSize) + srcByteOffset.
                let mut src_byte_index = k as usize * element_size + src_byte_offset;

                // viii. Let limit be targetByteIndex + count × elementSize.
                let limit = target_byte_index + count * element_size;

                // ix. Repeat, while targetByteIndex < limit,
                while target_byte_index < limit {
                    // 1. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, Uint8, true, Unordered).
                    let value = src_buffer.get_value_from_buffer(
                        src_byte_index,
                        TypedArrayKind::Uint8,
                        true,
                        SharedMemoryOrder::Unordered,
                        None,
                    );

                    // 2. Perform SetValueInBuffer(targetBuffer, targetByteIndex, Uint8, value, true, Unordered).
                    target_buffer.set_value_in_buffer(
                        target_byte_index,
                        TypedArrayKind::Uint8,
                        &value,
                        SharedMemoryOrder::Unordered,
                        None,
                        context,
                    )?;

                    // 3. Set srcByteIndex to srcByteIndex + 1.
                    src_byte_index += 1;

                    // 4. Set targetByteIndex to targetByteIndex + 1.
                    target_byte_index += 1;
                }
            }
        }

        drop(a_borrow);

        // 15. Return A.
        Ok(a.into())
    }

    /// `23.2.3.26 %TypedArray%.prototype.some ( callbackfn [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.some
    fn some(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => {
                return context.throw_type_error(
                    "TypedArray.prototype.some called with non-callable callback function",
                )
            }
        };

        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ! Get(O, Pk).
            let k_value = obj.get(k, context).expect("Get cannot fail here");

            // c. Let testResult be ! ToBoolean(? Call(callbackfn, thisArg, « kValue, 𝔽(k), O »)).
            // d. If testResult is true, return true.
            if callback_fn
                .call(
                    args.get_or_undefined(1),
                    &[k_value, k.into(), this.clone()],
                    context,
                )?
                .to_boolean()
            {
                return Ok(true.into());
            }
        }

        // 7. Return false.
        Ok(false.into())
    }

    /// `23.2.3.27 %TypedArray%.prototype.sort ( comparefn )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.sort
    fn sort(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. If comparefn is not undefined and IsCallable(comparefn) is false, throw a TypeError exception.
        let compare_fn = match args.get(0) {
            None | Some(JsValue::Undefined) => None,
            Some(JsValue::Object(obj)) if obj.is_callable() => Some(obj),
            _ => {
                return context
                    .throw_type_error("TypedArray.sort called with non-callable comparefn")
            }
        };

        // 2. Let obj be the this value.
        let obj = this.as_object().ok_or_else(|| {
            context.construct_type_error("TypedArray.sort must be called on typed array object")
        })?;

        // 4. Let buffer be obj.[[ViewedArrayBuffer]].
        // 5. Let len be obj.[[ArrayLength]].
        let (buffer, len) = {
            // 3. Perform ? ValidateTypedArray(obj).
            let obj_borrow = obj.borrow();
            let o = obj_borrow.as_typed_array().ok_or_else(|| {
                context.construct_type_error("TypedArray.sort must be called on typed array object")
            })?;
            if o.is_detached() {
                return context.throw_type_error(
                    "TypedArray.sort called on typed array object with detached array buffer",
                );
            }

            (
                o.viewed_array_buffer()
                    .expect("Already checked for detached buffer")
                    .clone(),
                o.array_length(),
            )
        };

        // 4. Let items be a new empty List.
        let mut items = Vec::with_capacity(len);

        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kPresent be ? HasProperty(obj, Pk).
            // c. If kPresent is true, then
            if obj.has_property(k, context)? {
                // i. Let kValue be ? Get(obj, Pk).
                let k_val = obj.get(k, context)?;
                // ii. Append kValue to items.
                items.push(k_val);
            }
            // d. Set k to k + 1.
        }

        // 7. Let itemCount be the number of elements in items.
        let item_count = items.len();

        let sort_compare = |x: &JsValue,
                            y: &JsValue,
                            compare_fn: Option<&JsObject>,
                            context: &mut Context|
         -> JsResult<Ordering> {
            // 1. Assert: Both Type(x) and Type(y) are Number or both are BigInt.
            // 2. If comparefn is not undefined, then
            if let Some(obj) = compare_fn {
                // a. Let v be ? ToNumber(? Call(comparefn, undefined, « x, y »)).
                let v = obj
                    .call(&JsValue::undefined(), &[x.clone(), y.clone()], context)?
                    .to_number(context)?;

                // b. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
                if buffer
                    .borrow()
                    .as_array_buffer()
                    .expect("Must be array buffer")
                    .is_detached_buffer()
                {
                    return context
                        .throw_type_error("Cannot sort typed array with detached buffer");
                }

                // c. If v is NaN, return +0𝔽.
                // d. Return v.
                return Ok(v.partial_cmp(&0.0).unwrap_or(Ordering::Equal));
            }

            if let (JsValue::BigInt(x), JsValue::BigInt(y)) = (x, y) {
                // 6. If x < y, return -1𝔽.
                if x < y {
                    return Ok(Ordering::Less);
                }

                // 7. If x > y, return 1𝔽.
                if x > y {
                    return Ok(Ordering::Greater);
                }

                // 8. If x is -0𝔽 and y is +0𝔽, return -1𝔽.
                if x.is_zero()
                    && y.is_zero()
                    && x.as_inner().is_negative()
                    && y.as_inner().is_positive()
                {
                    return Ok(Ordering::Less);
                }

                // 9. If x is +0𝔽 and y is -0𝔽, return 1𝔽.
                if x.is_zero()
                    && y.is_zero()
                    && x.as_inner().is_positive()
                    && y.as_inner().is_negative()
                {
                    return Ok(Ordering::Greater);
                }

                // 10. Return +0𝔽.
                Ok(Ordering::Equal)
            } else {
                let x = x
                    .as_number()
                    .expect("Typed array can only contain number or bigint");
                let y = y
                    .as_number()
                    .expect("Typed array can only contain number or bigint");

                // 3. If x and y are both NaN, return +0𝔽.
                if x.is_nan() && y.is_nan() {
                    return Ok(Ordering::Equal);
                }

                // 4. If x is NaN, return 1𝔽.
                if x.is_nan() {
                    return Ok(Ordering::Greater);
                }

                // 5. If y is NaN, return -1𝔽.
                if y.is_nan() {
                    return Ok(Ordering::Less);
                }

                // 6. If x < y, return -1𝔽.
                if x < y {
                    return Ok(Ordering::Less);
                }

                // 7. If x > y, return 1𝔽.
                if x > y {
                    return Ok(Ordering::Greater);
                }

                // 8. If x is -0𝔽 and y is +0𝔽, return -1𝔽.
                if x.is_zero() && y.is_zero() && x.is_sign_negative() && y.is_sign_positive() {
                    return Ok(Ordering::Less);
                }

                // 9. If x is +0𝔽 and y is -0𝔽, return 1𝔽.
                if x.is_zero() && y.is_zero() && x.is_sign_positive() && y.is_sign_negative() {
                    return Ok(Ordering::Greater);
                }

                // 10. Return +0𝔽.
                Ok(Ordering::Equal)
            }
        };

        // 8. Sort items using an implementation-defined sequence of calls to SortCompare.
        // If any such call returns an abrupt completion, stop before performing any further
        // calls to SortCompare or steps in this algorithm and return that completion.
        let mut sort_err = Ok(());
        items.sort_by(|x, y| {
            if sort_err.is_ok() {
                sort_compare(x, y, compare_fn, context).unwrap_or_else(|err| {
                    sort_err = Err(err);
                    Ordering::Equal
                })
            } else {
                Ordering::Equal
            }
        });
        sort_err?;

        // 9. Let j be 0.
        // 10. Repeat, while j < itemCount,
        for (j, item) in items.into_iter().enumerate() {
            // a. Perform ? Set(obj, ! ToString(𝔽(j)), items[j], true).
            obj.set(j, item, true, context)?;
            // b. Set j to j + 1.
        }

        // 11. Repeat, while j < len,
        for j in item_count..len {
            // a. Perform ? DeletePropertyOrThrow(obj, ! ToString(𝔽(j))).
            obj.delete_property_or_throw(j, context)?;
            // b. Set j to j + 1.
        }

        // 12. Return obj.
        Ok(obj.clone().into())
    }

    /// `23.2.3.28 %TypedArray%.prototype.subarray ( begin, end )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.subarray
    fn subarray(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        let obj = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;

        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        let buffer = o
            .viewed_array_buffer()
            .expect("Buffer cannot be detached here");

        // 5. Let srcLength be O.[[ArrayLength]].
        let src_length = o.array_length() as i64;

        // 6. Let relativeBegin be ? ToIntegerOrInfinity(begin).
        let begin_index = match args.get_or_undefined(0).to_integer_or_infinity(context)? {
            // 7. If relativeBegin is -∞, let beginIndex be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 8. Else if relativeBegin < 0, let beginIndex be max(srcLength + relativeBegin, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(src_length + i, 0),
            // 9. Else, let beginIndex be min(relativeBegin, srcLength).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, src_length),
            IntegerOrInfinity::PositiveInfinity => src_length,
        };

        // 10. If end is undefined, let relativeEnd be srcLength; else let relativeEnd be ? ToIntegerOrInfinity(end).
        let end = args.get_or_undefined(1);
        let relative_end = if end.is_undefined() {
            IntegerOrInfinity::Integer(src_length)
        } else {
            end.to_integer_or_infinity(context)?
        };

        let end_index = match relative_end {
            // 11. If relativeEnd is -∞, let endIndex be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 12. Else if relativeEnd < 0, let endIndex be max(srcLength + relativeEnd, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(src_length + i, 0),
            // 13. Else, let endIndex be min(relativeEnd, srcLength).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, src_length),
            IntegerOrInfinity::PositiveInfinity => src_length,
        };

        // 14. Let newLength be max(endIndex - beginIndex, 0).
        let new_length = std::cmp::max(end_index - begin_index, 0);

        // 15. Let constructorName be the String value of O.[[TypedArrayName]].
        // 16. Let elementSize be the Element Size value specified in Table 73 for constructorName.
        let element_size = o.typed_array_name().element_size();

        // 17. Let srcByteOffset be O.[[ByteOffset]].
        let src_byte_offset = o.byte_offset();

        // 18. Let beginByteOffset be srcByteOffset + beginIndex × elementSize.
        let begin_byte_offset = src_byte_offset + begin_index as usize * element_size;

        // 19. Let argumentsList be « buffer, 𝔽(beginByteOffset), 𝔽(newLength) ».
        // 20. Return ? TypedArraySpeciesCreate(O, argumentsList).
        Ok(Self::species_create(
            obj,
            o.typed_array_name(),
            &[
                buffer.clone().into(),
                begin_byte_offset.into(),
                new_length.into(),
            ],
            context,
        )?
        .into())
    }

    // TODO: 23.2.3.29 %TypedArray%.prototype.toLocaleString ( [ reserved1 [ , reserved2 ] ] )

    /// `23.2.3.31 %TypedArray%.prototype.values ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.values
    fn values(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let o = this
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.borrow()
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?
            .is_detached()
        {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. Return CreateArrayIterator(O, value).
        Ok(ArrayIterator::create_array_iterator(
            o.clone(),
            PropertyNameKind::Value,
            context,
        ))
    }

    /// `23.2.3.33 get %TypedArray%.prototype [ @@toStringTag ]`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-%typedarray%.prototype-@@tostringtag
    #[allow(clippy::unnecessary_wraps)]
    fn to_string_tag(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If Type(O) is not Object, return undefined.
        // 3. If O does not have a [[TypedArrayName]] internal slot, return undefined.
        // 4. Let name be O.[[TypedArrayName]].
        // 5. Assert: Type(name) is String.
        // 6. Return name.
        Ok(this
            .as_object()
            .and_then(|obj| {
                obj.borrow()
                    .as_typed_array()
                    .map(|o| o.typed_array_name().name().into())
            })
            .unwrap_or(JsValue::Undefined))
    }

    /// `23.2.4.1 TypedArraySpeciesCreate ( exemplar, argumentList )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#typedarray-species-create
    fn species_create(
        exemplar: &JsObject,
        typed_array_name: TypedArrayKind,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. Let defaultConstructor be the intrinsic object listed in column one of Table 73 for exemplar.[[TypedArrayName]].
        let default_constructor = match typed_array_name {
            TypedArrayKind::Int8 => StandardConstructors::typed_int8_array,
            TypedArrayKind::Uint8 => StandardConstructors::typed_uint8_array,
            TypedArrayKind::Uint8Clamped => StandardConstructors::typed_uint8clamped_array,
            TypedArrayKind::Int16 => StandardConstructors::typed_int16_array,
            TypedArrayKind::Uint16 => StandardConstructors::typed_uint16_array,
            TypedArrayKind::Int32 => StandardConstructors::typed_int32_array,
            TypedArrayKind::Uint32 => StandardConstructors::typed_uint32_array,
            TypedArrayKind::BigInt64 => StandardConstructors::typed_bigint64_array,
            TypedArrayKind::BigUint64 => StandardConstructors::typed_biguint64_array,
            TypedArrayKind::Float32 => StandardConstructors::typed_float32_array,
            TypedArrayKind::Float64 => StandardConstructors::typed_float64_array,
        };

        // 2. Let constructor be ? SpeciesConstructor(exemplar, defaultConstructor).
        let constructor = exemplar.species_constructor(default_constructor, context)?;

        // 3. Let result be ? TypedArrayCreate(constructor, argumentList).
        let result = Self::create(&constructor, args, context)?;

        // 4. Assert: result has [[TypedArrayName]] and [[ContentType]] internal slots.
        // 5. If result.[[ContentType]] ≠ exemplar.[[ContentType]], throw a TypeError exception.
        if result
            .borrow()
            .as_typed_array()
            .expect("This can only be a typed array object")
            .typed_array_name()
            .content_type()
            != typed_array_name.content_type()
        {
            return context
                .throw_type_error("New typed array has different context type than exemplar");
        }

        // 6. Return result.
        Ok(result)
    }

    /// `23.2.4.2 TypedArrayCreate ( constructor, argumentList )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#typedarray-create
    fn create(
        constructor: &JsObject,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. Let newTypedArray be ? Construct(constructor, argumentList).
        let new_typed_array = constructor.construct(args, &constructor.clone().into(), context)?;

        // 2. Perform ? ValidateTypedArray(newTypedArray).
        let obj = new_typed_array
            .as_object()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow
            .as_typed_array()
            .ok_or_else(|| context.construct_type_error("Value is not a typed array object"))?;
        if o.is_detached() {
            return context.throw_type_error("Buffer of the typed array is detached");
        }

        // 3. If argumentList is a List of a single Number, then
        if args.len() == 1 {
            if let Some(number) = args[0].as_number() {
                // a. If newTypedArray.[[ArrayLength]] < ℝ(argumentList[0]), throw a TypeError exception.
                if (o.array_length() as f64) < number {
                    return context
                        .throw_type_error("New typed array length is smaller than expected");
                }
            }
        }

        // 4. Return newTypedArray.
        Ok(obj.clone())
    }

    /// <https://tc39.es/ecma262/#sec-allocatetypedarraybuffer>
    fn allocate_buffer(
        indexed: &mut IntegerIndexed,
        length: usize,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Assert: O.[[ViewedArrayBuffer]] is undefined.
        assert!(indexed.viewed_array_buffer().is_none());

        // 2. Let constructorName be the String value of O.[[TypedArrayName]].
        // 3. Let elementSize be the Element Size value specified in Table 73 for constructorName.
        let element_size = indexed.typed_array_name().element_size();

        // 4. Let byteLength be elementSize × length.
        let byte_length = element_size * length;

        // 5. Let data be ? AllocateArrayBuffer(%ArrayBuffer%, byteLength).
        let data = ArrayBuffer::allocate(
            &context
                .intrinsics()
                .constructors()
                .array_buffer()
                .constructor()
                .into(),
            byte_length,
            context,
        )?;

        // 6. Set O.[[ViewedArrayBuffer]] to data.
        indexed.set_viewed_array_buffer(Some(data));
        // 7. Set O.[[ByteLength]] to byteLength.
        indexed.set_byte_length(byte_length);
        // 8. Set O.[[ByteOffset]] to 0.
        indexed.set_byte_offset(0);
        // 9. Set O.[[ArrayLength]] to length.
        indexed.set_array_length(length);

        // 10. Return O.
        Ok(())
    }

    /// <https://tc39.es/ecma262/#sec-initializetypedarrayfromlist>
    fn initialize_from_list(
        o: &JsObject,
        values: Vec<JsValue>,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let len be the number of elements in values.
        let len = values.len();
        {
            let mut o = o.borrow_mut();
            let o_inner = o.as_typed_array_mut().expect("expected a TypedArray");

            // 2. Perform ? AllocateTypedArrayBuffer(O, len).
            Self::allocate_buffer(o_inner, len, context)?;
        }

        // 3. Let k be 0.
        // 4. Repeat, while k < len,
        for (k, k_value) in values.into_iter().enumerate() {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be the first element of values and remove that element from values.
            // c. Perform ? Set(O, Pk, kValue, true).
            o.set(k, k_value, true, context)?;
            // d. Set k to k + 1.
        }

        // 5. Assert: values is now an empty List.
        // It no longer exists.
        Ok(())
    }

    /// `AllocateTypedArray ( constructorName, newTarget, defaultProto [ , length ] )`
    ///
    /// It is used to validate and create an instance of a `TypedArray` constructor. If the `length`
    /// argument is passed, an `ArrayBuffer` of that length is also allocated and associated with the
    /// new `TypedArray` instance. `AllocateTypedArray` provides common semantics that is used by
    /// `TypedArray`.
    ///
    /// For more information, check the [spec][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-allocatetypedarray
    fn allocate<P>(
        constructor_name: TypedArrayKind,
        new_target: &JsValue,
        default_proto: P,
        length: Option<usize>,
        context: &mut Context,
    ) -> JsResult<JsObject>
    where
        P: FnOnce(&StandardConstructors) -> &StandardConstructor,
    {
        // 1. Let proto be ? GetPrototypeFromConstructor(newTarget, defaultProto).
        let proto = get_prototype_from_constructor(new_target, default_proto, context)?;

        // 3. Assert: obj.[[ViewedArrayBuffer]] is undefined.
        // 4. Set obj.[[TypedArrayName]] to constructorName.
        // 5. If constructorName is "BigInt64Array" or "BigUint64Array", set obj.[[ContentType]] to BigInt.
        // 6. Otherwise, set obj.[[ContentType]] to Number.
        // 7. If length is not present, then
        // a. Set obj.[[ByteLength]] to 0.
        // b. Set obj.[[ByteOffset]] to 0.
        // c. Set obj.[[ArrayLength]] to 0.
        let mut indexed = IntegerIndexed::new(None, constructor_name, 0, 0, 0);

        // 8. Else,
        if let Some(length) = length {
            // a. Perform ? AllocateTypedArrayBuffer(obj, length).
            Self::allocate_buffer(&mut indexed, length, context)?;
        }

        // 2. Let obj be ! IntegerIndexedObjectCreate(proto).
        let obj = IntegerIndexed::create(proto, indexed, context);

        // 9. Return obj.
        Ok(obj)
    }

    /// `23.2.5.1.2 InitializeTypedArrayFromTypedArray ( O, srcArray )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-initializetypedarrayfromtypedarray
    fn initialize_from_typed_array(
        o: &JsObject,
        src_array: &JsObject,
        context: &mut Context,
    ) -> JsResult<()> {
        let o_obj = o.borrow();
        let src_array_obj = src_array.borrow();
        let o_array = o_obj.as_typed_array().expect("this must be a typed array");
        let src_array = src_array_obj
            .as_typed_array()
            .expect("this must be a typed array");

        // 1. Let srcData be srcArray.[[ViewedArrayBuffer]].
        // 2. If IsDetachedBuffer(srcData) is true, throw a TypeError exception.
        if src_array.is_detached() {
            return context.throw_type_error("Cannot initialize typed array from detached buffer");
        }
        let src_data_obj = src_array
            .viewed_array_buffer()
            .expect("Already checked for detached buffer");

        // 3. Let constructorName be the String value of O.[[TypedArrayName]].
        // 4. Let elementType be the Element Type value in Table 73 for constructorName.
        // 10. Let elementSize be the Element Size value specified in Table 73 for constructorName.
        let constructor_name = o_array.typed_array_name();

        // 5. Let elementLength be srcArray.[[ArrayLength]].
        let element_length = src_array.array_length();

        // 6. Let srcName be the String value of srcArray.[[TypedArrayName]].
        // 7. Let srcType be the Element Type value in Table 73 for srcName.
        // 8. Let srcElementSize be the Element Size value specified in Table 73 for srcName.
        let src_name = src_array.typed_array_name();

        // 9. Let srcByteOffset be srcArray.[[ByteOffset]].
        let src_byte_offset = src_array.byte_offset();

        // 11. Let byteLength be elementSize × elementLength.
        let byte_length = constructor_name.element_size() * element_length;

        // 12. If IsSharedArrayBuffer(srcData) is false, then
        // a. Let bufferConstructor be ? SpeciesConstructor(srcData, %ArrayBuffer%).
        // TODO: Shared Array Buffer
        // 13. Else,
        // a. Let bufferConstructor be %ArrayBuffer%.
        let buffer_constructor =
            src_data_obj.species_constructor(StandardConstructors::array_buffer, context)?;

        let src_data_obj_b = src_data_obj.borrow();
        let src_data = src_data_obj_b
            .as_array_buffer()
            .expect("Already checked for detached buffer");

        // 14. If elementType is the same as srcType, then
        let data = if constructor_name == src_name {
            // a. Let data be ? CloneArrayBuffer(srcData, srcByteOffset, byteLength, bufferConstructor).
            src_data.clone_array_buffer(
                src_byte_offset,
                byte_length,
                &buffer_constructor.into(),
                context,
            )?
        // 15. Else,
        } else {
            // a. Let data be ? AllocateArrayBuffer(bufferConstructor, byteLength).
            let data_obj = ArrayBuffer::allocate(&buffer_constructor.into(), byte_length, context)?;
            let mut data_obj_b = data_obj.borrow_mut();
            let data = data_obj_b
                .as_array_buffer_mut()
                .expect("Must be ArrayBuffer");

            // b. If IsDetachedBuffer(srcData) is true, throw a TypeError exception.
            if src_data.is_detached_buffer() {
                return context
                    .throw_type_error("Cannot initialize typed array from detached buffer");
            }

            // c. If srcArray.[[ContentType]] ≠ O.[[ContentType]], throw a TypeError exception.
            if src_name.content_type() != constructor_name.content_type() {
                return context
                    .throw_type_error("Cannot initialize typed array from different content type");
            }

            // d. Let srcByteIndex be srcByteOffset.
            let mut src_byte_index = src_byte_offset;
            // e. Let targetByteIndex be 0.
            let mut target_byte_index = 0;
            // f. Let count be elementLength.
            let mut count = element_length;
            // g. Repeat, while count > 0,
            while count > 0 {
                // i. Let value be GetValueFromBuffer(srcData, srcByteIndex, srcType, true, Unordered).
                let value = src_data.get_value_from_buffer(
                    src_byte_index,
                    src_name,
                    true,
                    SharedMemoryOrder::Unordered,
                    None,
                );

                // ii. Perform SetValueInBuffer(data, targetByteIndex, elementType, value, true, Unordered).
                data.set_value_in_buffer(
                    target_byte_index,
                    constructor_name,
                    &value,
                    SharedMemoryOrder::Unordered,
                    None,
                    context,
                )?;

                // iii. Set srcByteIndex to srcByteIndex + srcElementSize.
                src_byte_index += src_name.element_size();

                // iv. Set targetByteIndex to targetByteIndex + elementSize.
                target_byte_index += constructor_name.element_size();

                // v. Set count to count - 1.
                count -= 1;
            }
            drop(data_obj_b);
            data_obj
        };

        // 16. Set O.[[ViewedArrayBuffer]] to data.
        // 17. Set O.[[ByteLength]] to byteLength.
        // 18. Set O.[[ByteOffset]] to 0.
        // 19. Set O.[[ArrayLength]] to elementLength.
        drop(o_obj);
        o.borrow_mut().data = ObjectData::integer_indexed(IntegerIndexed::new(
            Some(data),
            constructor_name,
            0,
            byte_length,
            element_length,
        ));

        Ok(())
    }

    /// `23.2.5.1.3 InitializeTypedArrayFromArrayBuffer ( O, buffer, byteOffset, length )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-initializetypedarrayfromarraybuffer
    fn initialize_from_array_buffer(
        o: &JsObject,
        buffer: JsObject,
        byte_offset: &JsValue,
        length: &JsValue,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let constructorName be the String value of O.[[TypedArrayName]].
        // 2. Let elementSize be the Element Size value specified in Table 73 for constructorName.
        let constructor_name = o
            .borrow()
            .as_typed_array()
            .expect("This must be a typed array")
            .typed_array_name();

        // 3. Let offset be ? ToIndex(byteOffset).
        let offset = byte_offset.to_index(context)?;

        // 4. If offset modulo elementSize ≠ 0, throw a RangeError exception.
        if offset % constructor_name.element_size() != 0 {
            return context.throw_range_error("Invalid length for typed array");
        }

        let buffer_byte_length = {
            let buffer_obj_b = buffer.borrow();
            let buffer_array = buffer_obj_b
                .as_array_buffer()
                .expect("This must be an ArrayBuffer");

            // 6. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
            if buffer_array.is_detached_buffer() {
                return context
                    .throw_type_error("Cannot construct typed array from detached buffer");
            }

            // 7. Let bufferByteLength be buffer.[[ArrayBufferByteLength]].
            buffer_array.array_buffer_byte_length()
        };

        // 8. If length is undefined, then
        let new_byte_length = if length.is_undefined() {
            // a. If bufferByteLength modulo elementSize ≠ 0, throw a RangeError exception.
            if buffer_byte_length % constructor_name.element_size() != 0 {
                return context.throw_range_error("Invalid length for typed array");
            }

            // b. Let newByteLength be bufferByteLength - offset.
            let new_byte_length = buffer_byte_length as isize - offset as isize;

            // c. If newByteLength < 0, throw a RangeError exception.
            if new_byte_length < 0 {
                return context.throw_range_error("Invalid length for typed array");
            }

            new_byte_length as usize
        // 9. Else,
        } else {
            // 5. If length is not undefined, then
            // a. Let newLength be ? ToIndex(length).

            // a. Let newByteLength be newLength × elementSize.
            let new_byte_length = length.to_index(context)? * constructor_name.element_size();

            // b. If offset + newByteLength > bufferByteLength, throw a RangeError exception.
            if offset + new_byte_length > buffer_byte_length {
                return context.throw_range_error("Invalid length for typed array");
            }

            new_byte_length
        };

        let mut o_obj_borrow = o.borrow_mut();
        let o = o_obj_borrow
            .as_typed_array_mut()
            .expect("This must be an ArrayBuffer");

        // 10. Set O.[[ViewedArrayBuffer]] to buffer.
        o.set_viewed_array_buffer(Some(buffer));
        // 11. Set O.[[ByteLength]] to newByteLength.
        o.set_byte_length(new_byte_length);
        // 12. Set O.[[ByteOffset]] to offset.
        o.set_byte_offset(offset);
        // 13. Set O.[[ArrayLength]] to newByteLength / elementSize.
        o.set_array_length(new_byte_length / constructor_name.element_size());

        Ok(())
    }

    /// `23.2.5.1.5 InitializeTypedArrayFromArrayLike ( O, arrayLike )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-initializetypedarrayfromarraylike
    fn initialize_from_array_like(
        o: &JsObject,
        array_like: &JsObject,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let len be ? LengthOfArrayLike(arrayLike).
        let len = array_like.length_of_array_like(context)?;

        // 2. Perform ? AllocateTypedArrayBuffer(O, len).
        {
            let mut o_borrow = o.borrow_mut();
            let o = o_borrow.as_typed_array_mut().expect("Must be typed array");
            Self::allocate_buffer(o, len, context)?;
        }

        // 3. Let k be 0.
        // 4. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ? Get(arrayLike, Pk).
            let k_value = array_like.get(k, context)?;

            // c. Perform ? Set(O, Pk, kValue, true).
            o.set(k, k_value, true, context)?;
        }

        Ok(())
    }
}

/// Names of all the typed arrays.
#[derive(Debug, Clone, Copy, Finalize, PartialEq)]
pub(crate) enum TypedArrayKind {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    BigInt64,
    BigUint64,
    Float32,
    Float64,
}

unsafe impl Trace for TypedArrayKind {
    // Safe because `TypedArrayName` is `Copy`
    unsafe_empty_trace!();
}

impl TypedArrayKind {
    /// Gets the element size of the given typed array name, as per the [spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#table-the-typedarray-constructors
    #[inline]
    pub(crate) const fn element_size(self) -> usize {
        match self {
            Self::Int8 | Self::Uint8 | Self::Uint8Clamped => 1,
            Self::Int16 | Self::Uint16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::BigInt64 | Self::BigUint64 | Self::Float64 => 8,
        }
    }

    /// Gets the content type of this typed array name.
    #[inline]
    pub(crate) const fn content_type(self) -> ContentType {
        match self {
            Self::BigInt64 | Self::BigUint64 => ContentType::BigInt,
            _ => ContentType::Number,
        }
    }

    /// Gets the name of this typed array name.
    #[inline]
    pub(crate) const fn name(&self) -> &str {
        match self {
            TypedArrayKind::Int8 => "Int8Array",
            TypedArrayKind::Uint8 => "Uint8Array",
            TypedArrayKind::Uint8Clamped => "Uint8ClampedArray",
            TypedArrayKind::Int16 => "Int16Array",
            TypedArrayKind::Uint16 => "Uint16Array",
            TypedArrayKind::Int32 => "Int32Array",
            TypedArrayKind::Uint32 => "Uint32Array",
            TypedArrayKind::BigInt64 => "BigInt64Array",
            TypedArrayKind::BigUint64 => "BigUint64Array",
            TypedArrayKind::Float32 => "Float32Array",
            TypedArrayKind::Float64 => "Float64Array",
        }
    }

    pub(crate) fn is_big_int_element_type(self) -> bool {
        matches!(self, TypedArrayKind::BigUint64 | TypedArrayKind::BigInt64)
    }
}

typed_array!(Int8Array, Int8, "Int8Array", typed_int8_array);
typed_array!(Uint8Array, Uint8, "Uint8Array", typed_uint8_array);
typed_array!(
    Uint8ClampedArray,
    Uint8Clamped,
    "Uint8ClampedArray",
    typed_uint8clamped_array
);
typed_array!(Int16Array, Int16, "Int16Array", typed_int16_array);
typed_array!(Uint16Array, Uint16, "Uint16Array", typed_uint16_array);
typed_array!(Int32Array, Int32, "Int32Array", typed_int32_array);
typed_array!(Uint32Array, Uint32, "Uint32Array", typed_uint32_array);
typed_array!(
    BigInt64Array,
    BigInt64,
    "BigInt64Array",
    typed_bigint64_array
);
typed_array!(
    BigUint64Array,
    BigUint64,
    "BigUint64Array",
    typed_biguint64_array
);
typed_array!(Float32Array, Float32, "Float32Array", typed_float32_array);
typed_array!(Float64Array, Float64, "Float64Array", typed_float64_array);
