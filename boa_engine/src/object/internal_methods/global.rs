use crate::{
    object::{
        internal_methods::ordinary_get_prototype_of, InternalObjectMethods, JsObject, JsPrototype,
        ORDINARY_INTERNAL_METHODS,
    },
    property::{DescriptorKind, PropertyDescriptor, PropertyKey},
    value::JsValue,
    Context, JsResult,
};
use boa_profiler::Profiler;

/// Definitions of the internal object methods for global object.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-global-object
pub(crate) static GLOBAL_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __get_prototype_of__: global_get_prototype_of,
    __set_prototype_of__: global_set_prototype_of,
    __is_extensible__: global_is_extensible,
    __prevent_extensions__: global_prevent_extensions,
    __get_own_property__: global_get_own_property,
    __define_own_property__: global_define_own_property,
    __has_property__: global_has_property,
    __get__: global_get,
    __set__: global_set,
    __delete__: global_delete,
    __own_property_keys__: global_own_property_keys,
    ..ORDINARY_INTERNAL_METHODS
};

/// Abstract operation `OrdinaryGetPrototypeOf`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarygetprototypeof
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn global_get_prototype_of(
    _: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<JsPrototype> {
    // 1. Return O.[[Prototype]].
    Ok(context.realm.global_prototype.clone())
}

/// Abstract operation `OrdinarySetPrototypeOf`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarysetprototypeof
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn global_set_prototype_of(
    _: &JsObject,
    val: JsPrototype,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Assert: Either Type(V) is Object or Type(V) is Null.
    {
        // 2. Let current be O.[[Prototype]].
        let current = &context.realm.global_prototype;

        // 3. If SameValue(V, current) is true, return true.
        if val == *current {
            return Ok(true);
        }
    }

    // 4. Let extensible be O.[[Extensible]].
    // 5. If extensible is false, return false.
    if !context.realm.global_extensible {
        return Ok(false);
    }

    // 6. Let p be V.
    let mut p = val.clone();

    // 7. Let done be false.
    // 8. Repeat, while done is false,
    // a. If p is null, set done to true.
    while let Some(proto) = p {
        // b. Else if SameValue(p, O) is true, return false.
        if &proto == context.realm.global_object() {
            return Ok(false);
        }
        // c. Else,
        // i. If p.[[GetPrototypeOf]] is not the ordinary object internal method defined
        // in 10.1.1, set done to true.
        else if proto.borrow().data.internal_methods.__get_prototype_of__ as usize
            != ordinary_get_prototype_of as usize
        {
            break;
        }
        // ii. Else, set p to p.[[Prototype]].
        p = proto.prototype().clone();
    }

    // 9. Set O.[[Prototype]] to V.
    context.realm.global_object().set_prototype(val);

    // 10. Return true.
    Ok(true)
}

/// Abstract operation `OrdinaryGetOwnProperty`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarygetownproperty
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn global_get_own_property(
    _obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<Option<PropertyDescriptor>> {
    let _timer = Profiler::global().start_event("Object::global_get_own_property", "object");
    // 1. Assert: IsPropertyKey(P) is true.
    // 2. If O does not have an own property with key P, return undefined.
    // 3. Let D be a newly created Property Descriptor with no fields.
    // 4. Let X be O's own property whose key is P.
    // 5. If X is a data property, then
    //      a. Set D.[[Value]] to the value of X's [[Value]] attribute.
    //      b. Set D.[[Writable]] to the value of X's [[Writable]] attribute.
    // 6. Else,
    //      a. Assert: X is an accessor property.
    //      b. Set D.[[Get]] to the value of X's [[Get]] attribute.
    //      c. Set D.[[Set]] to the value of X's [[Set]] attribute.
    // 7. Set D.[[Enumerable]] to the value of X's [[Enumerable]] attribute.
    // 8. Set D.[[Configurable]] to the value of X's [[Configurable]] attribute.
    // 9. Return D.
    Ok(context.realm.global_property_map.get(key))
}

/// Abstract operation `OrdinaryIsExtensible`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinaryisextensible
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn global_is_extensible(_obj: &JsObject, context: &mut Context<'_>) -> JsResult<bool> {
    // 1. Return O.[[Extensible]].
    Ok(context.realm.global_extensible)
}

/// Abstract operation `OrdinaryPreventExtensions`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarypreventextensions
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn global_prevent_extensions(
    _obj: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Set O.[[Extensible]] to false.
    context.realm.global_extensible = false;

    // 2. Return true.
    Ok(true)
}

/// Abstract operation `OrdinaryDefineOwnProperty`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarydefineownproperty
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn global_define_own_property(
    obj: &JsObject,
    key: PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let _timer = Profiler::global().start_event("Object::global_define_own_property", "object");
    // 1. Let current be ? O.[[GetOwnProperty]](P).
    let current = global_get_own_property(obj, &key, context)?;

    // 2. Let extensible be ? IsExtensible(O).
    let extensible = obj.__is_extensible__(context)?;

    // 3. Return ValidateAndApplyPropertyDescriptor(O, P, extensible, Desc, current).
    Ok(validate_and_apply_property_descriptor(
        &key, extensible, desc, current, context,
    ))
}

/// Abstract operation `OrdinaryHasProperty`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinaryhasproperty
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn global_has_property(
    _obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let _timer = Profiler::global().start_event("Object::global_has_property", "object");
    // 1. Assert: IsPropertyKey(P) is true.
    // 2. Let hasOwn be ? O.[[GetOwnProperty]](P).
    // 3. If hasOwn is not undefined, return true.
    if context.realm.global_property_map.contains_key(key) {
        Ok(true)
    } else {
        // 4. Let parent be ? O.[[GetPrototypeOf]]().
        let parent = context.realm.global_prototype.clone();

        // 5. If parent is not null, then
        // a. Return ? parent.[[HasProperty]](P).
        // 6. Return false.
        parent.map_or(Ok(false), |obj| obj.__has_property__(key, context))
    }
}

/// Abstract operation `OrdinaryGet`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinaryget
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn global_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    let _timer = Profiler::global().start_event("Object::global_get", "object");
    // 1. Assert: IsPropertyKey(P) is true.
    // 2. Let desc be ? O.[[GetOwnProperty]](P).
    match global_get_own_property(obj, key, context)? {
        // If desc is undefined, then
        None => {
            // a. Let parent be ? O.[[GetPrototypeOf]]().
            if let Some(parent) = context.realm.global_prototype.clone() {
                // c. Return ? parent.[[Get]](P, Receiver).
                parent.__get__(key, receiver, context)
            }
            // b. If parent is null, return undefined.
            else {
                Ok(JsValue::undefined())
            }
        }
        Some(ref desc) => match desc.kind() {
            // 4. If IsDataDescriptor(desc) is true, return desc.[[Value]].
            DescriptorKind::Data {
                value: Some(value), ..
            } => Ok(value.clone()),
            // 5. Assert: IsAccessorDescriptor(desc) is true.
            // 6. Let getter be desc.[[Get]].
            DescriptorKind::Accessor { get: Some(get), .. } if !get.is_undefined() => {
                // 8. Return ? Call(getter, Receiver).
                get.call(&receiver, &[], context)
            }
            // 7. If getter is undefined, return undefined.
            _ => Ok(JsValue::undefined()),
        },
    }
}

/// Abstract operation `OrdinarySet`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinaryset
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn global_set(
    _obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    _receiver: JsValue,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    global_set_no_receiver(&key, value, context)
}

pub(crate) fn global_set_no_receiver(
    key: &PropertyKey,
    value: JsValue,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let _timer = Profiler::global().start_event("Object::global_set", "object");

    // 1. Assert: IsPropertyKey(P) is true.
    // 2. Let ownDesc be ? O.[[GetOwnProperty]](P).
    // 3. Return OrdinarySetWithOwnDescriptor(O, P, V, Receiver, ownDesc).

    // OrdinarySetWithOwnDescriptor ( O, P, V, Receiver, ownDesc )
    // https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-ordinarysetwithowndescriptor

    // 1. Assert: IsPropertyKey(P) is true.
    let own_desc = if let Some(desc) = context.realm.global_property_map.get(key) {
        desc
    }
    // c. Else,
    else {
        PropertyDescriptor::builder()
            .value(value.clone())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build()
    };

    // 3. If IsDataDescriptor(ownDesc) is true, then
    if own_desc.is_data_descriptor() {
        // a. If ownDesc.[[Writable]] is false, return false.
        if !own_desc.expect_writable() {
            return Ok(false);
        }

        // c. Let existingDescriptor be ? Receiver.[[GetOwnProperty]](P).
        // d. If existingDescriptor is not undefined, then
        let desc = if let Some(existing_desc) = context.realm.global_property_map.get(key) {
            // i. If IsAccessorDescriptor(existingDescriptor) is true, return false.
            if existing_desc.is_accessor_descriptor() {
                return Ok(false);
            }

            // ii. If existingDescriptor.[[Writable]] is false, return false.
            if !existing_desc.expect_writable() {
                return Ok(false);
            }

            // iii. Let valueDesc be the PropertyDescriptor { [[Value]]: V }.
            // iv. Return ? Receiver.[[DefineOwnProperty]](P, valueDesc).
            PropertyDescriptor::builder().value(value).build()
        } else {
            // i. Assert: Receiver does not currently have a property P.
            // ii. Return ? CreateDataProperty(Receiver, P, V).
            PropertyDescriptor::builder()
                .value(value)
                .writable(true)
                .enumerable(true)
                .configurable(true)
                .build()
        };

        // 1. Let current be ? O.[[GetOwnProperty]](P).
        let current = context.realm.global_property_map.get(key);

        // 2. Let extensible be ? IsExtensible(O).
        let extensible = context.realm.global_extensible;

        // 3. Return ValidateAndApplyPropertyDescriptor(O, P, extensible, Desc, current).
        return Ok(validate_and_apply_property_descriptor(
            key, extensible, desc, current, context,
        ));
    }

    // 4. Assert: IsAccessorDescriptor(ownDesc) is true.
    debug_assert!(own_desc.is_accessor_descriptor());

    // 5. Let setter be ownDesc.[[Set]].
    match own_desc.set() {
        Some(set) if !set.is_undefined() => {
            // 7. Perform ? Call(setter, Receiver, « V »).
            set.call(&context.global_object().clone().into(), &[value], context)?;

            // 8. Return true.
            Ok(true)
        }
        // 6. If setter is undefined, return false.
        _ => Ok(false),
    }
}

/// Abstract operation `OrdinaryDelete`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarydelete
#[allow(clippy::unnecessary_wraps, clippy::needless_pass_by_value)]
pub(crate) fn global_delete(
    _obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    global_delete_no_receiver(key, context)
}

/// Abstract operation `OrdinaryDelete`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarydelete
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn global_delete_no_receiver(
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let _timer = Profiler::global().start_event("Object::global_delete", "object");
    // 1. Assert: IsPropertyKey(P) is true.
    // 2. Let desc be ? O.[[GetOwnProperty]](P).
    match context.realm.global_property_map.get(key) {
        // 4. If desc.[[Configurable]] is true, then
        Some(desc) if desc.expect_configurable() => {
            // a. Remove the own property with name P from O.
            context.realm.global_property_map.remove(key);
            // b. Return true.
            Ok(true)
        }
        // 5. Return false.
        Some(_) => Ok(false),
        // 3. If desc is undefined, return true.
        None => Ok(true),
    }
}

/// Abstract operation `OrdinaryOwnPropertyKeys`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinaryownpropertykeys
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn global_own_property_keys(
    _: &JsObject,
    context: &mut Context<'_>,
) -> JsResult<Vec<PropertyKey>> {
    // 1. Let keys be a new empty List.
    let mut keys = Vec::new();

    let ordered_indexes = {
        let mut indexes: Vec<_> = context
            .realm
            .global_property_map
            .index_property_keys()
            .collect();
        indexes.sort_unstable();
        indexes
    };

    // 2. For each own property key P of O such that P is an array index, in ascending numeric index order, do
    // a. Add P as the last element of keys.
    keys.extend(ordered_indexes.into_iter().map(Into::into));

    // 3. For each own property key P of O such that Type(P) is String and P is not an array index, in ascending chronological order of property creation, do
    // a. Add P as the last element of keys.
    keys.extend(
        context
            .realm
            .global_property_map
            .string_property_keys()
            .cloned()
            .map(Into::into),
    );

    // 4. For each own property key P of O such that Type(P) is Symbol, in ascending chronological order of property creation, do
    // a. Add P as the last element of keys.
    keys.extend(
        context
            .realm
            .global_property_map
            .symbol_property_keys()
            .cloned()
            .map(Into::into),
    );

    // 5. Return keys.
    Ok(keys)
}

/// Abstract operation `ValidateAndApplyPropertyDescriptor`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-validateandapplypropertydescriptor
pub(crate) fn validate_and_apply_property_descriptor(
    key: &PropertyKey,
    extensible: bool,
    desc: PropertyDescriptor,
    current: Option<PropertyDescriptor>,
    context: &mut Context<'_>,
) -> bool {
    let _timer = Profiler::global().start_event(
        "Object::global_validate_and_apply_property_descriptor",
        "object",
    );
    // 1. Assert: If O is not undefined, then IsPropertyKey(P) is true.

    let Some(mut current) = current else {
        // 2. If current is undefined, then
        // a. If extensible is false, return false.
        if !extensible {
            return false;
        }

        // b. Assert: extensible is true.
        context.realm.global_property_map.insert(
            key,
            // c. If IsGenericDescriptor(Desc) is true or IsDataDescriptor(Desc) is true, then
            if desc.is_generic_descriptor() || desc.is_data_descriptor() {
                // i. If O is not undefined, create an own data property named P of
                // object O whose [[Value]], [[Writable]], [[Enumerable]], and
                // [[Configurable]] attribute values are described by Desc.
                // If the value of an attribute field of Desc is absent, the attribute
                // of the newly created property is set to its default value.
                desc.into_data_defaulted()
            }
            // d. Else,
            else {
                // i. Assert: ! IsAccessorDescriptor(Desc) is true.

                // ii. If O is not undefined, create an own accessor property named P
                // of object O whose [[Get]], [[Set]], [[Enumerable]], and [[Configurable]]
                // attribute values are described by Desc. If the value of an attribute field
                // of Desc is absent, the attribute of the newly created property is set to
                // its default value.
                desc.into_accessor_defaulted()
            },
        );

        // e. Return true.
        return true;
    };

    // 3. If every field in Desc is absent, return true.
    if desc.is_empty() {
        return true;
    }

    // 4. If current.[[Configurable]] is false, then
    if !current.expect_configurable() {
        // a. If Desc.[[Configurable]] is present and its value is true, return false.
        if matches!(desc.configurable(), Some(true)) {
            return false;
        }

        // b. If Desc.[[Enumerable]] is present and ! SameValue(Desc.[[Enumerable]], current.[[Enumerable]])
        // is false, return false.
        if matches!(desc.enumerable(), Some(desc_enum) if desc_enum != current.expect_enumerable())
        {
            return false;
        }
    }

    // 5. If ! IsGenericDescriptor(Desc) is true, then
    if desc.is_generic_descriptor() {
        // a. NOTE: No further validation is required.
    }
    // 6. Else if ! SameValue(! IsDataDescriptor(current), ! IsDataDescriptor(Desc)) is false, then
    else if current.is_data_descriptor() != desc.is_data_descriptor() {
        // a. If current.[[Configurable]] is false, return false.
        if !current.expect_configurable() {
            return false;
        }

        // b. If IsDataDescriptor(current) is true, then
        if current.is_data_descriptor() {
            // i. If O is not undefined, convert the property named P of object O from a data
            // property to an accessor property. Preserve the existing values of the converted
            // property's [[Configurable]] and [[Enumerable]] attributes and set the rest of
            // the property's attributes to their default values.
            current = current.into_accessor_defaulted();
        }
        // c. Else,
        else {
            // i. If O is not undefined, convert the property named P of object O from an
            // accessor property to a data property. Preserve the existing values of the
            // converted property's [[Configurable]] and [[Enumerable]] attributes and set
            // the rest of the property's attributes to their default values.
            current = current.into_data_defaulted();
        }
    }
    // 7. Else if IsDataDescriptor(current) and IsDataDescriptor(Desc) are both true, then
    else if current.is_data_descriptor() && desc.is_data_descriptor() {
        // a. If current.[[Configurable]] is false and current.[[Writable]] is false, then
        if !current.expect_configurable() && !current.expect_writable() {
            // i. If Desc.[[Writable]] is present and Desc.[[Writable]] is true, return false.
            if matches!(desc.writable(), Some(true)) {
                return false;
            }
            // ii. If Desc.[[Value]] is present and SameValue(Desc.[[Value]], current.[[Value]]) is false, return false.
            if matches!(desc.value(), Some(value) if !JsValue::same_value(value, current.expect_value()))
            {
                return false;
            }
            // iii. Return true.
            return true;
        }
    }
    // 8. Else,
    // a. Assert: ! IsAccessorDescriptor(current) and ! IsAccessorDescriptor(Desc) are both true.
    // b. If current.[[Configurable]] is false, then
    else if !current.expect_configurable() {
        // i. If Desc.[[Set]] is present and SameValue(Desc.[[Set]], current.[[Set]]) is false, return false.
        if matches!(desc.set(), Some(set) if !JsValue::same_value(set, current.expect_set())) {
            return false;
        }

        // ii. If Desc.[[Get]] is present and SameValue(Desc.[[Get]], current.[[Get]]) is false, return false.
        if matches!(desc.get(), Some(get) if !JsValue::same_value(get, current.expect_get())) {
            return false;
        }
        // iii. Return true.
        return true;
    }

    // 9. If O is not undefined, then
    // a. For each field of Desc that is present, set the corresponding attribute of the
    // property named P of object O to the value of the field.
    current.fill_with(&desc);
    context.realm.global_property_map.insert(key, current);

    // 10. Return true.
    true
}
