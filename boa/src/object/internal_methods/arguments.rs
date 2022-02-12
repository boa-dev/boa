use crate::{
    object::JsObject,
    property::{DescriptorKind, PropertyDescriptor, PropertyKey},
    Context, JsResult, JsValue,
};

use super::{InternalObjectMethods, ORDINARY_INTERNAL_METHODS};

pub(crate) static ARGUMENTS_EXOTIC_INTERNAL_METHODS: InternalObjectMethods =
    InternalObjectMethods {
        __get_own_property__: arguments_exotic_get_own_property,
        __define_own_property__: arguments_exotic_define_own_property,
        __get__: arguments_exotic_get,
        __set__: arguments_exotic_set,
        __delete__: arguments_exotic_delete,
        ..ORDINARY_INTERNAL_METHODS
    };

/// `[[GetOwnProperty]]` for arguments exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-arguments-exotic-objects-getownproperty-p
#[inline]
pub(crate) fn arguments_exotic_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context,
) -> JsResult<Option<PropertyDescriptor>> {
    // 1. Let desc be OrdinaryGetOwnProperty(args, P).
    // 2. If desc is undefined, return desc.
    let desc = if let Some(desc) = super::ordinary_get_own_property(obj, key, context)? {
        desc
    } else {
        return Ok(None);
    };

    // 3. Let map be args.[[ParameterMap]].
    let map = obj
        .borrow()
        .as_mapped_arguments()
        .expect("arguments exotic method must only be callable from arguments objects")
        .parameter_map();

    Ok(Some(
        // 4. Let isMapped be ! HasOwnProperty(map, P).
        // 5. If isMapped is true, then
        if map
            .has_own_property(key.clone(), context)
            .expect("HasOwnProperty must not fail per the spec")
        {
            // a. Set desc.[[Value]] to Get(map, P).
            PropertyDescriptor::builder()
                .value(map.get(key.clone(), context)?)
                .maybe_writable(desc.writable())
                .maybe_enumerable(desc.enumerable())
                .maybe_configurable(desc.configurable())
                .build()
        } else {
            // 6. Return desc.
            desc
        },
    ))
}

/// `[[DefineOwnProperty]]` for arguments exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-arguments-exotic-objects-defineownproperty-p-desc
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn arguments_exotic_define_own_property(
    obj: &JsObject,
    key: PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. Let map be args.[[ParameterMap]].
    let map = obj
        .borrow()
        .as_mapped_arguments()
        .expect("arguments exotic method must only be callable from arguments objects")
        .parameter_map();

    // 2. Let isMapped be HasOwnProperty(map, P).
    let is_mapped = map.has_own_property(key.clone(), context)?;

    let new_arg_desc = match desc.kind() {
        // 4. If isMapped is true and IsDataDescriptor(Desc) is true, then
        // a. If Desc.[[Value]] is not present and Desc.[[Writable]] is present and its
        // value is false, then
        DescriptorKind::Data {
            writable: Some(false),
            value: None,
        } if is_mapped =>
        // i. Set newArgDesc to a copy of Desc.
        // ii. Set newArgDesc.[[Value]] to Get(map, P).
        {
            PropertyDescriptor::builder()
                .value(map.get(key.clone(), context)?)
                .writable(false)
                .maybe_enumerable(desc.enumerable())
                .maybe_configurable(desc.configurable())
                .build()
        }

        // 3. Let newArgDesc be Desc.
        _ => desc.clone(),
    };

    // 5. Let allowed be ? OrdinaryDefineOwnProperty(args, P, newArgDesc).
    // 6. If allowed is false, return false.
    if !super::ordinary_define_own_property(obj, key.clone(), new_arg_desc, context)? {
        return Ok(false);
    }

    // 7. If isMapped is true, then
    if is_mapped {
        // a. If IsAccessorDescriptor(Desc) is true, then
        if desc.is_accessor_descriptor() {
            // i. Call map.[[Delete]](P).
            map.__delete__(&key, context)?;
        }
        // b. Else,
        else {
            // i. If Desc.[[Value]] is present, then
            if let Some(value) = desc.value() {
                // 1. Let setStatus be Set(map, P, Desc.[[Value]], false).
                let set_status = map.set(key.clone(), value, false, context);

                // 2. Assert: setStatus is true because formal parameters mapped by argument objects are always writable.
                assert_eq!(set_status, Ok(true));
            }

            // ii. If Desc.[[Writable]] is present and its value is false, then
            if let Some(false) = desc.writable() {
                // 1. Call map.[[Delete]](P).
                map.__delete__(&key, context)?;
            }
        }
    }

    // 8. Return true.
    Ok(true)
}

/// `[[Get]]` for arguments exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-arguments-exotic-objects-get-p-receiver
pub(crate) fn arguments_exotic_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    // 1. Let map be args.[[ParameterMap]].
    let map = obj
        .borrow()
        .as_mapped_arguments()
        .expect("arguments exotic method must only be callable from arguments objects")
        .parameter_map();

    // 2. Let isMapped be ! HasOwnProperty(map, P).
    // 4. Else,
    if map
        .has_own_property(key.clone(), context)
        .expect("HasOwnProperty must not fail per the spec")
    {
        // a. Assert: map contains a formal parameter mapping for P.
        // b. Return Get(map, P).
        map.get(key.clone(), context)

    // 3. If isMapped is false, then
    } else {
        // a. Return ? OrdinaryGet(args, P, Receiver).
        super::ordinary_get(obj, key, receiver, context)
    }
}

/// `[[Set]]` for arguments exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-arguments-exotic-objects-set-p-v-receiver
pub(crate) fn arguments_exotic_set(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. If SameValue(args, Receiver) is false, then
    // a. Let isMapped be false.
    // 2. Else,
    if JsValue::same_value(&obj.clone().into(), &receiver) {
        // a. Let map be args.[[ParameterMap]].
        let map = obj
            .borrow()
            .as_mapped_arguments()
            .expect("arguments exotic method must only be callable from arguments objects")
            .parameter_map();

        // b. Let isMapped be ! HasOwnProperty(map, P).
        // 3. If isMapped is true, then
        if map
            .has_own_property(key.clone(), context)
            .expect("HasOwnProperty must not fail per the spec")
        {
            // a. Let setStatus be Set(map, P, V, false).
            let set_status = map.set(key.clone(), value.clone(), false, context);

            // b. Assert: setStatus is true because formal parameters mapped by argument objects are always writable.
            assert_eq!(set_status, Ok(true));
        }
    }

    // 4. Return ? OrdinarySet(args, P, V, Receiver).
    super::ordinary_set(obj, key, value, receiver, context)
}

/// `[[Delete]]` for arguments exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-arguments-exotic-objects-delete-p
pub(crate) fn arguments_exotic_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. Let map be args.[[ParameterMap]].
    let map = obj
        .borrow()
        .as_mapped_arguments()
        .expect("arguments exotic method must only be callable from arguments objects")
        .parameter_map();

    // 2. Let isMapped be ! HasOwnProperty(map, P).
    let is_mapped = map
        .has_own_property(key.clone(), context)
        .expect("HasOwnProperty must not fail per the spec");

    // 3. Let result be ? OrdinaryDelete(args, P).
    let result = super::ordinary_delete(obj, key, context)?;

    // 4. If result is true and isMapped is true, then
    if is_mapped && result {
        // a. Call map.[[Delete]](P).
        map.__delete__(key, context)?;
    }

    // 5. Return result.
    Ok(result)
}
