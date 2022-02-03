use crate::{
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
    Context, JsResult, JsValue,
};

use super::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS};

/// Definitions of the internal object methods for string exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-string-exotic-objects
pub(crate) static STRING_EXOTIC_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __get_own_property__: string_exotic_get_own_property,
    __define_own_property__: string_exotic_define_own_property,
    __own_property_keys__: string_exotic_own_property_keys,
    ..ORDINARY_INTERNAL_METHODS
};

/// Gets own property of 'String' exotic object
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-string-exotic-objects-getownproperty-p
#[inline]
pub(crate) fn string_exotic_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context,
) -> JsResult<Option<PropertyDescriptor>> {
    // 1. Assert: IsPropertyKey(P) is true.
    // 2. Let desc be OrdinaryGetOwnProperty(S, P).
    let desc = super::ordinary_get_own_property(obj, key, context)?;

    // 3. If desc is not undefined, return desc.
    if desc.is_some() {
        Ok(desc)
    } else {
        // 4. Return ! StringGetOwnProperty(S, P).
        Ok(string_get_own_property(obj, key))
    }
}

/// Defines own property of 'String' exotic object
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-string-exotic-objects-defineownproperty-p-desc
#[inline]
pub(crate) fn string_exotic_define_own_property(
    obj: &JsObject,
    key: PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. Assert: IsPropertyKey(P) is true.
    // 2. Let stringDesc be ! StringGetOwnProperty(S, P).
    let string_desc = string_get_own_property(obj, &key);

    // 3. If stringDesc is not undefined, then
    if let Some(string_desc) = string_desc {
        // a. Let extensible be S.[[Extensible]].
        let extensible = obj.borrow().extensible;
        // b. Return ! IsCompatiblePropertyDescriptor(extensible, Desc, stringDesc).
        Ok(super::is_compatible_property_descriptor(
            extensible,
            desc,
            Some(string_desc),
        ))
    } else {
        // 4. Return ! OrdinaryDefineOwnProperty(S, P, Desc).
        super::ordinary_define_own_property(obj, key, desc, context)
    }
}

/// Gets own property keys of 'String' exotic object
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-string-exotic-objects-ownpropertykeys
#[inline]
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn string_exotic_own_property_keys(
    obj: &JsObject,
    _context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    let obj = obj.borrow();

    // 2. Let str be O.[[StringData]].
    // 3. Assert: Type(str) is String.
    let string = obj
        .as_string()
        .expect("string exotic method should only be callable from string objects");
    // 4. Let len be the length of str.
    let len = string.encode_utf16().count();

    // 1. Let keys be a new empty List.
    let mut keys = Vec::with_capacity(len);

    // 5. For each integer i starting with 0 such that i < len, in ascending order, do
    // a. Add ! ToString(𝔽(i)) as the last element of keys.
    keys.extend((0..len).into_iter().map(Into::into));

    // 6. For each own property key P of O such that P is an array index
    // and ! ToIntegerOrInfinity(P) ≥ len, in ascending numeric index order, do
    // a. Add P as the last element of keys.
    let mut remaining_indices: Vec<_> = obj
        .properties
        .index_property_keys()
        .copied()
        .filter(|idx| (*idx as usize) >= len)
        .collect();
    remaining_indices.sort_unstable();
    keys.extend(remaining_indices.into_iter().map(Into::into));

    // 7. For each own property key P of O such that Type(P) is String and P is not
    // an array index, in ascending chronological order of property creation, do
    // a. Add P as the last element of keys.
    keys.extend(
        obj.properties
            .string_property_keys()
            .cloned()
            .map(Into::into),
    );

    // 8. For each own property key P of O such that Type(P) is Symbol, in ascending
    // chronological order of property creation, do
    // a. Add P as the last element of keys.
    keys.extend(
        obj.properties
            .symbol_property_keys()
            .cloned()
            .map(Into::into),
    );

    // 9. Return keys.
    Ok(keys)
}

/// `StringGetOwnProperty` abstract operation
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-stringgetownproperty
#[allow(clippy::float_cmp)]
#[inline]
fn string_get_own_property(obj: &JsObject, key: &PropertyKey) -> Option<PropertyDescriptor> {
    // 1. Assert: S is an Object that has a [[StringData]] internal slot.
    // 2. Assert: IsPropertyKey(P) is true.
    // 3. If Type(P) is not String, return undefined.
    // 4. Let index be ! CanonicalNumericIndexString(P).
    // 5. If index is undefined, return undefined.
    // 6. If IsIntegralNumber(index) is false, return undefined.
    // 7. If index is -0𝔽, return undefined.
    let pos = match key {
        PropertyKey::Index(index) => *index as usize,
        _ => return None,
    };

    // 8. Let str be S.[[StringData]].
    // 9. Assert: Type(str) is String.
    let string = obj
        .borrow()
        .as_string()
        .expect("string exotic method should only be callable from string objects");

    // 10. Let len be the length of str.
    // 11. If ℝ(index) < 0 or len ≤ ℝ(index), return undefined.
    // 12. Let resultStr be the String value of length 1, containing one code unit from str, specifically the code unit at index ℝ(index).
    let result_str = string
        .encode_utf16()
        .nth(pos)
        .map(|c| JsValue::from(String::from_utf16_lossy(&[c])))?;

    // 13. Return the PropertyDescriptor { [[Value]]: resultStr, [[Writable]]: false, [[Enumerable]]: true, [[Configurable]]: false }.
    let desc = PropertyDescriptor::builder()
        .value(result_str)
        .writable(false)
        .enumerable(true)
        .configurable(false)
        .build();

    Some(desc)
}
