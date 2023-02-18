use crate::{
    builtins::{array, object::Object},
    error::JsNativeError,
    object::{InternalObjectMethods, JsObject, JsPrototype},
    property::{PropertyDescriptor, PropertyKey},
    string::utf16,
    value::Type,
    Context, JsResult, JsValue,
};
use rustc_hash::FxHashSet;

/// Definitions of the internal object methods for array exotic objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-array-exotic-objects
pub(crate) static PROXY_EXOTIC_INTERNAL_METHODS_BASIC: InternalObjectMethods =
    InternalObjectMethods {
        __get_prototype_of__: proxy_exotic_get_prototype_of,
        __set_prototype_of__: proxy_exotic_set_prototype_of,
        __is_extensible__: proxy_exotic_is_extensible,
        __prevent_extensions__: proxy_exotic_prevent_extensions,
        __get_own_property__: proxy_exotic_get_own_property,
        __define_own_property__: proxy_exotic_define_own_property,
        __has_property__: proxy_exotic_has_property,
        __get__: proxy_exotic_get,
        __set__: proxy_exotic_set,
        __delete__: proxy_exotic_delete,
        __own_property_keys__: proxy_exotic_own_property_keys,
        __call__: None,
        __construct__: None,
    };

pub(crate) static PROXY_EXOTIC_INTERNAL_METHODS_WITH_CALL: InternalObjectMethods =
    InternalObjectMethods {
        __call__: Some(proxy_exotic_call),
        ..PROXY_EXOTIC_INTERNAL_METHODS_BASIC
    };

pub(crate) static PROXY_EXOTIC_INTERNAL_METHODS_ALL: InternalObjectMethods =
    InternalObjectMethods {
        __call__: Some(proxy_exotic_call),
        __construct__: Some(proxy_exotic_construct),
        ..PROXY_EXOTIC_INTERNAL_METHODS_BASIC
    };

/// `10.5.1 [[GetPrototypeOf]] ( )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-getprototypeof
pub(crate) fn proxy_exotic_get_prototype_of(
    obj: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<JsPrototype> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "getPrototypeOf").
    let Some(trap) = handler.get_method(utf16!("getPrototypeOf"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? target.[[GetPrototypeOf]]().
        return target.__get_prototype_of__(context);
    };

    // 7. Let handlerProto be ? Call(trap, handler, « target »).
    let handler_proto = trap.call(&handler.into(), &[target.clone().into()], context)?;

    // 8. If Type(handlerProto) is neither Object nor Null, throw a TypeError exception.
    let handler_proto = match &handler_proto {
        JsValue::Object(obj) => Some(obj.clone()),
        JsValue::Null => None,
        _ => {
            return Err(JsNativeError::typ()
                .with_message("Proxy trap result is neither object nor null")
                .into())
        }
    };

    // 9. Let extensibleTarget be ? IsExtensible(target).
    // 10. If extensibleTarget is true, return handlerProto.
    if target.is_extensible(context)? {
        return Ok(handler_proto);
    }

    // 11. Let targetProto be ? target.[[GetPrototypeOf]]().
    let target_proto = target.__get_prototype_of__(context)?;

    // 12. If SameValue(handlerProto, targetProto) is false, throw a TypeError exception.
    if handler_proto != target_proto {
        return Err(JsNativeError::typ()
            .with_message("Proxy trap returned unexpected prototype")
            .into());
    }

    // 13. Return handlerProto.
    Ok(handler_proto)
}

/// `10.5.2 [[SetPrototypeOf]] ( V )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-setprototypeof-v
pub(crate) fn proxy_exotic_set_prototype_of(
    obj: &JsObject,
    val: JsPrototype,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "setPrototypeOf").
    let Some(trap) = handler.get_method(utf16!("setPrototypeOf"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? target.[[SetPrototypeOf]](V).
        return target.__set_prototype_of__(val, context);
    };

    // 7. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, V »)).
    // 8. If booleanTrapResult is false, return false.
    if !trap
        .call(
            &handler.into(),
            &[
                target.clone().into(),
                val.clone().map_or(JsValue::Null, Into::into),
            ],
            context,
        )?
        .to_boolean()
    {
        return Ok(false);
    }

    // 9. Let extensibleTarget be ? IsExtensible(target).
    // 10. If extensibleTarget is true, return true.
    if target.is_extensible(context)? {
        return Ok(true);
    }

    // 11. Let targetProto be ? target.[[GetPrototypeOf]]().
    let target_proto = target.__get_prototype_of__(context)?;

    // 12. If SameValue(V, targetProto) is false, throw a TypeError exception.
    if val != target_proto {
        return Err(JsNativeError::typ()
            .with_message("Proxy trap failed to set prototype")
            .into());
    }

    // 13. Return true.
    Ok(true)
}

/// `10.5.3 [[IsExtensible]] ( )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-isextensible
pub(crate) fn proxy_exotic_is_extensible(
    obj: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "isExtensible").
    let Some(trap) = handler.get_method(utf16!("isExtensible"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? IsExtensible(target).
        return target.is_extensible(context);
    };

    // 7. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target »)).
    let boolean_trap_result = trap
        .call(&handler.into(), &[target.clone().into()], context)?
        .to_boolean();

    // 8. Let targetResult be ? IsExtensible(target).
    let target_result = target.is_extensible(context)?;

    // 9. If SameValue(booleanTrapResult, targetResult) is false, throw a TypeError exception.
    if boolean_trap_result != target_result {
        return Err(JsNativeError::typ()
            .with_message("Proxy trap returned unexpected extensible value")
            .into());
    }

    // 10. Return booleanTrapResult.
    Ok(boolean_trap_result)
}

/// `10.5.4 [[PreventExtensions]] ( )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-preventextensions
pub(crate) fn proxy_exotic_prevent_extensions(
    obj: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "preventExtensions").
    let Some(trap) = handler.get_method(utf16!("preventExtensions"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? target.[[PreventExtensions]]().
        return target.__prevent_extensions__(context);
    };

    // 7. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target »)).
    let boolean_trap_result = trap
        .call(&handler.into(), &[target.clone().into()], context)?
        .to_boolean();

    // 8. If booleanTrapResult is true, then
    if boolean_trap_result && target.is_extensible(context)? {
        // a. Let extensibleTarget be ? IsExtensible(target).
        // b. If extensibleTarget is true, throw a TypeError exception.
        return Err(JsNativeError::typ()
            .with_message("Proxy trap failed to set extensible")
            .into());
    }

    // 9. Return booleanTrapResult.
    Ok(boolean_trap_result)
}

/// `10.5.5 [[GetOwnProperty]] ( P )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-getownproperty-p
pub(crate) fn proxy_exotic_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<Option<PropertyDescriptor>> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "getOwnPropertyDescriptor").
    let Some(trap) = handler.get_method(utf16!("getOwnPropertyDescriptor"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? target.[[GetOwnProperty]](P).
        return target.__get_own_property__(key, context);
    };

    // 7. Let trapResultObj be ? Call(trap, handler, « target, P »).
    let trap_result_obj = trap.call(
        &handler.into(),
        &[target.clone().into(), key.clone().into()],
        context,
    )?;

    // 8. If Type(trapResultObj) is neither Object nor Undefined, throw a TypeError exception.
    if !trap_result_obj.is_object() && !trap_result_obj.is_undefined() {
        return Err(JsNativeError::typ()
            .with_message("Proxy trap result is neither object nor undefined")
            .into());
    }

    // 9. Let targetDesc be ? target.[[GetOwnProperty]](P).
    let target_desc = target.__get_own_property__(key, context)?;

    // 10. If trapResultObj is undefined, then
    if trap_result_obj.is_undefined() {
        if let Some(desc) = target_desc {
            // b. If targetDesc.[[Configurable]] is false, throw a TypeError exception.
            if !desc.expect_configurable() {
                return Err(JsNativeError::typ()
                    .with_message(
                        "Proxy trap result is undefined adn target result is not configurable",
                    )
                    .into());
            }

            // c. Let extensibleTarget be ? IsExtensible(target).
            // d. If extensibleTarget is false, throw a TypeError exception.
            if !target.is_extensible(context)? {
                return Err(JsNativeError::typ()
                    .with_message("Proxy trap result is undefined and target is not extensible")
                    .into());
            }
            // e. Return undefined.
            return Ok(None);
        }

        // a. If targetDesc is undefined, return undefined.
        return Ok(None);
    }

    // 11. Let extensibleTarget be ? IsExtensible(target).
    let extensible_target = target.is_extensible(context)?;

    // 12. Let resultDesc be ? ToPropertyDescriptor(trapResultObj).
    let result_desc = trap_result_obj.to_property_descriptor(context)?;

    // 13. Call CompletePropertyDescriptor(resultDesc).
    let result_desc = result_desc.complete_property_descriptor();

    // 14. Let valid be IsCompatiblePropertyDescriptor(extensibleTarget, resultDesc, targetDesc).
    // 15. If valid is false, throw a TypeError exception.
    if !super::is_compatible_property_descriptor(
        extensible_target,
        result_desc.clone(),
        target_desc.clone(),
    ) {
        return Err(JsNativeError::typ()
            .with_message("Proxy trap returned unexpected property")
            .into());
    }

    // 16. If resultDesc.[[Configurable]] is false, then
    if !result_desc.expect_configurable() {
        // a. If targetDesc is undefined or targetDesc.[[Configurable]] is true, then
        match &target_desc {
            Some(desc) if !desc.expect_configurable() => {
                // b. If resultDesc has a [[Writable]] field and resultDesc.[[Writable]] is false, then
                if result_desc.writable() == Some(false) {
                    // i. If targetDesc.[[Writable]] is true, throw a TypeError exception.
                    if desc.expect_writable() {
                        return
                            Err(JsNativeError::typ().with_message("Proxy trap result is writable and not configurable while target result is not configurable").into())
                        ;
                    }
                }
            }
            // i. Throw a TypeError exception.
            _ => {
                return Err(JsNativeError::typ()
                    .with_message(
                        "Proxy trap result is not configurable and target result is undefined",
                    )
                    .into())
            }
        }
    }

    // 17. Return resultDesc.
    Ok(Some(result_desc))
}

/// `10.5.6 [[DefineOwnProperty]] ( P, Desc )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-defineownproperty-p-desc
pub(crate) fn proxy_exotic_define_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "defineProperty").
    let Some(trap) = handler.get_method(utf16!("defineProperty"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? target.[[DefineOwnProperty]](P, Desc).
        return target.__define_own_property__(key, desc, context);
    };

    // 7. Let descObj be FromPropertyDescriptor(Desc).
    let desc_obj = Object::from_property_descriptor(Some(desc.clone()), context);

    // 8. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, P, descObj »)).
    // 9. If booleanTrapResult is false, return false.
    if !trap
        .call(
            &handler.into(),
            &[target.clone().into(), key.clone().into(), desc_obj],
            context,
        )?
        .to_boolean()
    {
        return Ok(false);
    }

    // 10. Let targetDesc be ? target.[[GetOwnProperty]](P).
    let target_desc = target.__get_own_property__(key, context)?;

    // 11. Let extensibleTarget be ? IsExtensible(target).
    let extensible_target = target.is_extensible(context)?;

    // 12. If Desc has a [[Configurable]] field and if Desc.[[Configurable]] is false, then
    let setting_config_false = matches!(desc.configurable(), Some(false));

    match target_desc {
        // 14. If targetDesc is undefined, then
        None => {
            // a. If extensibleTarget is false, throw a TypeError exception.
            if !extensible_target {
                return Err(JsNativeError::typ()
                    .with_message("Proxy trap failed to set property")
                    .into());
            }

            // b. If settingConfigFalse is true, throw a TypeError exception.
            if setting_config_false {
                return Err(JsNativeError::typ()
                    .with_message("Proxy trap failed to set property")
                    .into());
            }
        }
        // 15. Else,
        Some(target_desc) => {
            // a. If IsCompatiblePropertyDescriptor(extensibleTarget, Desc, targetDesc) is false, throw a TypeError exception.
            if !super::is_compatible_property_descriptor(
                extensible_target,
                desc.clone(),
                Some(target_desc.clone()),
            ) {
                return Err(JsNativeError::typ()
                    .with_message("Proxy trap set property to unexpected value")
                    .into());
            }

            // b. If settingConfigFalse is true and targetDesc.[[Configurable]] is true, throw a TypeError exception.
            if setting_config_false && target_desc.expect_configurable() {
                return Err(JsNativeError::typ()
                    .with_message("Proxy trap set property with unexpected configurable field")
                    .into());
            }

            // c. If IsDataDescriptor(targetDesc) is true, targetDesc.[[Configurable]] is false, and targetDesc.[[Writable]] is true, then
            if target_desc.is_data_descriptor()
                && !target_desc.expect_configurable()
                && target_desc.expect_writable()
            {
                // i. If Desc has a [[Writable]] field and Desc.[[Writable]] is false, throw a TypeError exception.
                if let Some(writable) = desc.writable() {
                    if !writable {
                        return Err(JsNativeError::typ()
                            .with_message("Proxy trap set property with unexpected writable field")
                            .into());
                    }
                }
            }
        }
    }

    // 16. Return true.
    Ok(true)
}

/// `10.5.7 [[HasProperty]] ( P )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-hasproperty-p
pub(crate) fn proxy_exotic_has_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "has").
    let Some(trap) = handler.get_method(utf16!("has"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? target.[[HasProperty]](P).
        return target.has_property(key.clone(), context);
    };

    // 7. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, P »)).
    let boolean_trap_result = trap
        .call(
            &handler.into(),
            &[target.clone().into(), key.clone().into()],
            context,
        )?
        .to_boolean();

    // 8. If booleanTrapResult is false, then
    if !boolean_trap_result {
        // a. Let targetDesc be ? target.[[GetOwnProperty]](P).
        let target_desc = target.__get_own_property__(key, context)?;

        // b. If targetDesc is not undefined, then
        if let Some(target_desc) = target_desc {
            // i. If targetDesc.[[Configurable]] is false, throw a TypeError exception.
            if !target_desc.expect_configurable() {
                return Err(JsNativeError::typ()
                    .with_message("Proxy trap returned unexpected property")
                    .into());
            }

            // ii. Let extensibleTarget be ? IsExtensible(target).
            // iii. If extensibleTarget is false, throw a TypeError exception.
            if !target.is_extensible(context)? {
                return Err(JsNativeError::typ()
                    .with_message("Proxy trap returned unexpected property")
                    .into());
            }
        }
    }

    // 9. Return booleanTrapResult.
    Ok(boolean_trap_result)
}

/// `10.5.8 [[Get]] ( P, Receiver )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-get-p-receiver
pub(crate) fn proxy_exotic_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "get").
    let Some(trap) = handler.get_method(utf16!("get"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? target.[[Get]](P, Receiver).
        return target.__get__(key, receiver, context);
    };

    // 7. Let trapResult be ? Call(trap, handler, « target, P, Receiver »).
    let trap_result = trap.call(
        &handler.into(),
        &[target.clone().into(), key.clone().into(), receiver],
        context,
    )?;

    // 8. Let targetDesc be ? target.[[GetOwnProperty]](P).
    let target_desc = target.__get_own_property__(key, context)?;

    // 9. If targetDesc is not undefined and targetDesc.[[Configurable]] is false, then
    if let Some(target_desc) = target_desc {
        if !target_desc.expect_configurable() {
            // a. If IsDataDescriptor(targetDesc) is true and targetDesc.[[Writable]] is false, then
            if target_desc.is_data_descriptor() && !target_desc.expect_writable() {
                // i. If SameValue(trapResult, targetDesc.[[Value]]) is false, throw a TypeError exception.
                if !JsValue::same_value(&trap_result, target_desc.expect_value()) {
                    return Err(JsNativeError::typ()
                        .with_message("Proxy trap returned unexpected data descriptor")
                        .into());
                }
            }

            // b. If IsAccessorDescriptor(targetDesc) is true and targetDesc.[[Get]] is undefined, then
            if target_desc.is_accessor_descriptor() && target_desc.expect_get().is_undefined() {
                // i. If trapResult is not undefined, throw a TypeError exception.
                if !trap_result.is_undefined() {
                    return Err(JsNativeError::typ()
                        .with_message("Proxy trap returned unexpected accessor descriptor")
                        .into());
                }
            }
        }
    }

    // 10. Return trapResult.
    Ok(trap_result)
}

/// `10.5.9 [[Set]] ( P, V, Receiver )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-set-p-v-receiver
pub(crate) fn proxy_exotic_set(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "set").
    let Some(trap) = handler.get_method(utf16!("set"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? target.[[Set]](P, V, Receiver).
        return target.__set__(key, value, receiver, context);
    };

    // 7. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, P, V, Receiver »)).
    // 8. If booleanTrapResult is false, return false.
    if !trap
        .call(
            &handler.into(),
            &[
                target.clone().into(),
                key.clone().into(),
                value.clone(),
                receiver,
            ],
            context,
        )?
        .to_boolean()
    {
        return Ok(false);
    }

    // 9. Let targetDesc be ? target.[[GetOwnProperty]](P).
    let target_desc = target.__get_own_property__(&key, context)?;

    // 10. If targetDesc is not undefined and targetDesc.[[Configurable]] is false, then
    if let Some(target_desc) = target_desc {
        if !target_desc.expect_configurable() {
            // a. If IsDataDescriptor(targetDesc) is true and targetDesc.[[Writable]] is false, then
            if target_desc.is_data_descriptor() && !target_desc.expect_writable() {
                // i. If SameValue(V, targetDesc.[[Value]]) is false, throw a TypeError exception.
                if !JsValue::same_value(&value, target_desc.expect_value()) {
                    return Err(JsNativeError::typ()
                        .with_message("Proxy trap set unexpected data descriptor")
                        .into());
                }
            }

            // b. If IsAccessorDescriptor(targetDesc) is true, then
            if target_desc.is_accessor_descriptor() {
                // i. If targetDesc.[[Set]] is undefined, throw a TypeError exception.
                match target_desc.set() {
                    None | Some(&JsValue::Undefined) => {
                        return Err(JsNativeError::typ()
                            .with_message("Proxy trap set unexpected accessor descriptor")
                            .into());
                    }
                    _ => {}
                }
            }
        }
    }

    // 11. Return true.
    Ok(true)
}

/// `10.5.10 [[Delete]] ( P )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-delete-p
pub(crate) fn proxy_exotic_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "deleteProperty").
    let Some(trap) = handler.get_method(utf16!("deleteProperty"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? target.[[Delete]](P).
        return target.__delete__(key, context);
    };

    // 7. Let booleanTrapResult be ! ToBoolean(? Call(trap, handler, « target, P »)).
    // 8. If booleanTrapResult is false, return false.
    if !trap
        .call(
            &handler.into(),
            &[target.clone().into(), key.clone().into()],
            context,
        )?
        .to_boolean()
    {
        return Ok(false);
    }

    // 9. Let targetDesc be ? target.[[GetOwnProperty]](P).
    match target.__get_own_property__(key, context)? {
        // 10. If targetDesc is undefined, return true.
        None => return Ok(true),
        // 11. If targetDesc.[[Configurable]] is false, throw a TypeError exception.
        Some(target_desc) => {
            if !target_desc.expect_configurable() {
                return Err(JsNativeError::typ()
                    .with_message("Proxy trap failed to delete property")
                    .into());
            }
        }
    }

    // 12. Let extensibleTarget be ? IsExtensible(target).
    // 13. If extensibleTarget is false, throw a TypeError exception.
    if !target.is_extensible(context)? {
        return Err(JsNativeError::typ()
            .with_message("Proxy trap failed to delete property")
            .into());
    }

    // 14. Return true.
    Ok(true)
}

/// `10.5.11 [[OwnPropertyKeys]] ( )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
pub(crate) fn proxy_exotic_own_property_keys(
    obj: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<Vec<PropertyKey>> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "ownKeys").
    let Some(trap) = handler.get_method(utf16!("ownKeys"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? target.[[OwnPropertyKeys]]().
        return target.__own_property_keys__(context);
    };

    // 7. Let trapResultArray be ? Call(trap, handler, « target »).
    let trap_result_array = trap.call(&handler.into(), &[target.clone().into()], context)?;

    // 8. Let trapResult be ? CreateListFromArrayLike(trapResultArray, « String, Symbol »).
    let trap_result_raw =
        trap_result_array.create_list_from_array_like(&[Type::String, Type::Symbol], context)?;

    // 9. If trapResult contains any duplicate entries, throw a TypeError exception.
    let mut unchecked_result_keys: FxHashSet<PropertyKey> = FxHashSet::default();
    let mut trap_result = Vec::new();
    for value in &trap_result_raw {
        match value {
            JsValue::String(s) => {
                if !unchecked_result_keys.insert(s.clone().into()) {
                    return Err(JsNativeError::typ()
                        .with_message("Proxy trap result contains duplicate string property keys")
                        .into());
                }
                trap_result.push(s.clone().into());
            }
            JsValue::Symbol(s) => {
                if !unchecked_result_keys.insert(s.clone().into()) {
                    return Err(JsNativeError::typ()
                        .with_message("Proxy trap result contains duplicate symbol property keys")
                        .into());
                }
                trap_result.push(s.clone().into());
            }
            _ => {}
        }
    }

    // 10. Let extensibleTarget be ? IsExtensible(target).
    let extensible_target = target.is_extensible(context)?;

    // 11. Let targetKeys be ? target.[[OwnPropertyKeys]]().
    // 12. Assert: targetKeys is a List of property keys.
    // 13. Assert: targetKeys contains no duplicate entries.
    let target_keys = target.__own_property_keys__(context)?;

    // 14. Let targetConfigurableKeys be a new empty List.
    // 15. Let targetNonconfigurableKeys be a new empty List.
    let mut target_configurable_keys = Vec::new();
    let mut target_nonconfigurable_keys = Vec::new();

    // 16. For each element key of targetKeys, do
    for key in target_keys {
        // a. Let desc be ? target.[[GetOwnProperty]](key).
        match target.__get_own_property__(&key, context)? {
            // b. If desc is not undefined and desc.[[Configurable]] is false, then
            Some(desc) if !desc.expect_configurable() => {
                // i. Append key as an element of targetNonconfigurableKeys.
                target_nonconfigurable_keys.push(key);
            }
            // c. Else,
            _ => {
                // i. Append key as an element of targetConfigurableKeys.
                target_configurable_keys.push(key);
            }
        }
    }

    // 17. If extensibleTarget is true and targetNonconfigurableKeys is empty, then
    if extensible_target && target_nonconfigurable_keys.is_empty() {
        // a. Return trapResult.
        return Ok(trap_result);
    }

    // 18. Let uncheckedResultKeys be a List whose elements are the elements of trapResult.
    // 19. For each element key of targetNonconfigurableKeys, do
    for key in target_nonconfigurable_keys {
        // a. If key is not an element of uncheckedResultKeys, throw a TypeError exception.
        // b. Remove key from uncheckedResultKeys.
        if !unchecked_result_keys.remove(&key) {
            return Err(JsNativeError::typ()
                .with_message("Proxy trap failed to return all non-configurable property keys")
                .into());
        }
    }

    // 20. If extensibleTarget is true, return trapResult.
    if extensible_target {
        return Ok(trap_result);
    }

    // 21. For each element key of targetConfigurableKeys, do
    for key in target_configurable_keys {
        // a. If key is not an element of uncheckedResultKeys, throw a TypeError exception.
        // b. Remove key from uncheckedResultKeys.
        if !unchecked_result_keys.remove(&key) {
            return Err(JsNativeError::typ()
                .with_message("Proxy trap failed to return all configurable property keys")
                .into());
        }
    }

    // 22. If uncheckedResultKeys is not empty, throw a TypeError exception.
    if !unchecked_result_keys.is_empty() {
        return Err(JsNativeError::typ()
            .with_message("Proxy trap failed to return all property keys")
            .into());
    }

    // 23. Return trapResult.
    Ok(trap_result)
}

/// `10.5.12 [[Call]] ( thisArgument, argumentsList )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist
fn proxy_exotic_call(
    obj: &JsObject,
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Let trap be ? GetMethod(handler, "apply").
    let Some(trap) = handler.get_method(utf16!("apply"), context)? else {
        // 6. If trap is undefined, then
        // a. Return ? Call(target, thisArgument, argumentsList).
        return target.call(this, args, context);
    };

    // 7. Let argArray be ! CreateArrayFromList(argumentsList).
    let arg_array = array::Array::create_array_from_list(args.to_vec(), context);

    // 8. Return ? Call(trap, handler, « target, thisArgument, argArray »).
    trap.call(
        &handler.into(),
        &[target.clone().into(), this.clone(), arg_array.into()],
        context,
    )
}

/// `[[Construct]] ( argumentsList, newTarget )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
fn proxy_exotic_construct(
    obj: &JsObject,
    args: &[JsValue],
    new_target: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<JsObject> {
    // 1. Let handler be O.[[ProxyHandler]].
    // 2. If handler is null, throw a TypeError exception.
    // 3. Assert: Type(handler) is Object.
    // 4. Let target be O.[[ProxyTarget]].
    let (target, handler) = obj
        .borrow()
        .as_proxy()
        .expect("Proxy object internal internal method called on non-proxy object")
        .try_data()?;

    // 5. Assert: IsConstructor(target) is true.
    assert!(target.is_constructor());

    // 6. Let trap be ? GetMethod(handler, "construct").
    let Some(trap) = handler.get_method(utf16!("construct"), context)? else {
        // 7. If trap is undefined, then
        // a. Return ? Construct(target, argumentsList, newTarget).
        return target.construct(args, Some(new_target), context);
    };

    // 8. Let argArray be ! CreateArrayFromList(argumentsList).
    let arg_array = array::Array::create_array_from_list(args.to_vec(), context);

    // 9. Let newObj be ? Call(trap, handler, « target, argArray, newTarget »).
    let new_obj = trap.call(
        &handler.into(),
        &[
            target.clone().into(),
            arg_array.into(),
            new_target.clone().into(),
        ],
        context,
    )?;

    // 10. If Type(newObj) is not Object, throw a TypeError exception.
    let new_obj = new_obj.as_object().cloned().ok_or_else(|| {
        JsNativeError::typ().with_message("Proxy trap constructor returned non-object value")
    })?;

    // 11. Return newObj.
    Ok(new_obj)
}
