use crate::{
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
    Context, JsResult, JsValue,
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
    let desc = super::ordinary_get_own_property(obj, key, context)?;

    if desc.is_some() {
        Ok(desc)
    } else {
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
    let string_desc = string_get_own_property(obj, &key);

    if let Some(string_desc) = string_desc {
        let extensible = obj.borrow().extensible;
        Ok(super::is_compatible_property_descriptor(
            extensible,
            desc,
            string_desc,
        ))
    } else {
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
pub(crate) fn string_exotic_own_property_keys(
    obj: &JsObject,
    _context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    let mut keys = Vec::new();

    let obj = obj.borrow();

    let string = obj
        .as_string()
        .expect("string exotic method should only be callable from string objects");
    let len = string.encode_utf16().count();

    for i in 0..len {
        keys.push(i.into());
    }

    // todo: sort keys or ensure in some way that indexed properties are sorted... or maybe it's not necessary?
    for elem in obj
        .properties
        .keys()
        .filter(|prop| !matches!(prop, PropertyKey::Index(i) if (*i as usize) < len))
    {
        keys.push(elem)
    }

    Ok(keys)
}

/// StringGetOwnProperty abstract operation
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-stringgetownproperty
#[allow(clippy::float_cmp)]
#[inline]
fn string_get_own_property(obj: &JsObject, key: &PropertyKey) -> Option<PropertyDescriptor> {
    let obj = obj.borrow();

    let pos = match key {
        PropertyKey::Index(index) => *index as usize,
        PropertyKey::String(index) => {
            let index = index.canonical_numeric_index_string()?;
            if index != ((index as usize) as f64) {
                return None;
            }
            index as usize
        }
        _ => return None,
    };
    let string = obj
        .as_string()
        .expect("string exotic method should only be callable from string objects");

    let result_str = string
        .encode_utf16()
        .nth(pos)
        .map(|c| JsValue::from(String::from_utf16_lossy(&[c])))?;

    let desc = PropertyDescriptor::builder()
        .value(result_str)
        .writable(false)
        .enumerable(true)
        .configurable(false)
        .build();

    Some(desc)
}
