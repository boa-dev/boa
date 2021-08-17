use crate::{
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
    Context, JsResult,
};

/// Define an own property for an array.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-exotic-objects-defineownproperty-p-desc
pub(crate) fn array_define_own_property(
    obj: &JsObject,
    key: PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context,
) -> JsResult<bool> {
    match key {
        PropertyKey::String(ref s) if s == "length" => {
            let new_len_val = match desc.value() {
                Some(value) => value,
                _ => {
                    return super::ordinary_define_own_property(obj, "length".into(), desc, context)
                }
            };

            let new_len = new_len_val.to_u32(context)?;
            let number_len = new_len_val.to_number(context)?;

            #[allow(clippy::float_cmp)]
            if new_len as f64 != number_len {
                return Err(context.construct_range_error("bad length for array"));
            }

            let mut new_len_desc = PropertyDescriptor::builder()
                .value(new_len)
                .maybe_writable(desc.writable())
                .maybe_enumerable(desc.enumerable())
                .maybe_configurable(desc.configurable());
            let old_len_desc = obj
                .__get_own_property__(&"length".into(), context)?
                .unwrap();
            let old_len = old_len_desc.expect_value();
            if new_len >= old_len.to_u32(context)? {
                return super::ordinary_define_own_property(
                    obj,
                    "length".into(),
                    new_len_desc.build(),
                    context,
                );
            }

            if !old_len_desc.expect_writable() {
                return Ok(false);
            }

            let new_writable = if new_len_desc.inner().writable().unwrap_or(true) {
                true
            } else {
                new_len_desc = new_len_desc.writable(true);
                false
            };

            if !super::ordinary_define_own_property(
                obj,
                "length".into(),
                new_len_desc.clone().build(),
                context,
            )? {
                return Ok(false);
            }

            let max_value = obj.borrow().properties.index_property_keys().max().copied();

            if let Some(mut index) = max_value {
                while index >= new_len {
                    let contains_index = obj.borrow().properties.contains_key(&index.into());
                    if contains_index && !obj.__delete__(&index.into(), context)? {
                        new_len_desc = new_len_desc.value(index + 1);
                        if !new_writable {
                            new_len_desc = new_len_desc.writable(false);
                        }
                        super::ordinary_define_own_property(
                            obj,
                            "length".into(),
                            new_len_desc.build(),
                            context,
                        )?;
                        return Ok(false);
                    }

                    index = if let Some(sub) = index.checked_sub(1) {
                        sub
                    } else {
                        break;
                    }
                }
            }

            if !new_writable {
                super::ordinary_define_own_property(
                    obj,
                    "length".into(),
                    PropertyDescriptor::builder().writable(false).build(),
                    context,
                )?;
            }
            Ok(true)
        }
        PropertyKey::Index(index) => {
            let old_len_desc = obj
                .__get_own_property__(&"length".into(), context)?
                .unwrap();
            let old_len = old_len_desc.expect_value().to_u32(context)?;
            if index >= old_len && !old_len_desc.expect_writable() {
                return Ok(false);
            }
            if super::ordinary_define_own_property(obj, key, desc, context)? {
                if index >= old_len && index < u32::MAX {
                    let desc = PropertyDescriptor::builder()
                        .value(index + 1)
                        .maybe_writable(old_len_desc.writable())
                        .maybe_enumerable(old_len_desc.enumerable())
                        .maybe_configurable(old_len_desc.configurable());
                    super::ordinary_define_own_property(
                        obj,
                        "length".into(),
                        desc.into(),
                        context,
                    )?;
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }
        _ => super::ordinary_define_own_property(obj, key, desc, context),
    }
}
