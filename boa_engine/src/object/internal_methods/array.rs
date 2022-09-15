use crate::{
    error::JsNativeError,
    object::JsObject,
    property::{PropertyDescriptor, PropertyKey},
    string::utf16,
    Context, JsResult,
};

use super::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS};

/// Definitions of the internal object methods for array exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-exotic-objects
pub(crate) static ARRAY_EXOTIC_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __define_own_property__: array_exotic_define_own_property,
    ..ORDINARY_INTERNAL_METHODS
};

/// Define an own property for an array exotic object.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-exotic-objects-defineownproperty-p-desc
pub(crate) fn array_exotic_define_own_property(
    obj: &JsObject,
    key: PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. Assert: IsPropertyKey(P) is true.
    match key {
        // 2. If P is "length", then
        PropertyKey::String(ref s) if s == utf16!("length") => {
            // a. Return ? ArraySetLength(A, Desc).

            array_set_length(obj, desc, context)
        }
        // 3. Else if P is an array index, then
        PropertyKey::Index(index) if index < u32::MAX => {
            // a. Let oldLenDesc be OrdinaryGetOwnProperty(A, "length").
            let old_len_desc = super::ordinary_get_own_property(obj, &"length".into(), context)?
                .expect("the property descriptor must exist");

            // b. Assert: ! IsDataDescriptor(oldLenDesc) is true.
            debug_assert!(old_len_desc.is_data_descriptor());

            // c. Assert: oldLenDesc.[[Configurable]] is false.
            debug_assert!(!old_len_desc.expect_configurable());

            // d. Let oldLen be oldLenDesc.[[Value]].
            // e. Assert: oldLen is a non-negative integral Number.
            // f. Let index be ! ToUint32(P).
            let old_len = old_len_desc
                .expect_value()
                .to_u32(context)
                .expect("this ToUint32 call must not fail");

            // g. If index ≥ oldLen and oldLenDesc.[[Writable]] is false, return false.
            if index >= old_len && !old_len_desc.expect_writable() {
                return Ok(false);
            }

            // h. Let succeeded be ! OrdinaryDefineOwnProperty(A, P, Desc).
            if super::ordinary_define_own_property(obj, key, desc, context)? {
                // j. If index ≥ oldLen, then
                if index >= old_len {
                    // i. Set oldLenDesc.[[Value]] to index + 1𝔽.
                    let old_len_desc = PropertyDescriptor::builder()
                        .value(index + 1)
                        .maybe_writable(old_len_desc.writable())
                        .maybe_enumerable(old_len_desc.enumerable())
                        .maybe_configurable(old_len_desc.configurable());

                    // ii. Set succeeded to OrdinaryDefineOwnProperty(A, "length", oldLenDesc).
                    let succeeded = super::ordinary_define_own_property(
                        obj,
                        "length".into(),
                        old_len_desc.into(),
                        context,
                    )?;

                    // iii. Assert: succeeded is true.
                    debug_assert!(succeeded);
                }

                // k. Return true.
                Ok(true)
            } else {
                // i. If succeeded is false, return false.
                Ok(false)
            }
        }
        // 4. Return OrdinaryDefineOwnProperty(A, P, Desc).
        _ => super::ordinary_define_own_property(obj, key, desc, context),
    }
}

/// Abstract operation `ArraySetLength ( A, Desc )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-arraysetlength
fn array_set_length(
    obj: &JsObject,
    desc: PropertyDescriptor,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. If Desc.[[Value]] is absent, then
    let new_len_val = match desc.value() {
        Some(value) => value,
        _ => {
            // a. Return OrdinaryDefineOwnProperty(A, "length", Desc).
            return super::ordinary_define_own_property(obj, "length".into(), desc, context);
        }
    };

    // 3. Let newLen be ? ToUint32(Desc.[[Value]]).
    let new_len = new_len_val.to_u32(context)?;

    // 4. Let numberLen be ? ToNumber(Desc.[[Value]]).
    let number_len = new_len_val.to_number(context)?;

    // 5. If SameValueZero(newLen, numberLen) is false, throw a RangeError exception.
    #[allow(clippy::float_cmp)]
    if f64::from(new_len) != number_len {
        return Err(JsNativeError::range()
            .with_message("bad length for array")
            .into());
    }

    // 2. Let newLenDesc be a copy of Desc.
    // 6. Set newLenDesc.[[Value]] to newLen.
    let mut new_len_desc = PropertyDescriptor::builder()
        .value(new_len)
        .maybe_writable(desc.writable())
        .maybe_enumerable(desc.enumerable())
        .maybe_configurable(desc.configurable());

    // 7. Let oldLenDesc be OrdinaryGetOwnProperty(A, "length").
    let old_len_desc = super::ordinary_get_own_property(obj, &"length".into(), context)?
        .expect("the property descriptor must exist");

    // 8. Assert: ! IsDataDescriptor(oldLenDesc) is true.
    debug_assert!(old_len_desc.is_data_descriptor());

    // 9. Assert: oldLenDesc.[[Configurable]] is false.
    debug_assert!(!old_len_desc.expect_configurable());

    // 10. Let oldLen be oldLenDesc.[[Value]].
    let old_len = old_len_desc.expect_value();

    // 11. If newLen ≥ oldLen, then
    if new_len >= old_len.to_u32(context)? {
        // a. Return OrdinaryDefineOwnProperty(A, "length", newLenDesc).
        return super::ordinary_define_own_property(
            obj,
            "length".into(),
            new_len_desc.build(),
            context,
        );
    }

    // 12. If oldLenDesc.[[Writable]] is false, return false.
    if !old_len_desc.expect_writable() {
        return Ok(false);
    }

    // 13. If newLenDesc.[[Writable]] is absent or has the value true, let newWritable be true.
    let new_writable = if new_len_desc.inner().writable().unwrap_or(true) {
        true
    }
    // 14. Else,
    else {
        // a. NOTE: Setting the [[Writable]] attribute to false is deferred in case any
        // elements cannot be deleted.
        // c. Set newLenDesc.[[Writable]] to true.
        new_len_desc = new_len_desc.writable(true);

        // b. Let newWritable be false.
        false
    };

    // 15. Let succeeded be ! OrdinaryDefineOwnProperty(A, "length", newLenDesc).
    // 16. If succeeded is false, return false.
    if !super::ordinary_define_own_property(
        obj,
        "length".into(),
        new_len_desc.clone().build(),
        context,
    )
    .expect("this OrdinaryDefineOwnProperty call must not fail")
    {
        return Ok(false);
    }

    // 17. For each own property key P of A that is an array index, whose numeric value is
    // greater than or equal to newLen, in descending numeric index order, do
    let ordered_keys = {
        let mut keys: Vec<_> = obj
            .borrow()
            .properties
            .index_property_keys()
            .filter(|idx| new_len <= *idx && *idx < u32::MAX)
            .collect();
        keys.sort_unstable_by(|x, y| y.cmp(x));
        keys
    };

    for index in ordered_keys {
        // a. Let deleteSucceeded be ! A.[[Delete]](P).
        // b. If deleteSucceeded is false, then
        if !obj.__delete__(&index.into(), context)? {
            // i. Set newLenDesc.[[Value]] to ! ToUint32(P) + 1𝔽.
            new_len_desc = new_len_desc.value(index + 1);

            // ii. If newWritable is false, set newLenDesc.[[Writable]] to false.
            if !new_writable {
                new_len_desc = new_len_desc.writable(false);
            }

            // iii. Perform ! OrdinaryDefineOwnProperty(A, "length", newLenDesc).
            super::ordinary_define_own_property(
                obj,
                "length".into(),
                new_len_desc.build(),
                context,
            )
            .expect("this OrdinaryDefineOwnProperty call must not fail");

            // iv. Return false.
            return Ok(false);
        }
    }

    // 18. If newWritable is false, then
    if !new_writable {
        // a. Set succeeded to ! OrdinaryDefineOwnProperty(A, "length",
        // PropertyDescriptor { [[Writable]]: false }).
        let succeeded = super::ordinary_define_own_property(
            obj,
            "length".into(),
            PropertyDescriptor::builder().writable(false).build(),
            context,
        )
        .expect("this OrdinaryDefineOwnProperty call must not fail");

        // b. Assert: succeeded is true.
        debug_assert!(succeeded);
    }

    // 19. Return true.
    Ok(true)
}
