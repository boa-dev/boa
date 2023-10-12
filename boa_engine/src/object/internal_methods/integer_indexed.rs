use std::sync::atomic;

use boa_macros::utf16;

use crate::{
    builtins::Number,
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
    Context, JsResult, JsString, JsValue,
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

/// `CanonicalNumericIndexString ( argument )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-canonicalnumericindexstring
fn canonical_numeric_index_string(argument: &JsString) -> Option<f64> {
    // 1. If argument is "-0", return -0𝔽.
    if argument == utf16!("-0") {
        return Some(-0.0);
    }

    // 2. Let n be ! ToNumber(argument).
    let n = argument.to_number();

    // 3. If ! ToString(n) is argument, return n.
    if &Number::to_js_string(n) == argument {
        return Some(n);
    }

    // 4. Return undefined.
    None
}

/// `[[GetOwnProperty]]` internal method for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-getownproperty-p
pub(crate) fn integer_indexed_exotic_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<Option<PropertyDescriptor>> {
    let p = match key {
        PropertyKey::String(key) => {
            // 1.a. Let numericIndex be CanonicalNumericIndexString(P).
            canonical_numeric_index_string(key)
        }
        PropertyKey::Index(index) => Some(index.get().into()),
        PropertyKey::Symbol(_) => None,
    };

    // 1. If P is a String, then
    // 1.b. If numericIndex is not undefined, then
    if let Some(numeric_index) = p {
        // i. Let value be IntegerIndexedElementGet(O, numericIndex).
        let value = integer_indexed_element_get(obj, numeric_index);

        // ii. If value is undefined, return undefined.
        // iii. Return the PropertyDescriptor { [[Value]]: value, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true }.
        return Ok(value.map(|v| {
            PropertyDescriptor::builder()
                .value(v)
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .build()
        }));
    }

    // 2. Return OrdinaryGetOwnProperty(O, P).
    super::ordinary_get_own_property(obj, key, context)
}

/// `[[HasProperty]]` internal method for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-hasproperty-p
pub(crate) fn integer_indexed_exotic_has_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let p = match key {
        PropertyKey::String(key) => {
            // 1.a. Let numericIndex be CanonicalNumericIndexString(P).
            canonical_numeric_index_string(key)
        }
        PropertyKey::Index(index) => Some(index.get().into()),
        PropertyKey::Symbol(_) => None,
    };

    // 1. If P is a String, then
    // 1.b. If numericIndex is not undefined, return IsValidIntegerIndex(O, numericIndex).
    if let Some(numeric_index) = p {
        return Ok(is_valid_integer_index(obj, numeric_index));
    }

    // 2. Return ? OrdinaryHasProperty(O, P).
    super::ordinary_has_property(obj, key, context)
}

/// `[[DefineOwnProperty]]` internal method for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-defineownproperty-p-desc
pub(crate) fn integer_indexed_exotic_define_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let p = match key {
        PropertyKey::String(key) => {
            // 1.a. Let numericIndex be CanonicalNumericIndexString(P).
            canonical_numeric_index_string(key)
        }
        PropertyKey::Index(index) => Some(index.get().into()),
        PropertyKey::Symbol(_) => None,
    };

    // 1. If P is a String, then
    // 1.b. If numericIndex is not undefined, then
    if let Some(numeric_index) = p {
        // i. If IsValidIntegerIndex(O, numericIndex) is false, return false.
        if !is_valid_integer_index(obj, numeric_index) {
            return Ok(false);
        }

        // ii. If Desc has a [[Configurable]] field and Desc.[[Configurable]] is false, return false.
        if desc.configurable() == Some(false) {
            return Ok(false);
        }

        // iii. If Desc has an [[Enumerable]] field and Desc.[[Enumerable]] is false, return false.
        if desc.enumerable() == Some(false) {
            return Ok(false);
        }

        // iv. If IsAccessorDescriptor(Desc) is true, return false.
        if desc.is_accessor_descriptor() {
            return Ok(false);
        }

        // v. If Desc has a [[Writable]] field and Desc.[[Writable]] is false, return false.
        if desc.writable() == Some(false) {
            return Ok(false);
        }

        // vi. If Desc has a [[Value]] field, perform ? IntegerIndexedElementSet(O, numericIndex, Desc.[[Value]]).
        if let Some(value) = desc.value() {
            integer_indexed_element_set(obj, numeric_index, value, context)?;
        }

        // vii. Return true.
        return Ok(true);
    }

    // 2. Return ! OrdinaryDefineOwnProperty(O, P, Desc).
    super::ordinary_define_own_property(obj, key, desc, context)
}

/// Internal method `[[Get]]` for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-get-p-receiver
pub(crate) fn integer_indexed_exotic_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    let p = match key {
        PropertyKey::String(key) => {
            // 1.a. Let numericIndex be CanonicalNumericIndexString(P).
            canonical_numeric_index_string(key)
        }
        PropertyKey::Index(index) => Some(index.get().into()),
        PropertyKey::Symbol(_) => None,
    };

    // 1. If P is a String, then
    // 1.b. If numericIndex is not undefined, then
    if let Some(numeric_index) = p {
        // i. Return IntegerIndexedElementGet(O, numericIndex).
        return Ok(integer_indexed_element_get(obj, numeric_index).unwrap_or_default());
    }

    // 2. Return ? OrdinaryGet(O, P, Receiver).
    super::ordinary_get(obj, key, receiver, context)
}

/// Internal method `[[Set]]` for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-set-p-v-receiver
pub(crate) fn integer_indexed_exotic_set(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let p = match &key {
        PropertyKey::String(key) => {
            // 1.a. Let numericIndex be CanonicalNumericIndexString(P).
            canonical_numeric_index_string(key)
        }
        PropertyKey::Index(index) => Some(index.get().into()),
        PropertyKey::Symbol(_) => None,
    };

    // 1. If P is a String, then
    // 1.b. If numericIndex is not undefined, then
    if let Some(numeric_index) = p {
        // i. If SameValue(O, Receiver) is true, then
        if JsValue::same_value(&obj.clone().into(), &receiver) {
            // 1. Perform ? IntegerIndexedElementSet(O, numericIndex, V).
            integer_indexed_element_set(obj, numeric_index, &value, context)?;

            // 2. Return true.
            return Ok(true);
        }

        // ii. If IsValidIntegerIndex(O, numericIndex) is false, return true.
        if !is_valid_integer_index(obj, numeric_index) {
            return Ok(true);
        }
    }

    // 2. Return ? OrdinarySet(O, P, V, Receiver).
    super::ordinary_set(obj, key, value, receiver, context)
}

/// Internal method `[[Delete]]` for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-delete-p
pub(crate) fn integer_indexed_exotic_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let p = match &key {
        PropertyKey::String(key) => {
            // 1.a. Let numericIndex be CanonicalNumericIndexString(P).
            canonical_numeric_index_string(key)
        }
        PropertyKey::Index(index) => Some(index.get().into()),
        PropertyKey::Symbol(_) => None,
    };

    // 1. If P is a String, then
    // 1.b. If numericIndex is not undefined, then
    if let Some(numeric_index) = p {
        // i. If IsValidIntegerIndex(O, numericIndex) is false, return true; else return false.
        return Ok(!is_valid_integer_index(obj, numeric_index));
    }

    // 2. Return ! OrdinaryDelete(O, P).
    super::ordinary_delete(obj, key, context)
}

/// Internal method `[[OwnPropertyKeys]]` for Integer-Indexed exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integer-indexed-exotic-objects-ownpropertykeys
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn integer_indexed_exotic_own_property_keys(
    obj: &JsObject,
    _context: &mut Context<'_>,
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
        //     a. For each integer i starting with 0 such that i < O.[[ArrayLength]], in ascending order, do
        //         i. Add ! ToString(𝔽(i)) as the last element of keys.
        (0..inner.array_length()).map(PropertyKey::from).collect()
    };

    // 3. For each own property key P of O such that Type(P) is String and P is not an array index, in ascending chronological order of property creation, do
    //     a. Add P as the last element of keys.
    //
    // 4. For each own property key P of O such that Type(P) is Symbol, in ascending chronological order of property creation, do
    //     a. Add P as the last element of keys.
    keys.extend(obj.properties.shape.keys());

    // 5. Return keys.
    Ok(keys)
}

/// Abstract operation `IsValidIntegerIndex ( O, index )`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-isvalidintegerindex
pub(crate) fn is_valid_integer_index(obj: &JsObject, index: f64) -> bool {
    let obj = obj.borrow();
    let inner = obj.as_typed_array().expect(
        "integer indexed exotic method should only be callable from integer indexed objects",
    );

    // 1. If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, return false.
    if inner.is_detached() {
        return false;
    }

    // 2. If IsIntegralNumber(index) is false, return false.
    if index.is_nan() || index.is_infinite() || index.fract() != 0.0 {
        return false;
    }

    // 3. If index is -0𝔽, return false.
    if index == 0.0 && index.is_sign_negative() {
        return false;
    }

    // 4. If ℝ(index) < 0 or ℝ(index) ≥ O.[[ArrayLength]], return false.
    if index < 0.0 || index >= inner.array_length() as f64 {
        return false;
    }

    // 5. Return true.
    true
}

/// Abstract operation `IntegerIndexedElementGet ( O, index )`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integerindexedelementget
fn integer_indexed_element_get(obj: &JsObject, index: f64) -> Option<JsValue> {
    // 1. If ! IsValidIntegerIndex(O, index) is false, return undefined.
    if !is_valid_integer_index(obj, index) {
        return None;
    }

    let obj = obj.borrow();
    let inner = obj.as_typed_array().expect("Must be a typed array");
    let buffer = inner.viewed_array_buffer();
    let buffer = buffer.borrow();
    let buffer = buffer.as_buffer().expect("Must be a buffer");
    let buffer = buffer
        .data()
        .expect("already checked that it's not detached");

    // 2. Let offset be O.[[ByteOffset]].
    let offset = inner.byte_offset();

    // 3. Let arrayTypeName be the String value of O.[[TypedArrayName]].
    // 6. Let elementType be the Element Type value in Table 73 for arrayTypeName.
    let elem_type = inner.kind();

    // 4. Let elementSize be the Element Size value specified in Table 73 for arrayTypeName.
    let size = elem_type.element_size();

    // 5. Let indexedPosition be (ℝ(index) × elementSize) + offset.
    let indexed_position = ((index as u64 * size) + offset) as usize;

    // 7. Return GetValueFromBuffer(O.[[ViewedArrayBuffer]], indexedPosition, elementType, true, Unordered).

    // SAFETY: The integer indexed object guarantees that the buffer is aligned.
    // The call to `is_valid_integer_index` guarantees that the index is in-bounds.
    let value = unsafe {
        buffer
            .subslice(indexed_position..)
            .get_value(elem_type, atomic::Ordering::Relaxed)
    };

    Some(value.into())
}

/// Abstract operation `IntegerIndexedElementSet ( O, index, value )`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-integerindexedelementset
pub(crate) fn integer_indexed_element_set(
    obj: &JsObject,
    index: f64,
    value: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<()> {
    let obj_borrow = obj.borrow();
    let inner = obj_borrow.as_typed_array().expect(
        "integer indexed exotic method should only be callable from integer indexed objects",
    );

    // 1. If O.[[ContentType]] is BigInt, let numValue be ? ToBigInt(value).
    // 2. Otherwise, let numValue be ? ToNumber(value).
    let value = inner.kind().get_element(value, context)?;

    if !is_valid_integer_index(obj, index) {
        return Ok(());
    }

    // 3. If ! IsValidIntegerIndex(O, index) is true, then
    // a. Let offset be O.[[ByteOffset]].
    let offset = inner.byte_offset();

    // b. Let arrayTypeName be the String value of O.[[TypedArrayName]].
    // e. Let elementType be the Element Type value in Table 73 for arrayTypeName.
    let elem_type = inner.kind();

    // c. Let elementSize be the Element Size value specified in Table 73 for arrayTypeName.
    let size = elem_type.element_size();

    // d. Let indexedPosition be (ℝ(index) × elementSize) + offset.
    let indexed_position = ((index as u64 * size) + offset) as usize;

    let buffer = inner.viewed_array_buffer();
    let mut buffer = buffer.borrow_mut();
    let mut buffer = buffer.as_buffer_mut().expect("Must be a buffer");
    let mut buffer = buffer
        .data_mut()
        .expect("already checked that it's not detached");

    // f. Perform SetValueInBuffer(O.[[ViewedArrayBuffer]], indexedPosition, elementType, numValue, true, Unordered).

    // SAFETY: The integer indexed object guarantees that the buffer is aligned.
    // The call to `is_valid_integer_index` guarantees that the index is in-bounds.
    unsafe {
        buffer
            .subslice_mut(indexed_position..)
            .set_value(value, atomic::Ordering::Relaxed);
    }

    // 4. Return NormalCompletion(undefined).
    Ok(())
}
