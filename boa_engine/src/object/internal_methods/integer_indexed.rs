use crate::{
    builtins::{array_buffer::SharedMemoryOrder, typed_array::integer_indexed_object::ContentType},
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
    Context, JsResult, JsValue,
};

use super::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS};

/// Definitions of the internal object methods for integer-indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects
pub(crate) static INTEGER_INDEXED_EXOTIC_INTERNAL_METHODS: InternalObjectMethods =
    InternalObjectMethods {
        __get_own_property__: integer_indexed_exotic_get_own_property,
        __has_property__: integer_indexed_exotic_has_property,
        __define_own_property__: integer_indexed_exotic_define_own_property,
        __get__: integer_indexed_exotic_get,
        __set__: integer_indexed_exotic_set,
        __delete__: integer_indexed_exotic_delete,
        __own_property_keys__: integer_indexed_exotic_own_property_keys,
        ..ORDINARY_INTERNAL_METHODS
    };

/// `[[GetOwnProperty]]` internal method for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-getownproperty-p
#[inline]
pub(crate) fn integer_indexed_exotic_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context,
) -> JsResult<Option<PropertyDescriptor>> {
    // 1. If Type(P) is String, then
    // a. Let numericIndex be ! CanonicalNumericIndexString(P).
    // b. If numericIndex is not undefined, then
    if let PropertyKey::Index(index) = key {
        // i. Let value be ! IntegerIndexedElementGet(O, numericIndex).
        // ii. If value is undefined, return undefined.
        // iii. Return the PropertyDescriptor { [[Value]]: value, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true }.
        Ok(
            integer_indexed_element_get(obj, u64::from(*index)).map(|v| {
                PropertyDescriptor::builder()
                    .value(v)
                    .writable(true)
                    .enumerable(true)
                    .configurable(true)
                    .build()
            }),
        )
    } else {
        // 2. Return OrdinaryGetOwnProperty(O, P).
        super::ordinary_get_own_property(obj, key, context)
    }
}

/// `[[HasProperty]]` internal method for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-hasproperty-p
#[inline]
pub(crate) fn integer_indexed_exotic_has_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. If Type(P) is String, then
    // a. Let numericIndex be ! CanonicalNumericIndexString(P).
    if let PropertyKey::Index(index) = key {
        // b. If numericIndex is not undefined, return ! IsValidIntegerIndex(O, numericIndex).
        Ok(is_valid_integer_index(obj, u64::from(*index)))
    } else {
        // 2. Return ? OrdinaryHasProperty(O, P).
        super::ordinary_has_property(obj, key, context)
    }
}

/// `[[DefineOwnProperty]]` internal method for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-defineownproperty-p-desc
#[inline]
pub(crate) fn integer_indexed_exotic_define_own_property(
    obj: &JsObject,
    key: PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. If Type(P) is String, then
    // a. Let numericIndex be ! CanonicalNumericIndexString(P).
    // b. If numericIndex is not undefined, then
    if let PropertyKey::Index(index) = key {
        // i. If ! IsValidIntegerIndex(O, numericIndex) is false, return false.
        // ii. If Desc has a [[Configurable]] field and if Desc.[[Configurable]] is false, return false.
        // iii. If Desc has an [[Enumerable]] field and if Desc.[[Enumerable]] is false, return false.
        // v. If Desc has a [[Writable]] field and if Desc.[[Writable]] is false, return false.
        // iv. If ! IsAccessorDescriptor(Desc) is true, return false.
        if !is_valid_integer_index(obj, u64::from(index))
            || !desc
                .configurable()
                .or_else(|| desc.enumerable())
                .or_else(|| desc.writable())
                .unwrap_or(true)
            || desc.is_accessor_descriptor()
        {
            return Ok(false);
        }

        // vi. If Desc has a [[Value]] field, perform ? IntegerIndexedElementSet(O, numericIndex, Desc.[[Value]]).
        if let Some(value) = desc.value() {
            integer_indexed_element_set(obj, index as usize, value, context)?;
        }

        // vii. Return true.
        Ok(true)
    } else {
        // 2. Return ! OrdinaryDefineOwnProperty(O, P, Desc).
        super::ordinary_define_own_property(obj, key, desc, context)
    }
}

/// Internal method `[[Get]]` for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-get-p-receiver
#[inline]
pub(crate) fn integer_indexed_exotic_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. If Type(P) is String, then
    // a. Let numericIndex be ! CanonicalNumericIndexString(P).
    // b. If numericIndex is not undefined, then
    if let PropertyKey::Index(index) = key {
        // i. Return ! IntegerIndexedElementGet(O, numericIndex).
        Ok(integer_indexed_element_get(obj, u64::from(*index)).unwrap_or_default())
    } else {
        // 2. Return ? OrdinaryGet(O, P, Receiver).
        super::ordinary_get(obj, key, receiver, context)
    }
}

/// Internal method `[[Set]]` for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-set-p-v-receiver
#[inline]
pub(crate) fn integer_indexed_exotic_set(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. If Type(P) is String, then
    // a. Let numericIndex be ! CanonicalNumericIndexString(P).
    // b. If numericIndex is not undefined, then
    if let PropertyKey::Index(index) = key {
        // i. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
        integer_indexed_element_set(obj, index as usize, &value, context)?;

        // ii. Return true.
        Ok(true)
    } else {
        // 2. Return ? OrdinarySet(O, P, V, Receiver).
        super::ordinary_set(obj, key, value, receiver, context)
    }
}

/// Internal method `[[Delete]]` for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-delete-p
#[inline]
pub(crate) fn integer_indexed_exotic_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. If Type(P) is String, then
    // a. Let numericIndex be ! CanonicalNumericIndexString(P).
    // b. If numericIndex is not undefined, then
    if let PropertyKey::Index(index) = key {
        // i. If ! IsValidIntegerIndex(O, numericIndex) is false, return true; else return false.
        Ok(!is_valid_integer_index(obj, u64::from(*index)))
    } else {
        // 2. Return ? OrdinaryDelete(O, P).
        super::ordinary_delete(obj, key, context)
    }
}

/// Internal method `[[OwnPropertyKeys]]` for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-ownpropertykeys
#[inline]
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn integer_indexed_exotic_own_property_keys(
    obj: &JsObject,
    _context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    let obj = obj.borrow();
    let inner = obj.as_typed_array().expect(
        "integer indexed exotic method should only be callable from integer indexed objects",
    );

    // 1. Let keys be a new empty List.
    let mut keys = if inner.is_detached() {
        vec![]
    } else {
        // 2. If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is false, then
        // a. For each integer i starting with 0 such that i < O.[[ArrayLength]], in ascending order, do
        // i. Add ! ToString(𝔽(i)) as the last element of keys.
        (0..inner.array_length())
            .into_iter()
            .map(|index| PropertyKey::Index(index as u32))
            .collect()
    };

    // 3. For each own property key P of O such that Type(P) is String and P is not an array index, in ascending chronological order of property creation, do
    // a. Add P as the last element of keys.
    keys.extend(
        obj.properties
            .string_property_keys()
            .cloned()
            .map(Into::into),
    );

    // 4. For each own property key P of O such that Type(P) is Symbol, in ascending chronological order of property creation, do
    // a. Add P as the last element of keys.
    keys.extend(
        obj.properties
            .symbol_property_keys()
            .cloned()
            .map(Into::into),
    );

    // 5. Return keys.
    Ok(keys)
}

/// Abstract operation `IsValidIntegerIndex ( O, index )`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-isvalidintegerindex
pub(crate) fn is_valid_integer_index(obj: &JsObject, index: u64) -> bool {
    let obj = obj.borrow();
    let inner = obj.as_typed_array().expect(
        "integer indexed exotic method should only be callable from integer indexed objects",
    );
    // 1. If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, return false.
    // 2. If ! IsIntegralNumber(index) is false, return false.
    // 3. If index is -0𝔽, return false.
    // 4. If ℝ(index) < 0 or ℝ(index) ≥ O.[[ArrayLength]], return false.
    // 5. Return true.
    !inner.is_detached() && index < inner.array_length()
}

/// Abstract operation `IntegerIndexedElementGet ( O, index )`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integerindexedelementget
fn integer_indexed_element_get(obj: &JsObject, index: u64) -> Option<JsValue> {
    // 1. If ! IsValidIntegerIndex(O, index) is false, return undefined.
    if !is_valid_integer_index(obj, index) {
        return None;
    }

    let obj = obj.borrow();
    let inner = obj
        .as_typed_array()
        .expect("Already checked for detached buffer");
    let buffer_obj = inner
        .viewed_array_buffer()
        .expect("Already checked for detached buffer");
    let buffer_obj_borrow = buffer_obj.borrow();
    let buffer = buffer_obj_borrow
        .as_array_buffer()
        .expect("Already checked for detached buffer");

    // 2. Let offset be O.[[ByteOffset]].
    let offset = inner.byte_offset();

    // 3. Let arrayTypeName be the String value of O.[[TypedArrayName]].
    // 6. Let elementType be the Element Type value in Table 73 for arrayTypeName.
    let elem_type = inner.typed_array_name();

    // 4. Let elementSize be the Element Size value specified in Table 73 for arrayTypeName.
    let size = elem_type.element_size();

    // 5. Let indexedPosition be (ℝ(index) × elementSize) + offset.
    let indexed_position = (index * size) + offset;

    // 7. Return GetValueFromBuffer(O.[[ViewedArrayBuffer]], indexedPosition, elementType, true, Unordered).
    Some(buffer.get_value_from_buffer(
        indexed_position,
        elem_type,
        true,
        SharedMemoryOrder::Unordered,
        None,
    ))
}

/// Abstract operation `IntegerIndexedElementSet ( O, index, value )`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integerindexedelementset
fn integer_indexed_element_set(
    obj: &JsObject,
    index: usize,
    value: &JsValue,
    context: &mut Context,
) -> JsResult<()> {
    let obj_borrow = obj.borrow();
    let inner = obj_borrow.as_typed_array().expect(
        "integer indexed exotic method should only be callable from integer indexed objects",
    );

    let num_value = if inner.typed_array_name().content_type() == ContentType::BigInt {
        // 1. If O.[[ContentType]] is BigInt, let numValue be ? ToBigInt(value).
        value.to_bigint(context)?.into()
    } else {
        // 2. Otherwise, let numValue be ? ToNumber(value).
        value.to_number(context)?.into()
    };

    // 3. If ! IsValidIntegerIndex(O, index) is true, then
    if is_valid_integer_index(obj, index as u64) {
        // a. Let offset be O.[[ByteOffset]].
        let offset = inner.byte_offset();

        // b. Let arrayTypeName be the String value of O.[[TypedArrayName]].
        // e. Let elementType be the Element Type value in Table 73 for arrayTypeName.
        let elem_type = inner.typed_array_name();

        // c. Let elementSize be the Element Size value specified in Table 73 for arrayTypeName.
        let size = elem_type.element_size();

        // d. Let indexedPosition be (ℝ(index) × elementSize) + offset.
        let indexed_position = (index as u64 * size) + offset;

        let buffer_obj = inner
            .viewed_array_buffer()
            .expect("Already checked for detached buffer");
        let mut buffer_obj_borrow = buffer_obj.borrow_mut();
        let buffer = buffer_obj_borrow
            .as_array_buffer_mut()
            .expect("Already checked for detached buffer");

        // f. Perform SetValueInBuffer(O.[[ViewedArrayBuffer]], indexedPosition, elementType, numValue, true, Unordered).
        buffer
            .set_value_in_buffer(
                indexed_position,
                elem_type,
                &num_value,
                SharedMemoryOrder::Unordered,
                None,
                context,
            )
            .expect("SetValueInBuffer cannot fail here");
    }

    // 4. Return NormalCompletion(undefined).
    Ok(())
}
