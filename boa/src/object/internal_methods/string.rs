use crate::{
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
    Context, JsResult, JsValue,
};

// todo: missing `[[DefineOwnProperty]]` and `[[OwnPropertyKeys]]`

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

/// StringGetOwnProperty abstract operation
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-stringgetownproperty
#[inline]
pub(crate) fn string_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
) -> Option<PropertyDescriptor> {
    let obj = obj.borrow();

    match key {
        PropertyKey::Index(index) => {
            let string = obj.as_string().unwrap();
            let pos = *index as usize;

            if pos >= string.len() {
                return None;
            }

            // todo: shouldn't it be encoded before checking length and position?
            let result_str = string
                .encode_utf16()
                .nth(pos)
                .map(|utf16_val| JsValue::from(String::from_utf16_lossy(&[utf16_val])))?;

            // todo: should expect that pos < string.len(). Skipped because of the above todo.
            // .expect("already verified that pos >= string.len()");

            let desc = PropertyDescriptor::builder()
                .value(result_str)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build();

            Some(desc)
        }
        _ => None,
    }
}
