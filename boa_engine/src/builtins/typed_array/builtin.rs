use std::{cmp::Ordering, ptr, sync::atomic};

use boa_macros::utf16;
use num_traits::Zero;

use crate::{
    builtins::{
        array::{find_via_predicate, ArrayIterator, Direction},
        array_buffer::{
            utils::{memcpy, memmove, SliceRefMut},
            ArrayBuffer, BufferRef,
        },
        iterable::iterable_to_list,
        BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{
        internal_methods::{get_prototype_from_constructor, integer_indexed_element_set},
        ObjectData,
    },
    property::{Attribute, PropertyNameKind},
    realm::Realm,
    string::common::StaticJsStrings,
    value::IntegerOrInfinity,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};

use super::{ContentType, IntegerIndexed, TypedArray, TypedArrayKind};

/// The JavaScript `%TypedArray%` object.
///
/// <https://tc39.es/ecma262/#sec-%typedarray%-intrinsic-object>
#[derive(Debug, Clone, Copy)]
pub(crate) struct BuiltinTypedArray;

impl IntrinsicObject for BuiltinTypedArray {
    fn init(realm: &Realm) {
        let get_species = BuiltInBuilder::callable(realm, Self::get_species)
            .name(js_string!("get [Symbol.species]"))
            .build();

        let get_buffer = BuiltInBuilder::callable(realm, Self::buffer)
            .name(js_string!("get buffer"))
            .build();

        let get_byte_length = BuiltInBuilder::callable(realm, Self::byte_length)
            .name(js_string!("get byteLength"))
            .build();

        let get_byte_offset = BuiltInBuilder::callable(realm, Self::byte_offset)
            .name(js_string!("get byteOffset"))
            .build();

        let get_length = BuiltInBuilder::callable(realm, Self::length)
            .name(js_string!("get length"))
            .build();

        let get_to_string_tag = BuiltInBuilder::callable(realm, Self::to_string_tag)
            .name(js_string!("get [Symbol.toStringTag]"))
            .build();

        let values_function = BuiltInBuilder::callable(realm, Self::values)
            .name(js_string!("values"))
            .length(0)
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_accessor(
                JsSymbol::species(),
                Some(get_species),
                None,
                Attribute::CONFIGURABLE,
            )
            .property(
                JsSymbol::iterator(),
                values_function.clone(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("buffer"),
                Some(get_buffer),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .accessor(
                utf16!("byteLength"),
                Some(get_byte_length),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .accessor(
                utf16!("byteOffset"),
                Some(get_byte_offset),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .accessor(
                utf16!("length"),
                Some(get_length),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .accessor(
                JsSymbol::to_string_tag(),
                Some(get_to_string_tag),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .static_method(Self::from, js_string!("from"), 1)
            .static_method(Self::of, js_string!("of"), 0)
            .method(Self::at, js_string!("at"), 1)
            .method(Self::copy_within, js_string!("copyWithin"), 2)
            .method(Self::entries, js_string!("entries"), 0)
            .method(Self::every, js_string!("every"), 1)
            .method(Self::fill, js_string!("fill"), 1)
            .method(Self::filter, js_string!("filter"), 1)
            .method(Self::find, js_string!("find"), 1)
            .method(Self::find_index, js_string!("findIndex"), 1)
            .method(Self::find_last, js_string!("findLast"), 1)
            .method(Self::find_last_index, js_string!("findLastIndex"), 1)
            .method(Self::foreach, js_string!("forEach"), 1)
            .method(Self::includes, js_string!("includes"), 1)
            .method(Self::index_of, js_string!("indexOf"), 1)
            .method(Self::join, js_string!("join"), 1)
            .method(Self::keys, js_string!("keys"), 0)
            .method(Self::last_index_of, js_string!("lastIndexOf"), 1)
            .method(Self::map, js_string!("map"), 1)
            .method(Self::reduce, js_string!("reduce"), 1)
            .method(Self::reduceright, js_string!("reduceRight"), 1)
            .method(Self::reverse, js_string!("reverse"), 0)
            .method(Self::set, js_string!("set"), 1)
            .method(Self::slice, js_string!("slice"), 2)
            .method(Self::some, js_string!("some"), 1)
            .method(Self::sort, js_string!("sort"), 1)
            .method(Self::subarray, js_string!("subarray"), 2)
            .method(Self::to_locale_string, js_string!("toLocaleString"), 0)
            // 23.2.3.29 %TypedArray%.prototype.toString ( )
            // The initial value of the %TypedArray%.prototype.toString data property is the same
            // built-in function object as the Array.prototype.toString method defined in 23.1.3.30.
            .property(
                js_string!("toString"),
                realm.intrinsics().objects().array_prototype_to_string(),
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .property(
                js_string!("values"),
                values_function,
                Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for BuiltinTypedArray {
    const NAME: JsString = StaticJsStrings::TYPED_ARRAY;
}

impl BuiltInConstructor for BuiltinTypedArray {
    const LENGTH: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::typed_array;

    /// `23.2.1.1 %TypedArray% ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%
    fn constructor(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Throw a TypeError exception.
        Err(JsNativeError::typ()
            .with_message("the TypedArray constructor should never be called directly")
            .into())
    }
}

impl BuiltinTypedArray {
    /// `23.2.2.1 %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.from
    fn from(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let C be the this value.
        // 2. If IsConstructor(C) is false, throw a TypeError exception.
        let constructor = match this.as_object() {
            Some(obj) if obj.is_constructor() => obj,
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("TypedArray.from called on non-constructable value")
                    .into())
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
                    return Err(JsNativeError::typ()
                        .with_message("TypedArray.from called with non-callable mapfn")
                        .into())
                }
            },
        };

        // 5. Let usingIterator be ? GetMethod(source, @@iterator).
        let source = args.get_or_undefined(0);
        let using_iterator = source.get_method(JsSymbol::iterator(), context)?;

        let this_arg = args.get_or_undefined(2);

        // 6. If usingIterator is not undefined, then
        if let Some(using_iterator) = using_iterator {
            // a. Let values be ? IterableToList(source, usingIterator).
            let values = iterable_to_list(context, source, Some(using_iterator))?;

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
    fn of(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let len be the number of elements in items.

        // 2. Let C be the this value.
        // 3. If IsConstructor(C) is false, throw a TypeError exception.
        let constructor = match this.as_object() {
            Some(obj) if obj.is_constructor() => obj,
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("TypedArray.of called on non-constructable value")
                    .into())
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
            new_obj.set(k, k_value.clone(), true, context)?;
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
    pub(super) fn get_species(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `23.2.3.1 %TypedArray%.prototype.at ( index )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.at
    pub(crate) fn at(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        drop(obj_borrow);

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
    fn buffer(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let typed_array = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;

        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        // 5. Return buffer.
        Ok(typed_array.viewed_array_buffer().clone().into())
    }

    /// `23.2.3.3 get %TypedArray%.prototype.byteLength`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-%typedarray%.prototype.bytelength
    pub(crate) fn byte_length(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let typed_array = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;

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
    pub(crate) fn byte_offset(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let typed_array = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;

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
    fn copy_within(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;

        let len = {
            let obj_borrow = obj.borrow();
            let o = obj_borrow.as_typed_array().ok_or_else(|| {
                JsNativeError::typ().with_message("Value is not a typed array object")
            })?;

            // 2. Perform ? ValidateTypedArray(O).
            if o.is_detached() {
                return Err(JsNativeError::typ()
                    .with_message("Buffer of the typed array is detached")
                    .into());
            }

            // 3. Let len be O.[[ArrayLength]].
            o.array_length()
        };

        // 4. Let relativeTarget be ? ToIntegerOrInfinity(target).
        let relative_target = args.get_or_undefined(0).to_integer_or_infinity(context)?;

        let to = match relative_target {
            // 5. If relativeTarget is -∞, let to be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 6. Else if relativeTarget < 0, let to be max(len + relativeTarget, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => len.checked_add_signed(i).unwrap_or(0),
            // 7. Else, let to be min(relativeTarget, len).
            // We can directly convert to `u64` since we covered the case where `i < 0`.
            IntegerOrInfinity::Integer(i) => std::cmp::min(i as u64, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 8. Let relativeStart be ? ToIntegerOrInfinity(start).
        let relative_start = args.get_or_undefined(1).to_integer_or_infinity(context)?;

        let from = match relative_start {
            // 9. If relativeStart is -∞, let from be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 10. Else if relativeStart < 0, let from be max(len + relativeStart, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => len.checked_add_signed(i).unwrap_or(0),
            // 11. Else, let from be min(relativeStart, len).
            // We can directly convert to `u64` since we covered the case where `i < 0`.
            IntegerOrInfinity::Integer(i) => std::cmp::min(i as u64, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 12. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
        let end = args.get_or_undefined(2);
        let r#final = if end.is_undefined() {
            len
        } else {
            match end.to_integer_or_infinity(context)? {
                // 13. If relativeEnd is -∞, let final be 0.
                IntegerOrInfinity::NegativeInfinity => 0,
                // 14. Else if relativeEnd < 0, let final be max(len + relativeEnd, 0).
                IntegerOrInfinity::Integer(i) if i < 0 => len.checked_add_signed(i).unwrap_or(0),
                // 15. Else, let final be min(relativeEnd, len).
                IntegerOrInfinity::Integer(i) => std::cmp::min(i as u64, len),
                IntegerOrInfinity::PositiveInfinity => len,
            }
        };

        // 16. Let count be min(final - from, len - to).
        let count = match (r#final.checked_sub(from), len.checked_sub(to)) {
            (Some(lhs), Some(rhs)) => std::cmp::min(lhs, rhs),
            _ => 0,
        };

        // 17. If count > 0, then
        if count > 0 {
            let obj_borrow = obj.borrow();
            let o = obj_borrow.as_typed_array().ok_or_else(|| {
                JsNativeError::typ().with_message("Value is not a typed array object")
            })?;

            // a. NOTE: The copying must be performed in a manner that preserves the bit-level encoding of the source data.
            // b. Let buffer be O.[[ViewedArrayBuffer]].
            // c. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
            let buffer_obj = o.viewed_array_buffer();
            let mut buffer_obj_borrow = buffer_obj.borrow_mut();
            let mut buffer = buffer_obj_borrow
                .as_buffer_mut()
                .expect("Already checked for detached buffer");
            let Some(buffer) = buffer.data_mut() else {
                return Err(JsNativeError::typ()
                    .with_message("Buffer of the typed array is detached")
                    .into());
            };

            // d. Let typedArrayName be the String value of O.[[TypedArrayName]].
            let kind = o.kind();

            // e. Let elementSize be the Element Size value specified in Table 73 for typedArrayName.
            let element_size = kind.element_size();

            // f. Let byteOffset be O.[[ByteOffset]].
            let byte_offset = o.byte_offset();

            // g. Let toByteIndex be to × elementSize + byteOffset.
            let to_byte_index = (to * element_size + byte_offset) as usize;

            // h. Let fromByteIndex be from × elementSize + byteOffset.
            let from_byte_index = (from * element_size + byte_offset) as usize;

            // i. Let countBytes be count × elementSize.
            let count_bytes = (count * element_size) as usize;

            // j. If fromByteIndex < toByteIndex and toByteIndex < fromByteIndex + countBytes, then
            //    ii. Set fromByteIndex to fromByteIndex + countBytes - 1.
            //    iii. Set toByteIndex to toByteIndex + countBytes - 1.
            //    i. Let direction be -1.
            // k. Else,
            //    i. Let direction be 1.
            // l. Repeat, while countBytes > 0,
            // i. Let value be GetValueFromBuffer(buffer, fromByteIndex, Uint8, true, Unordered).
            // ii. Perform SetValueInBuffer(buffer, toByteIndex, Uint8, value, true, Unordered).
            // iii. Set fromByteIndex to fromByteIndex + direction.
            // iv. Set toByteIndex to toByteIndex + direction.
            // v. Set countBytes to countBytes - 1.

            // SAFETY: All previous checks are made to ensure this memmove is always in-bounds,
            // making this operation safe.
            unsafe {
                memmove(buffer, from_byte_index, to_byte_index, count_bytes);
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
    fn entries(this: &JsValue, _: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.borrow()
            .as_typed_array()
            .ok_or_else(|| JsNativeError::typ().with_message("Value is not a typed array object"))?
            .is_detached()
        {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
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
    pub(crate) fn every(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        drop(obj_borrow);

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => {
                return Err(JsNativeError::typ()
                    .with_message(
                        "TypedArray.prototype.every called with non-callable callback function",
                    )
                    .into())
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
    pub(crate) fn fill(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        // 4. If O.[[ContentType]] is BigInt, set value to ? ToBigInt(value).
        let value: JsValue = if o.kind().content_type() == ContentType::BigInt {
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
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        drop(obj_borrow);

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
    pub(crate) fn filter(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        let typed_array_name = o.kind();

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        drop(obj_borrow);

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn =
            match args.get_or_undefined(0).as_object() {
                Some(obj) if obj.is_callable() => obj,
                _ => return Err(JsNativeError::typ()
                    .with_message(
                        "TypedArray.prototype.filter called with non-callable callback function",
                    )
                    .into()),
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
        let a = Self::species_create(obj, typed_array_name, &[captured.into()], context)?;

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
    pub(crate) fn find(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        drop(obj_borrow);

        let predicate = args.get_or_undefined(0);
        let this_arg = args.get_or_undefined(1);

        // 4. Let findRec be ? FindViaPredicate(O, len, ascending, predicate, thisArg).
        let (_, value) = find_via_predicate(
            obj,
            len,
            Direction::Ascending,
            predicate,
            this_arg,
            context,
            "TypedArray.prototype.find",
        )?;

        // 5. Return findRec.[[Value]].
        Ok(value)
    }

    /// `23.2.3.12 %TypedArray%.prototype.findIndex ( predicate [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.findindex
    pub(crate) fn find_index(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        drop(obj_borrow);

        let predicate = args.get_or_undefined(0);
        let this_arg = args.get_or_undefined(1);

        // 4. Let findRec be ? FindViaPredicate(O, len, ascending, predicate, thisArg).
        let (index, _) = find_via_predicate(
            obj,
            len,
            Direction::Ascending,
            predicate,
            this_arg,
            context,
            "TypedArray.prototype.findIndex",
        )?;

        // 5. Return findRec.[[Index]].
        Ok(index)
    }

    /// `23.2.3.13 %TypedArray%.prototype.findLast ( predicate [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.findlast
    pub(crate) fn find_last(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        let predicate = args.get_or_undefined(0);
        let this_arg = args.get_or_undefined(1);

        // 4. Let findRec be ? FindViaPredicate(O, len, descending, predicate, thisArg).
        let (_, value) = find_via_predicate(
            obj,
            len,
            Direction::Descending,
            predicate,
            this_arg,
            context,
            "TypedArray.prototype.findLast",
        )?;

        // 5. Return findRec.[[Value]].
        Ok(value)
    }

    /// `23.2.3.14 %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.findlastindex
    pub(crate) fn find_last_index(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        let predicate = args.get_or_undefined(0);
        let this_arg = args.get_or_undefined(1);

        // 4. Let findRec be ? FindViaPredicate(O, len, descending, predicate, thisArg).
        let (index, _) = find_via_predicate(
            obj,
            len,
            Direction::Descending,
            predicate,
            this_arg,
            context,
            "TypedArray.prototype.findLastIndex",
        )?;

        // 5. Return findRec.[[Index]].
        Ok(index)
    }

    /// `23.2.3.15 %TypedArray%.prototype.forEach ( callbackfn [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.foreach
    pub(crate) fn foreach(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        drop(obj_borrow);

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn =
            match args.get_or_undefined(0).as_object() {
                Some(obj) if obj.is_callable() => obj,
                _ => return Err(JsNativeError::typ()
                    .with_message(
                        "TypedArray.prototype.foreach called with non-callable callback function",
                    )
                    .into()),
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
    pub(crate) fn includes(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        drop(obj_borrow);

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
    pub(crate) fn index_of(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        drop(obj_borrow);

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
    pub(crate) fn join(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        drop(obj_borrow);

        // 4. If separator is undefined, let sep be the single-element String ",".
        let separator = args.get_or_undefined(0);
        let sep = if separator.is_undefined() {
            js_string!(",")
        // 5. Else, let sep be ? ToString(separator).
        } else {
            separator.to_string(context)?
        };

        // 6. Let R be the empty String.
        let mut r = js_string!();

        // 7. Let k be 0.
        // 8. Repeat, while k < len,
        for k in 0..len {
            // a. If k > 0, set R to the string-concatenation of R and sep.
            if k > 0 {
                r = js_string!(&r, &sep);
            }

            // b. Let element be ! Get(O, ! ToString(𝔽(k))).
            let element = obj.get(k, context).expect("Get cannot fail here");

            // c. If element is undefined, let next be the empty String; otherwise, let next be ! ToString(element).
            // d. Set R to the string-concatenation of R and next.
            if !element.is_undefined() {
                r = js_string!(&r, &element.to_string(context)?);
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
    pub(crate) fn keys(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.borrow()
            .as_typed_array()
            .ok_or_else(|| JsNativeError::typ().with_message("Value is not a typed array object"))?
            .is_detached()
        {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
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
    pub(crate) fn last_index_of(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        drop(obj_borrow);

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
    pub(crate) fn length(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
        // 3. Assert: O has [[ViewedArrayBuffer]] and [[ArrayLength]] internal slots.
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let typed_array = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;

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
    pub(crate) fn map(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        let typed_array_name = o.kind();

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        drop(obj_borrow);

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => {
                return Err(JsNativeError::typ()
                    .with_message(
                        "TypedArray.prototype.map called with non-callable callback function",
                    )
                    .into())
            }
        };

        // 5. Let A be ? TypedArraySpeciesCreate(O, « 𝔽(len) »).
        let a = Self::species_create(obj, typed_array_name, &[len.into()], context)?;

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
    pub(crate) fn reduce(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        drop(obj_borrow);

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn =
            match args.get_or_undefined(0).as_object() {
                Some(obj) if obj.is_callable() => obj,
                _ => return Err(JsNativeError::typ()
                    .with_message(
                        "TypedArray.prototype.reduce called with non-callable callback function",
                    )
                    .into()),
            };

        // 5. If len = 0 and initialValue is not present, throw a TypeError exception.
        if len == 0 && args.get(1).is_none() {
            return Err(JsNativeError::typ()
                .with_message("Typed array length is 0 and initial value is not present")
                .into());
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
    pub(crate) fn reduceright(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as i64;

        drop(obj_borrow);

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => return Err(JsNativeError::typ()
                .with_message(
                    "TypedArray.prototype.reduceright called with non-callable callback function",
                )
                .into()),
        };

        // 5. If len = 0 and initialValue is not present, throw a TypeError exception.
        if len == 0 && args.get(1).is_none() {
            return Err(JsNativeError::typ()
                .with_message("Typed array length is 0 and initial value is not present")
                .into());
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
    pub(crate) fn reverse(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length() as f64;

        drop(obj_borrow);

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
    pub(crate) fn set(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let target be the this value.
        // 2. Perform ? RequireInternalSlot(target, [[TypedArrayName]]).
        // 3. Assert: target has a [[ViewedArrayBuffer]] internal slot.
        let target = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("TypedArray.set must be called on typed array object")
        })?;
        if !target.is_typed_array() {
            return Err(JsNativeError::typ()
                .with_message("TypedArray.set must be called on typed array object")
                .into());
        }

        // 4. Let targetOffset be ? ToIntegerOrInfinity(offset).
        let target_offset = args.get_or_undefined(1).to_integer_or_infinity(context)?;

        // 5. If targetOffset < 0, throw a RangeError exception.
        let target_offset = match target_offset {
            IntegerOrInfinity::Integer(i) if i < 0 => {
                return Err(JsNativeError::range()
                    .with_message("TypedArray.set called with negative offset")
                    .into())
            }
            IntegerOrInfinity::NegativeInfinity => {
                return Err(JsNativeError::range()
                    .with_message("TypedArray.set called with negative offset")
                    .into())
            }
            IntegerOrInfinity::PositiveInfinity => U64OrPositiveInfinity::PositiveInfinity,
            IntegerOrInfinity::Integer(i) => U64OrPositiveInfinity::U64(i as u64),
        };

        let source = args.get_or_undefined(0);
        match source {
            // 6. If source is an Object that has a [[TypedArrayName]] internal slot, then
            JsValue::Object(source) if source.is_typed_array() => {
                // a. Perform ? SetTypedArrayFromTypedArray(target, targetOffset, source).
                Self::set_typed_array_from_typed_array(target, &target_offset, source, context)?;
            }
            // 7. Else,
            _ => {
                // a. Perform ? SetTypedArrayFromArrayLike(target, targetOffset, source).
                Self::set_typed_array_from_array_like(target, &target_offset, source, context)?;
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
        target_offset: &U64OrPositiveInfinity,
        source: &JsObject,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        let target_borrow = target.borrow();
        let target_array = target_borrow
            .as_typed_array()
            .expect("Target must be a typed array");

        let source_borrow = source.borrow();
        let source_array = source_borrow
            .as_typed_array()
            .expect("Source must be a typed array");

        // TODO: Implement growable buffers.
        // 1. Let targetBuffer be target.[[ViewedArrayBuffer]].
        // 2. Let targetRecord be MakeIntegerIndexedObjectWithBufferWitnessRecord(target, seq-cst).
        // 3. If IsIntegerIndexedObjectOutOfBounds(targetRecord) is true, throw a TypeError exception.
        // 4. Let targetLength be IntegerIndexedObjectLength(targetRecord).
        // 5. Let srcBuffer be source.[[ViewedArrayBuffer]].
        // 6. Let srcRecord be MakeIntegerIndexedObjectWithBufferWitnessRecord(source, seq-cst).
        // 7. If IsIntegerIndexedObjectOutOfBounds(srcRecord) is true, throw a TypeError exception.

        if target_array.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }
        let target_buffer_obj = target_array.viewed_array_buffer().clone();

        // 3. Let targetLength be target.[[ArrayLength]].
        let target_length = target_array.array_length();

        // 4. Let srcBuffer be source.[[ViewedArrayBuffer]].
        // 5. If IsDetachedBuffer(srcBuffer) is true, throw a TypeError exception.
        if source_array.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }
        let mut src_buffer_obj = source_array.viewed_array_buffer().clone();

        // 6. Let targetName be the String value of target.[[TypedArrayName]].
        // 7. Let targetType be the Element Type value in Table 73 for targetName.
        let target_type = target_array.kind();

        // 8. Let targetElementSize be the Element Size value specified in Table 73 for targetName.
        let target_element_size = target_type.element_size();

        // 9. Let targetByteOffset be target.[[ByteOffset]].
        let target_byte_offset = target_array.byte_offset();

        drop(target_borrow);

        // 10. Let srcName be the String value of source.[[TypedArrayName]].
        // 11. Let srcType be the Element Type value in Table 73 for srcName.
        let src_type = source_array.kind();

        // 12. Let srcElementSize be the Element Size value specified in Table 73 for srcName.
        let src_element_size = src_type.element_size();

        // 13. Let srcLength be source.[[ArrayLength]].
        let src_length = source_array.array_length();

        // 14. Let srcByteOffset be source.[[ByteOffset]].
        let src_byte_offset = source_array.byte_offset();

        // 15. If targetOffset is +∞, throw a RangeError exception.
        let target_offset = match target_offset {
            U64OrPositiveInfinity::U64(target_offset) => target_offset,
            U64OrPositiveInfinity::PositiveInfinity => {
                return Err(JsNativeError::range()
                    .with_message("Target offset cannot be Infinity")
                    .into());
            }
        };

        // 16. If srcLength + targetOffset > targetLength, throw a RangeError exception.
        if src_length + target_offset > target_length {
            return Err(JsNativeError::range()
                .with_message("Source typed array and target offset longer than target typed array")
                .into());
        }

        // 17. If target.[[ContentType]] ≠ source.[[ContentType]], throw a TypeError exception.
        if target_type.content_type() != src_type.content_type() {
            return Err(JsNativeError::typ()
                .with_message(
                    "Source typed array and target typed array have different content types",
                )
                .into());
        }

        // 18. If IsSharedArrayBuffer(srcBuffer) is true, IsSharedArrayBuffer(targetBuffer) is true,
        //     and srcBuffer.[[ArrayBufferData]] is targetBuffer.[[ArrayBufferData]], let
        //     sameSharedArrayBuffer be true; otherwise, let sameSharedArrayBuffer be false.
        let same = if JsObject::equals(&src_buffer_obj, &target_buffer_obj) {
            true
        } else {
            let src_buffer_obj = src_buffer_obj.borrow();
            let src_buffer = src_buffer_obj.as_buffer().expect("Must be an array buffer");

            let target_buffer_obj = target_buffer_obj.borrow();
            let target_buffer = target_buffer_obj
                .as_buffer()
                .expect("Must be an array buffer");

            match (src_buffer, target_buffer) {
                (BufferRef::Shared(src), BufferRef::Shared(dest)) => {
                    ptr::eq(src.data(), dest.data())
                }
                (_, _) => false,
            }
        };

        // 19. If SameValue(srcBuffer, targetBuffer) is true or sameSharedArrayBuffer is true, then
        let src_byte_index = if same {
            // a. Let srcByteLength be source.[[ByteLength]].
            let src_byte_offset = src_byte_offset as usize;
            let src_byte_length = source_array.byte_length() as usize;

            let s = {
                let slice = src_buffer_obj.borrow();
                let slice = slice.as_buffer().expect("Must be an array buffer");
                let slice = slice.data().expect("Already checked for detached buffer");

                // b. Set srcBuffer to ? CloneArrayBuffer(srcBuffer, srcByteOffset, srcByteLength, %ArrayBuffer%).
                // c. NOTE: %ArrayBuffer% is used to clone srcBuffer because is it known to not have any observable side-effects.
                slice
                    .subslice(src_byte_offset..src_byte_offset + src_byte_length)
                    .clone(context)?
            };
            src_buffer_obj = s;

            // d. Let srcByteIndex be 0.
            0
        }
        // 21. Else, let srcByteIndex be srcByteOffset.
        else {
            src_byte_offset
        };

        drop(source_borrow);

        // 22. Let targetByteIndex be targetOffset × targetElementSize + targetByteOffset.
        let target_byte_index = target_offset * target_element_size + target_byte_offset;

        let src_buffer = src_buffer_obj.borrow();
        let src_buffer = src_buffer.as_buffer().expect("Must be an array buffer");
        let src_buffer = src_buffer
            .data()
            .expect("Already checked for detached buffer");

        let mut target_buffer = target_buffer_obj.borrow_mut();
        let mut target_buffer = target_buffer
            .as_buffer_mut()
            .expect("Must be an array buffer");
        let mut target_buffer = target_buffer
            .data_mut()
            .expect("Already checked for detached buffer");

        // 24. If srcType is the same as targetType, then
        if src_type == target_type {
            let src_byte_index = src_byte_index as usize;
            let target_byte_index = target_byte_index as usize;
            let count = (target_element_size * src_length) as usize;

            // a. NOTE: If srcType and targetType are the same, the transfer must be performed in a manner that preserves the bit-level encoding of the source data.
            // b. Repeat, while targetByteIndex < limit,
            // i. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, Uint8, true, Unordered).
            // ii. Perform SetValueInBuffer(targetBuffer, targetByteIndex, Uint8, value, true, Unordered).
            // iii. Set srcByteIndex to srcByteIndex + 1.
            // iv. Set targetByteIndex to targetByteIndex + 1.

            // SAFETY: We already asserted that the indices are in bounds.
            unsafe {
                memcpy(
                    src_buffer.subslice(src_byte_index..),
                    target_buffer.subslice_mut(target_byte_index..),
                    count,
                );
            }
        }
        // 25. Else,
        else {
            // 23. Let limit be targetByteIndex + targetElementSize × srcLength.
            let limit = (target_byte_index + target_element_size * src_length) as usize;

            let mut src_byte_index = src_byte_index as usize;
            let mut target_byte_index = target_byte_index as usize;

            // a. Repeat, while targetByteIndex < limit,
            while target_byte_index < limit {
                // i. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, srcType, true, Unordered).

                let value = unsafe {
                    src_buffer
                        .subslice(src_byte_index..)
                        .get_value(src_type, atomic::Ordering::Relaxed)
                };

                let value = JsValue::from(value);

                let value = target_type
                    .get_element(&value, context)
                    .expect("value can only be f64 or BigInt");

                // ii. Perform SetValueInBuffer(targetBuffer, targetByteIndex, targetType, value, true, Unordered).
                unsafe {
                    target_buffer
                        .subslice_mut(target_byte_index..)
                        .set_value(value, atomic::Ordering::Relaxed);
                }

                // iii. Set srcByteIndex to srcByteIndex + srcElementSize.
                src_byte_index += src_element_size as usize;

                // iv. Set targetByteIndex to targetByteIndex + targetElementSize.
                target_byte_index += target_element_size as usize;
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
        target_offset: &U64OrPositiveInfinity,
        source: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        let target_length = {
            let target_borrow = target.borrow();
            let target_array = target_borrow
                .as_typed_array()
                .expect("Target must be a typed array");

            // 1. Let targetBuffer be target.[[ViewedArrayBuffer]].
            // 2. If IsDetachedBuffer(targetBuffer) is true, throw a TypeError exception.
            if target_array.is_detached() {
                return Err(JsNativeError::typ()
                    .with_message("Buffer of the typed array is detached")
                    .into());
            }

            // 3. Let targetLength be target.[[ArrayLength]].
            target_array.array_length()
        };

        // 4. Let src be ? ToObject(source).
        let src = source.to_object(context)?;

        // 5. Let srcLength be ? LengthOfArrayLike(src).
        let src_length = src.length_of_array_like(context)?;

        // 6. If targetOffset = +∞, throw a RangeError exception.
        let target_offset = match target_offset {
            U64OrPositiveInfinity::U64(target_offset) => target_offset,
            U64OrPositiveInfinity::PositiveInfinity => {
                return Err(JsNativeError::range()
                    .with_message("Target offset cannot be positive infinity")
                    .into())
            }
        };

        // 7. If srcLength + targetOffset > targetLength, throw a RangeError exception.
        if src_length + target_offset > target_length {
            return Err(JsNativeError::range()
                .with_message("Source object and target offset longer than target typed array")
                .into());
        }

        // 8. Let k be 0.
        // 9. Repeat, while k < srcLength,
        for k in 0..src_length {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let value be ? Get(src, Pk).
            let value = src.get(k, context)?;

            // c. Let targetIndex be 𝔽(targetOffset + k).
            let target_index = target_offset + k;

            // d. Perform ? IntegerIndexedElementSet(target, targetIndex, value).
            integer_indexed_element_set(target, target_index as f64, &value, context)?;

            // e. Set k to k + 1.
        }

        // 10. Return unused.
        Ok(())
    }

    /// `23.2.3.25 %TypedArray%.prototype.slice ( start, end )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.slice
    pub(crate) fn slice(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
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
        let count = std::cmp::max(r#final - k, 0) as u64;

        // 13. Let A be ? TypedArraySpeciesCreate(O, « 𝔽(count) »).
        let a = Self::species_create(obj, o.kind(), &[count.into()], context)?;
        let a_borrow = a.borrow();
        let a_array = a_borrow
            .as_typed_array()
            .expect("This must be a typed array");

        // 14. If count > 0, then
        if count > 0 {
            // a. If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
            if o.is_detached() {
                return Err(JsNativeError::typ()
                    .with_message("Buffer of the typed array is detached")
                    .into());
            }

            // b. Let srcName be the String value of O.[[TypedArrayName]].
            // c. Let srcType be the Element Type value in Table 73 for srcName.
            let src_type = o.kind();

            // d. Let targetName be the String value of A.[[TypedArrayName]].
            // e. Let targetType be the Element Type value in Table 73 for targetName.
            let target_type = a_array.kind();

            // f. If srcType is different from targetType, then
            #[allow(clippy::if_not_else)]
            if src_type != target_type {
                drop(obj_borrow);
                drop(a_borrow);

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
                let src_buffer_obj = o.viewed_array_buffer();
                let src_buffer_obj_borrow = src_buffer_obj.borrow();
                let src_buffer = src_buffer_obj_borrow
                    .as_buffer()
                    .expect("view must be a buffer");
                let src_buffer = src_buffer.data().expect("cannot be detached here");

                // ii. Let targetBuffer be A.[[ViewedArrayBuffer]].
                let target_buffer_obj = a_array.viewed_array_buffer();
                let mut target_buffer_obj_borrow = target_buffer_obj.borrow_mut();
                let mut target_buffer = target_buffer_obj_borrow
                    .as_buffer_mut()
                    .expect("view must be a buffer");
                let mut target_buffer = target_buffer.data_mut().expect("cannot be detached here");

                // iii. Let elementSize be the Element Size value specified in Table 73 for Element Type srcType.
                let element_size = o.kind().element_size();

                // iv. NOTE: If srcType and targetType are the same, the transfer must be performed in a manner that preserves the bit-level encoding of the source data.

                // v. Let srcByteOffset be O.[[ByteOffset]].
                let src_byte_offset = o.byte_offset();

                // vi. Let targetByteIndex be A.[[ByteOffset]].
                let target_byte_index = (a_array.byte_offset()) as usize;

                // vii. Let srcByteIndex be (k × elementSize) + srcByteOffset.
                let src_byte_index = (k as u64 * element_size + src_byte_offset) as usize;

                let byte_count = (count * element_size) as usize;

                // viii. Let limit be targetByteIndex + count × elementSize.
                // ix. Repeat, while targetByteIndex < limit,
                // 1. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, Uint8, true, Unordered).
                // 2. Perform SetValueInBuffer(targetBuffer, targetByteIndex, Uint8, value, true, Unordered).
                // 3. Set srcByteIndex to srcByteIndex + 1.
                // 4. Set targetByteIndex to targetByteIndex + 1.

                // SAFETY: All previous checks put the indices at least within the bounds of `src_buffer`.
                // Also, `target_buffer` is precisely allocated to fit all sliced elements from
                // `src_buffer`, making this operation safe.
                unsafe {
                    memcpy(
                        src_buffer.subslice(src_byte_index..),
                        target_buffer.subslice_mut(target_byte_index..),
                        byte_count,
                    );
                }

                drop(target_buffer_obj_borrow);
                drop(a_borrow);
            }
        } else {
            drop(a_borrow);
        }

        // 15. Return A.
        Ok(a.into())
    }

    /// `23.2.3.26 %TypedArray%.prototype.some ( callbackfn [ , thisArg ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.some
    pub(crate) fn some(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. Let len be O.[[ArrayLength]].
        let len = o.array_length();

        drop(obj_borrow);

        // 4. If IsCallable(callbackfn) is false, throw a TypeError exception.
        let callback_fn = match args.get_or_undefined(0).as_object() {
            Some(obj) if obj.is_callable() => obj,
            _ => {
                return Err(JsNativeError::typ()
                    .with_message(
                        "TypedArray.prototype.some called with non-callable callback function",
                    )
                    .into())
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
    pub(crate) fn sort(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If comparefn is not undefined and IsCallable(comparefn) is false, throw a TypeError exception.
        let compare_fn = match args.get(0) {
            None | Some(JsValue::Undefined) => None,
            Some(JsValue::Object(obj)) if obj.is_callable() => Some(obj),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("TypedArray.sort called with non-callable comparefn")
                    .into())
            }
        };

        // 2. Let obj be the this value.
        // 3. Perform ? ValidateTypedArray(obj).
        // 4. Let len be obj.[[ArrayLength]].
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("TypedArray.sort must be called on typed array object")
        })?;
        let len =
            {
                let obj_borrow = obj.borrow();
                let o = obj_borrow.as_typed_array().ok_or_else(|| {
                    JsNativeError::typ()
                        .with_message("TypedArray.sort must be called on typed array object")
                })?;
                if o.is_detached() {
                    return Err(JsNativeError::typ().with_message(
                "TypedArray.sort called on typed array object with detached array buffer",
            ).into());
                }

                o.array_length()
            };

        // 5. NOTE: The following closure performs a numeric comparison rather than the string comparison used in 23.1.3.30.
        // 6. Let SortCompare be a new Abstract Closure with parameters (x, y) that captures comparefn and performs the following steps when called:
        let sort_compare = |x: &JsValue,
                            y: &JsValue,
                            compare_fn: Option<&JsObject>,
                            context: &mut Context<'_>|
         -> JsResult<Ordering> {
            // a. Return ? CompareTypedArrayElements(x, y, comparefn).
            compare_typed_array_elements(x, y, compare_fn, context)
        };

        // Note: This step is currently inlined.
        // 7. Let sortedList be ? SortIndexedProperties(obj, len, SortCompare, read-through-holes).
        // 1. Let items be a new empty List.
        let mut items = Vec::with_capacity(len as usize);

        // 2. Let k be 0.
        // 3. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. If holes is skip-holes, then
            // i. Let kRead be ? HasProperty(obj, Pk).
            // c. Else,
            // i. Assert: holes is read-through-holes.
            // ii. Let kRead be true.
            // d. If kRead is true, then
            // i. Let kValue be ? Get(obj, Pk).
            let k_value = obj.get(k, context)?;

            // ii. Append kValue to items.
            items.push(k_value);
            // e. Set k to k + 1.
        }

        // 4. Sort items using an implementation-defined sequence of calls to SortCompare. If any such call returns an abrupt completion, stop before performing any further calls to SortCompare and return that Completion Record.
        // 5. Return items.
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

        // 8. Let j be 0.
        // 9. Repeat, while j < len,
        for (j, item) in items.into_iter().enumerate() {
            // a. Perform ! Set(obj, ! ToString(𝔽(j)), sortedList[j], true).
            obj.set(j, item, true, context)
                .expect("cannot fail per spec");

            // b. Set j to j + 1.
        }

        // 10. Return obj.
        Ok(obj.clone().into())
    }

    /// `23.2.3.28 %TypedArray%.prototype.subarray ( begin, end )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.subarray
    pub(crate) fn subarray(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        let obj_borrow = obj.borrow();
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;

        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        let buffer = o.viewed_array_buffer();

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
        let element_size = o.kind().element_size();

        // 17. Let srcByteOffset be O.[[ByteOffset]].
        let src_byte_offset = o.byte_offset();

        // 18. Let beginByteOffset be srcByteOffset + beginIndex × elementSize.
        let begin_byte_offset = src_byte_offset + begin_index as u64 * element_size;

        // 19. Let argumentsList be « buffer, 𝔽(beginByteOffset), 𝔽(newLength) ».
        // 20. Return ? TypedArraySpeciesCreate(O, argumentsList).
        Ok(Self::species_create(
            obj,
            o.kind(),
            &[
                buffer.clone().into(),
                begin_byte_offset.into(),
                new_length.into(),
            ],
            context,
        )?
        .into())
    }

    /// `%TypedArray%.prototype.toLocaleString ( [ reserved1 [ , reserved2 ] ] )`
    /// `Array.prototype.toLocaleString ( [ locales [ , options ] ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [ECMA-402 reference][spec-402]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.tolocalestring
    /// [spec-402]: https://402.ecma-international.org/10.0/#sup-array.prototype.tolocalestring
    fn to_locale_string(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let array be ? ToObject(this value).
        // Note: ValidateTypedArray is applied to the this value prior to evaluating the algorithm.
        let array = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;

        let len = {
            let obj_borrow = array.borrow();
            let o = obj_borrow.as_typed_array().ok_or_else(|| {
                JsNativeError::typ().with_message("Value is not a typed array object")
            })?;
            if o.is_detached() {
                return Err(JsNativeError::typ()
                    .with_message("Buffer of the typed array is detached")
                    .into());
            }

            // 2. Let len be array.[[ArrayLength]]
            o.array_length()
        };

        // 3. Let separator be the implementation-defined list-separator String value
        //    appropriate for the host environment's current locale (such as ", ").
        let separator = {
            #[cfg(feature = "intl")]
            {
                // TODO: this should eventually return a locale-sensitive separator.
                utf16!(", ")
            }

            #[cfg(not(feature = "intl"))]
            {
                utf16!(", ")
            }
        };

        // 4. Let R be the empty String.
        let mut r = Vec::new();

        // 5. Let k be 0.
        // 6. Repeat, while k < len,
        for k in 0..len {
            // a. If k > 0, then
            if k > 0 {
                // i. Set R to the string-concatenation of R and separator.
                r.extend_from_slice(separator);
            }

            // b. Let nextElement be ? Get(array, ! ToString(k)).
            let next_element = array.get(k, context)?;

            // c. If nextElement is not undefined or null, then
            if !next_element.is_null_or_undefined() {
                // i. Let S be ? ToString(? Invoke(nextElement, "toLocaleString", « locales, options »)).
                let s = next_element
                    .invoke(
                        utf16!("toLocaleString"),
                        &[
                            args.get_or_undefined(0).clone(),
                            args.get_or_undefined(1).clone(),
                        ],
                        context,
                    )?
                    .to_string(context)?;

                // ii. Set R to the string-concatenation of R and S.
                r.extend_from_slice(&s);
            }

            // d. Increase k by 1.
        }

        // 7. Return R.
        Ok(js_string!(r).into())
    }

    /// `23.2.3.31 %TypedArray%.prototype.values ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-%typedarray%.prototype.values
    fn values(this: &JsValue, _: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? ValidateTypedArray(O).
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.borrow()
            .as_typed_array()
            .ok_or_else(|| JsNativeError::typ().with_message("Value is not a typed array object"))?
            .is_detached()
        {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
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
    fn to_string_tag(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
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
                    .map(|o| o.kind().js_name().into())
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
        kind: TypedArrayKind,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // 1. Let defaultConstructor be the intrinsic object listed in column one of Table 73 for exemplar.[[TypedArrayName]].
        let default_constructor = kind.standard_constructor();

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
            .kind()
            .content_type()
            != kind.content_type()
        {
            return Err(JsNativeError::typ()
                .with_message("New typed array has different context type than exemplar")
                .into());
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
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // 1. Let newTypedArray be ? Construct(constructor, argumentList).
        let new_typed_array = constructor.construct(args, Some(constructor), context)?;

        let obj_borrow = new_typed_array.borrow();
        // 2. Perform ? ValidateTypedArray(newTypedArray).
        let o = obj_borrow.as_typed_array().ok_or_else(|| {
            JsNativeError::typ().with_message("Value is not a typed array object")
        })?;
        if o.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer of the typed array is detached")
                .into());
        }

        // 3. If argumentList is a List of a single Number, then
        if args.len() == 1 {
            if let Some(number) = args[0].as_number() {
                // a. If newTypedArray.[[ArrayLength]] < ℝ(argumentList[0]), throw a TypeError exception.
                if (o.array_length() as f64) < number {
                    return Err(JsNativeError::typ()
                        .with_message("New typed array length is smaller than expected")
                        .into());
                }
            }
        }

        // 4. Return newTypedArray.
        Ok(new_typed_array.clone())
    }

    /// <https://tc39.es/ecma262/#sec-allocatetypedarraybuffer>
    fn allocate_buffer<T: TypedArray>(
        length: u64,
        context: &mut Context<'_>,
    ) -> JsResult<IntegerIndexed> {
        // 1. Assert: O.[[ViewedArrayBuffer]] is undefined.

        // 2. Let constructorName be the String value of O.[[TypedArrayName]].
        // 3. Let elementSize be the Element Size value specified in Table 73 for constructorName.
        let element_size = T::ERASED.element_size();

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
        // 7. Set O.[[ByteLength]] to byteLength.
        // 8. Set O.[[ByteOffset]] to 0.
        // 9. Set O.[[ArrayLength]] to length.

        // 10. Return O.
        Ok(IntegerIndexed::new(data, T::ERASED, 0, byte_length, length))
    }

    /// <https://tc39.es/ecma262/#sec-initializetypedarrayfromlist>
    pub(crate) fn initialize_from_list<T: TypedArray>(
        proto: JsObject,
        values: Vec<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // 1. Let len be the number of elements in values.
        let len = values.len() as u64;
        // 2. Perform ? AllocateTypedArrayBuffer(O, len).
        let buf = Self::allocate_buffer::<T>(len, context)?;
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ObjectData::integer_indexed(buf),
        );

        // 3. Let k be 0.
        // 4. Repeat, while k < len,
        for (k, k_value) in values.into_iter().enumerate() {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be the first element of values and remove that element from values.
            // c. Perform ? Set(O, Pk, kValue, true).
            obj.set(k, k_value, true, context)?;
            // d. Set k to k + 1.
        }

        // 5. Assert: values is now an empty List.
        // It no longer exists.
        Ok(obj)
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
    pub(super) fn allocate<T: TypedArray>(
        new_target: &JsValue,
        length: u64,
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // 1. Let proto be ? GetPrototypeFromConstructor(newTarget, defaultProto).
        let proto = get_prototype_from_constructor(new_target, T::STANDARD_CONSTRUCTOR, context)?;

        // 3. Assert: obj.[[ViewedArrayBuffer]] is undefined.
        // 4. Set obj.[[TypedArrayName]] to constructorName.
        // 5. If constructorName is "BigInt64Array" or "BigUint64Array", set obj.[[ContentType]] to BigInt.
        // 6. Otherwise, set obj.[[ContentType]] to Number.
        // 7. If length is not present, then
        // a. Set obj.[[ByteLength]] to 0.
        // b. Set obj.[[ByteOffset]] to 0.
        // c. Set obj.[[ArrayLength]] to 0.

        // 8. Else,
        // a. Perform ? AllocateTypedArrayBuffer(obj, length).
        let indexed = Self::allocate_buffer::<T>(length, context)?;

        // 2. Let obj be ! IntegerIndexedObjectCreate(proto).
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ObjectData::integer_indexed(indexed),
        );

        // 9. Return obj.
        Ok(obj)
    }

    /// `23.2.5.1.2 InitializeTypedArrayFromTypedArray ( O, srcArray )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-initializetypedarrayfromtypedarray
    pub(super) fn initialize_from_typed_array<T: TypedArray>(
        proto: JsObject,
        src_array: &JsObject,
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        let src_array = src_array.borrow();
        let src_array = src_array
            .as_typed_array()
            .expect("this must be a typed array");
        let src_data = src_array.viewed_array_buffer();
        let src_data = src_data.borrow();
        let src_data = src_data
            .as_buffer()
            .expect("integer indexed must have a buffer");

        // 1. Let srcData be srcArray.[[ViewedArrayBuffer]].
        // 2. If IsDetachedBuffer(srcData) is true, throw a TypeError exception.
        let Some(src_data) = src_data.data() else {
            return Err(JsNativeError::typ()
                .with_message("Cannot initialize typed array from detached buffer")
                .into());
        };

        // 3. Let elementType be TypedArrayElementType(O).
        let element_type = T::ERASED;

        // 4. Let elementSize be TypedArrayElementSize(O).
        let target_element_size = element_type.element_size();

        // 5. Let srcType be TypedArrayElementType(srcArray).
        let src_type = src_array.kind();

        // 6. Let srcElementSize be TypedArrayElementSize(srcArray).
        let src_element_size = src_type.element_size();

        // 7. Let srcByteOffset be srcArray.[[ByteOffset]].
        let src_byte_offset = src_array.byte_offset();

        // 8. Let elementLength be srcArray.[[ArrayLength]].
        let element_length = src_array.array_length();

        // 9. Let byteLength be elementSize × elementLength.
        let byte_length = target_element_size * element_length;

        // 10. If elementType is srcType, then
        let new_buffer = if element_type == src_type {
            let start = src_byte_offset as usize;
            let end = src_byte_offset as usize;
            // a. Let data be ? CloneArrayBuffer(srcData, srcByteOffset, byteLength).
            src_data.subslice(start..start + end).clone(context)?
        }
        // 11. Else,
        else {
            // a. Let data be ? AllocateArrayBuffer(%ArrayBuffer%, byteLength).
            let data_obj = ArrayBuffer::allocate(
                &context
                    .realm()
                    .intrinsics()
                    .constructors()
                    .array_buffer()
                    .constructor()
                    .into(),
                byte_length,
                context,
            )?;
            let mut data_obj_b = data_obj.borrow_mut();
            let data = data_obj_b
                .as_array_buffer_mut()
                .expect("Must be ArrayBuffer");
            let mut data =
                SliceRefMut::Common(data.data_mut().expect("a new buffer cannot be detached"));

            // b. If srcArray.[[ContentType]] is not O.[[ContentType]], throw a TypeError exception.
            if src_type.content_type() != element_type.content_type() {
                return Err(JsNativeError::typ()
                    .with_message("Cannot initialize typed array from different content type")
                    .into());
            }

            let src_element_size = src_element_size as usize;
            let target_element_size = target_element_size as usize;

            // c. Let srcByteIndex be srcByteOffset.
            let mut src_byte_index = src_byte_offset as usize;

            // d. Let targetByteIndex be 0.
            let mut target_byte_index = 0;

            // e. Let count be elementLength.
            let mut count = element_length;

            // f. Repeat, while count > 0,
            while count > 0 {
                // i. Let value be GetValueFromBuffer(srcData, srcByteIndex, srcType, true, Unordered).
                // SAFETY: All integer indexed objects are always in-bounds and properly
                // aligned to their underlying buffer.
                let value = unsafe {
                    src_data
                        .subslice(src_byte_index..)
                        .get_value(src_type, atomic::Ordering::Relaxed)
                };

                let value = JsValue::from(value);

                // TODO: cast between types instead of converting to `JsValue`.
                let value = element_type
                    .get_element(&value, context)
                    .expect("value must be bigint or float");

                // ii. Perform SetValueInBuffer(data, targetByteIndex, elementType, value, true, Unordered).

                // SAFETY: The newly created buffer has at least `element_size * element_length`
                // bytes available, which makes `target_byte_index` always in-bounds.
                unsafe {
                    data.subslice_mut(target_byte_index..)
                        .set_value(value, atomic::Ordering::Relaxed);
                }

                // iii. Set srcByteIndex to srcByteIndex + srcElementSize.
                src_byte_index += src_element_size;

                // iv. Set targetByteIndex to targetByteIndex + elementSize.
                target_byte_index += target_element_size;

                // v. Set count to count - 1.
                count -= 1;
            }

            drop(data_obj_b);
            data_obj
        };

        // 12. Set O.[[ViewedArrayBuffer]] to data.
        // 13. Set O.[[ByteLength]] to byteLength.
        // 14. Set O.[[ByteOffset]] to 0.
        // 15. Set O.[[ArrayLength]] to elementLength.
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ObjectData::integer_indexed(IntegerIndexed::new(
                new_buffer,
                element_type,
                0,
                byte_length,
                element_length,
            )),
        );

        // 16. Return unused.
        Ok(obj)
    }

    /// `23.2.5.1.3 InitializeTypedArrayFromArrayBuffer ( O, buffer, byteOffset, length )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-initializetypedarrayfromarraybuffer
    pub(super) fn initialize_from_array_buffer<T: TypedArray>(
        proto: JsObject,
        buffer: JsObject,
        byte_offset: &JsValue,
        length: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // 1. Let elementSize be TypedArrayElementSize(O).
        let element_size = T::ERASED.element_size();

        // 2. Let offset be ? ToIndex(byteOffset).
        let offset = byte_offset.to_index(context)?;

        // 3. If offset modulo elementSize ≠ 0, throw a RangeError exception.
        if offset % element_size != 0 {
            return Err(JsNativeError::range()
                .with_message("Invalid offset for typed array")
                .into());
        }

        // 4. If length is not undefined, then
        let new_length = if length.is_undefined() {
            None
        } else {
            // a. Let newLength be ? ToIndex(length).
            Some(length.to_index(context)?)
        };

        // 5. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        // 6. Let bufferByteLength be buffer.[[ArrayBufferByteLength]].
        let buffer_byte_length = {
            let buffer_borrow = buffer.borrow();
            let buffer_array = buffer_borrow.as_buffer().expect("Must be a buffer");

            let Some(data) = buffer_array.data() else {
                return Err(JsNativeError::typ()
                    .with_message("Cannot construct typed array from detached buffer")
                    .into());
            };

            data.len() as u64
        };

        // 7. If length is undefined, then
        // 8. Else,
        let new_byte_length = if let Some(new_length) = new_length {
            // a. Let newByteLength be newLength × elementSize.
            let new_byte_length = new_length * element_size;

            // b. If offset + newByteLength > bufferByteLength, throw a RangeError exception.
            if offset + new_byte_length > buffer_byte_length {
                return Err(JsNativeError::range()
                    .with_message("Invalid length for typed array")
                    .into());
            }

            new_byte_length
        } else {
            // a. If bufferByteLength modulo elementSize ≠ 0, throw a RangeError exception.
            if buffer_byte_length % element_size != 0 {
                return Err(JsNativeError::range()
                    .with_message("Invalid length for typed array")
                    .into());
            }

            // b. Let newByteLength be bufferByteLength - offset.
            let new_byte_length = buffer_byte_length as i64 - offset as i64;

            // c. If newByteLength < 0, throw a RangeError exception.
            if new_byte_length < 0 {
                return Err(JsNativeError::range()
                    .with_message("Invalid length for typed array")
                    .into());
            }

            new_byte_length as u64
        };

        // 9. Set O.[[ViewedArrayBuffer]] to buffer.
        // 10. Set O.[[ByteLength]] to newByteLength.
        // 11. Set O.[[ByteOffset]] to offset.
        // 12. Set O.[[ArrayLength]] to newByteLength / elementSize.
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ObjectData::integer_indexed(IntegerIndexed::new(
                buffer,
                T::ERASED,
                offset,
                new_byte_length,
                new_byte_length / element_size,
            )),
        );

        // 13. Return unused.
        Ok(obj)
    }

    /// `23.2.5.1.5 InitializeTypedArrayFromArrayLike ( O, arrayLike )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-initializetypedarrayfromarraylike
    pub(super) fn initialize_from_array_like<T: TypedArray>(
        proto: JsObject,
        array_like: &JsObject,
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // 1. Let len be ? LengthOfArrayLike(arrayLike).
        let len = array_like.length_of_array_like(context)?;

        // 2. Perform ? AllocateTypedArrayBuffer(O, len).
        let buf = Self::allocate_buffer::<T>(len, context)?;
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ObjectData::integer_indexed(buf),
        );

        // 3. Let k be 0.
        // 4. Repeat, while k < len,
        for k in 0..len {
            // a. Let Pk be ! ToString(𝔽(k)).
            // b. Let kValue be ? Get(arrayLike, Pk).
            let k_value = array_like.get(k, context)?;

            // c. Perform ? Set(O, Pk, kValue, true).
            obj.set(k, k_value, true, context)?;
        }

        Ok(obj)
    }
}

/// `CompareTypedArrayElements ( x, y, comparefn )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-comparetypedarrayelements
fn compare_typed_array_elements(
    x: &JsValue,
    y: &JsValue,
    compare_fn: Option<&JsObject>,
    context: &mut Context<'_>,
) -> JsResult<Ordering> {
    // 1. Assert: x is a Number and y is a Number, or x is a BigInt and y is a BigInt.

    // 2. If comparefn is not undefined, then
    if let Some(compare_fn) = compare_fn {
        // a. Let v be ? ToNumber(? Call(comparefn, undefined, « x, y »)).
        let v = compare_fn
            .call(&JsValue::undefined(), &[x.clone(), y.clone()], context)?
            .to_number(context)?;

        // b. If v is NaN, return +0𝔽.
        if v.is_nan() {
            return Ok(Ordering::Equal);
        }

        // c. Return v.
        if v.is_sign_positive() {
            return Ok(Ordering::Greater);
        }
        return Ok(Ordering::Less);
    }

    match (x, y) {
        (JsValue::BigInt(x), JsValue::BigInt(y)) => {
            // Note: Other steps are not relevant for BigInts.
            // 6. If x < y, return -1𝔽.
            // 7. If x > y, return 1𝔽.
            // 10. Return +0𝔽.
            Ok(x.cmp(y))
        }
        (JsValue::Integer(x), JsValue::Integer(y)) => {
            // Note: Other steps are not relevant for integers.
            // 6. If x < y, return -1𝔽.
            // 7. If x > y, return 1𝔽.
            // 10. Return +0𝔽.
            Ok(x.cmp(y))
        }
        (JsValue::Rational(x), JsValue::Rational(y)) => {
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
            if x.is_sign_negative() && x.is_zero() && y.is_sign_positive() && y.is_zero() {
                return Ok(Ordering::Less);
            }

            // 9. If x is +0𝔽 and y is -0𝔽, return 1𝔽.
            if x.is_sign_positive() && x.is_zero() && y.is_sign_negative() && y.is_zero() {
                return Ok(Ordering::Greater);
            }

            // 10. Return +0𝔽.
            Ok(Ordering::Equal)
        }
        _ => unreachable!("x and y must be both Numbers or BigInts"),
    }
}

enum U64OrPositiveInfinity {
    U64(u64),
    PositiveInfinity,
}
