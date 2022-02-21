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
    // 4. Let isMapped be ! HasOwnProperty(map, P).
    // 5. If isMapped is true, then
    if let PropertyKey::Index(index) = key {
        if let Some(value) = obj
            .borrow()
            .as_mapped_arguments()
            .expect("arguments exotic method must only be callable from arguments objects")
            .get(*index as usize)
        {
            // a. Set desc.[[Value]] to Get(map, P).
            return Ok(Some(
                PropertyDescriptor::builder()
                    .value(value)
                    .maybe_writable(desc.writable())
                    .maybe_enumerable(desc.enumerable())
                    .maybe_configurable(desc.configurable())
                    .build(),
            ));
        }
    }

    // 6. Return desc.
    Ok(Some(desc))
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
    // 2. Let isMapped be HasOwnProperty(map, P).
    let mapped = if let PropertyKey::Index(index) = key {
        // 1. Let map be args.[[ParameterMap]].
        obj.borrow()
            .as_mapped_arguments()
            .expect("arguments exotic method must only be callable from arguments objects")
            .get(index as usize)
            .map(|value| (index as usize, value))
    } else {
        None
    };

    let new_arg_desc = match desc.kind() {
        // 4. If isMapped is true and IsDataDescriptor(Desc) is true, then
        // a. If Desc.[[Value]] is not present and Desc.[[Writable]] is present and its
        // value is false, then
        DescriptorKind::Data {
            writable: Some(false),
            value: None,
        } =>
        // i. Set newArgDesc to a copy of Desc.
        // ii. Set newArgDesc.[[Value]] to Get(map, P).
        {
            if let Some((_, value)) = &mapped {
                PropertyDescriptor::builder()
                    .value(value)
                    .writable(false)
                    .maybe_enumerable(desc.enumerable())
                    .maybe_configurable(desc.configurable())
                    .build()
            } else {
                desc.clone()
            }
        }

        // 3. Let newArgDesc be Desc.
        _ => desc.clone(),
    };

    // 5. Let allowed be ? OrdinaryDefineOwnProperty(args, P, newArgDesc).
    // 6. If allowed is false, return false.
    if !super::ordinary_define_own_property(obj, key, new_arg_desc, context)? {
        return Ok(false);
    }

    // 7. If isMapped is true, then
    if let Some((index, _)) = mapped {
        // 1. Let map be args.[[ParameterMap]].
        let mut obj_mut = obj.borrow_mut();
        let map = obj_mut
            .as_mapped_arguments_mut()
            .expect("arguments exotic method must only be callable from arguments objects");

        // a. If IsAccessorDescriptor(Desc) is true, then
        if desc.is_accessor_descriptor() {
            // i. Call map.[[Delete]](P).
            map.delete(index);
        }
        // b. Else,
        else {
            // i. If Desc.[[Value]] is present, then
            if let Some(value) = desc.value() {
                // 1. Let setStatus be Set(map, P, Desc.[[Value]], false).
                // 2. Assert: setStatus is true because formal parameters mapped by argument objects are always writable.
                map.set(index, value);
            }

            // ii. If Desc.[[Writable]] is present and its value is false, then
            if let Some(false) = desc.writable() {
                // 1. Call map.[[Delete]](P).
                map.delete(index);
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
    if let PropertyKey::Index(index) = key {
        // 1. Let map be args.[[ParameterMap]].
        // 2. Let isMapped be ! HasOwnProperty(map, P).
        if let Some(value) = obj
            .borrow()
            .as_mapped_arguments()
            .expect("arguments exotic method must only be callable from arguments objects")
            .get(*index as usize)
        {
            // a. Assert: map contains a formal parameter mapping for P.
            // b. Return Get(map, P).
            return Ok(value);
        }
    }

    // 3. If isMapped is false, then
    // a. Return ? OrdinaryGet(args, P, Receiver).
    super::ordinary_get(obj, key, receiver, context)
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
    if let PropertyKey::Index(index) = key {
        if JsValue::same_value(&obj.clone().into(), &receiver) {
            // a. Let map be args.[[ParameterMap]].
            // b. Let isMapped be ! HasOwnProperty(map, P).
            // 3. If isMapped is true, then
            // a. Let setStatus be Set(map, P, V, false).
            // b. Assert: setStatus is true because formal parameters mapped by argument objects are always writable.
            obj.borrow_mut()
                .as_mapped_arguments_mut()
                .expect("arguments exotic method must only be callable from arguments objects")
                .set(index as usize, &value);
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
    // 3. Let result be ? OrdinaryDelete(args, P).
    let result = super::ordinary_delete(obj, key, context)?;

    if result {
        if let PropertyKey::Index(index) = key {
            // 1. Let map be args.[[ParameterMap]].
            // 2. Let isMapped be ! HasOwnProperty(map, P).
            // 4. If result is true and isMapped is true, then
            // a. Call map.[[Delete]](P).
            obj.borrow_mut()
                .as_mapped_arguments_mut()
                .expect("arguments exotic method must only be callable from arguments objects")
                .delete(*index as usize);
        }
    }

    // 5. Return result.
    Ok(result)
}
