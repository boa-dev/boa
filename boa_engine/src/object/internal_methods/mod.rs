//! This module defines the object internal methods.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots

use super::{JsPrototype, PROTOTYPE};
use crate::{
    context::intrinsics::{StandardConstructor, StandardConstructors},
    object::JsObject,
    property::{DescriptorKind, PropertyDescriptor, PropertyKey},
    value::JsValue,
    Context, JsResult,
};
use boa_profiler::Profiler;

pub(super) mod arguments;
pub(super) mod array;
pub(super) mod bound_function;
pub(super) mod function;
pub(super) mod immutable_prototype;
pub(super) mod integer_indexed;
pub(super) mod module_namespace;
pub(super) mod proxy;
pub(super) mod string;

pub(crate) use array::ARRAY_EXOTIC_INTERNAL_METHODS;

impl JsObject {
    /// Internal method `[[GetPrototypeOf]]`
    ///
    /// Return either the prototype of this object or null.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
    #[track_caller]
    pub(crate) fn __get_prototype_of__(&self, context: &mut Context<'_>) -> JsResult<JsPrototype> {
        let _timer = Profiler::global().start_event("Object::__get_prototype_of__", "object");
        (self.vtable().__get_prototype_of__)(self, context)
    }

    /// Internal method `[[SetPrototypeOf]]`
    ///
    /// Set the property of a specified object to another object or `null`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
    pub(crate) fn __set_prototype_of__(
        &self,
        val: JsPrototype,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        let _timer = Profiler::global().start_event("Object::__set_prototype_of__", "object");
        (self.vtable().__set_prototype_of__)(self, val, context)
    }

    /// Internal method `[[IsExtensible]]`
    ///
    /// Check if the object is extensible.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible
    pub(crate) fn __is_extensible__(&self, context: &mut Context<'_>) -> JsResult<bool> {
        let _timer = Profiler::global().start_event("Object::__is_extensible__", "object");
        (self.vtable().__is_extensible__)(self, context)
    }

    /// Internal method `[[PreventExtensions]]`
    ///
    /// Disable extensibility for this object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
    pub(crate) fn __prevent_extensions__(&self, context: &mut Context<'_>) -> JsResult<bool> {
        let _timer = Profiler::global().start_event("Object::__prevent_extensions__", "object");
        (self.vtable().__prevent_extensions__)(self, context)
    }

    /// Internal method `[[GetOwnProperty]]`
    ///
    /// Get the specified property of this object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
    pub(crate) fn __get_own_property__(
        &self,
        key: &PropertyKey,
        context: &mut Context<'_>,
    ) -> JsResult<Option<PropertyDescriptor>> {
        let _timer = Profiler::global().start_event("Object::__get_own_property__", "object");
        (self.vtable().__get_own_property__)(self, key, context)
    }

    /// Internal method `[[DefineOwnProperty]]`
    ///
    /// Define a new property of this object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc
    pub(crate) fn __define_own_property__(
        &self,
        key: &PropertyKey,
        desc: PropertyDescriptor,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        let _timer = Profiler::global().start_event("Object::__define_own_property__", "object");
        (self.vtable().__define_own_property__)(self, key, desc, context)
    }

    /// Internal method `[[hasProperty]]`.
    ///
    /// Check if the object or its prototype has the required property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
    pub(crate) fn __has_property__(
        &self,
        key: &PropertyKey,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        let _timer = Profiler::global().start_event("Object::__has_property__", "object");
        (self.vtable().__has_property__)(self, key, context)
    }

    /// Internal method `[[Get]]`
    ///
    /// Get the specified property of this object or its prototype.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver
    pub(crate) fn __get__(
        &self,
        key: &PropertyKey,
        receiver: JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let _timer = Profiler::global().start_event("Object::__get__", "object");
        (self.vtable().__get__)(self, key, receiver, context)
    }

    /// Internal method `[[Set]]`
    ///
    /// Set the specified property of this object or its prototype to the provided value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver
    pub(crate) fn __set__(
        &self,
        key: PropertyKey,
        value: JsValue,
        receiver: JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        let _timer = Profiler::global().start_event("Object::__set__", "object");
        (self.vtable().__set__)(self, key, value, receiver, context)
    }

    /// Internal method `[[Delete]]`
    ///
    /// Delete the specified own property of this object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-delete-p
    pub(crate) fn __delete__(
        &self,
        key: &PropertyKey,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        let _timer = Profiler::global().start_event("Object::__delete__", "object");
        (self.vtable().__delete__)(self, key, context)
    }

    /// Internal method `[[OwnPropertyKeys]]`
    ///
    /// Get all the keys of the properties of this object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-ownpropertykeys
    #[track_caller]
    pub(crate) fn __own_property_keys__(
        &self,
        context: &mut Context<'_>,
    ) -> JsResult<Vec<PropertyKey>> {
        let _timer = Profiler::global().start_event("Object::__own_property_keys__", "object");
        (self.vtable().__own_property_keys__)(self, context)
    }

    /// Internal method `[[Call]]`
    ///
    /// Call this object if it has a `[[Call]]` internal method.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects-call-thisargument-argumentslist
    #[track_caller]
    pub(crate) fn __call__(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let _timer = Profiler::global().start_event("Object::__call__", "object");
        self.vtable()
            .__call__
            .expect("called `[[Call]]` for object without a `[[Call]]` internal method")(
            self, this, args, context,
        )
    }

    /// Internal method `[[Construct]]`
    ///
    /// Construct a new instance of this object if this object has a `[[Construct]]` internal method.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget
    #[track_caller]
    pub(crate) fn __construct__(
        &self,
        args: &[JsValue],
        new_target: &Self,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let _timer = Profiler::global().start_event("Object::__construct__", "object");
        self.vtable()
            .__construct__
            .expect("called `[[Construct]]` for object without a `[[Construct]]` internal method")(
            self, args, new_target, context,
        )
    }
}

/// Definitions of the internal object methods for ordinary objects.
///
/// If you want to implement an exotic object, create a new `static InternalObjectMethods`
/// overriding the desired internal methods with the definitions of the spec
/// and set all other methods to the default ordinary values, if necessary.
///
/// E.g. `string::STRING_EXOTIC_INTERNAL_METHODS`
///
/// Then, reference this static in the creation phase of an `ObjectData`.
///
/// E.g. `ObjectData::string`
pub(crate) static ORDINARY_INTERNAL_METHODS: InternalObjectMethods = InternalObjectMethods {
    __get_prototype_of__: ordinary_get_prototype_of,
    __set_prototype_of__: ordinary_set_prototype_of,
    __is_extensible__: ordinary_is_extensible,
    __prevent_extensions__: ordinary_prevent_extensions,
    __get_own_property__: ordinary_get_own_property,
    __define_own_property__: ordinary_define_own_property,
    __has_property__: ordinary_has_property,
    __get__: ordinary_get,
    __set__: ordinary_set,
    __delete__: ordinary_delete,
    __own_property_keys__: ordinary_own_property_keys,
    __call__: None,
    __construct__: None,
};

/// The internal representation of the internal methods of a `JsObject`.
///
/// This struct allows us to dynamically dispatch exotic objects with their
/// exclusive definitions of the internal methods, without having to
/// resort to `dyn Object`.
///
/// For a guide on how to implement exotic internal methods, see `ORDINARY_INTERNAL_METHODS`.
#[derive(Clone, Copy)]
#[allow(clippy::type_complexity)]
pub(crate) struct InternalObjectMethods {
    pub(crate) __get_prototype_of__: fn(&JsObject, &mut Context<'_>) -> JsResult<JsPrototype>,
    pub(crate) __set_prototype_of__: fn(&JsObject, JsPrototype, &mut Context<'_>) -> JsResult<bool>,
    pub(crate) __is_extensible__: fn(&JsObject, &mut Context<'_>) -> JsResult<bool>,
    pub(crate) __prevent_extensions__: fn(&JsObject, &mut Context<'_>) -> JsResult<bool>,
    pub(crate) __get_own_property__:
        fn(&JsObject, &PropertyKey, &mut Context<'_>) -> JsResult<Option<PropertyDescriptor>>,
    pub(crate) __define_own_property__:
        fn(&JsObject, &PropertyKey, PropertyDescriptor, &mut Context<'_>) -> JsResult<bool>,
    pub(crate) __has_property__: fn(&JsObject, &PropertyKey, &mut Context<'_>) -> JsResult<bool>,
    pub(crate) __get__: fn(&JsObject, &PropertyKey, JsValue, &mut Context<'_>) -> JsResult<JsValue>,
    pub(crate) __set__:
        fn(&JsObject, PropertyKey, JsValue, JsValue, &mut Context<'_>) -> JsResult<bool>,
    pub(crate) __delete__: fn(&JsObject, &PropertyKey, &mut Context<'_>) -> JsResult<bool>,
    pub(crate) __own_property_keys__: fn(&JsObject, &mut Context<'_>) -> JsResult<Vec<PropertyKey>>,
    pub(crate) __call__:
        Option<fn(&JsObject, &JsValue, &[JsValue], &mut Context<'_>) -> JsResult<JsValue>>,
    pub(crate) __construct__:
        Option<fn(&JsObject, &[JsValue], &JsObject, &mut Context<'_>) -> JsResult<JsObject>>,
}

/// Abstract operation `OrdinaryGetPrototypeOf`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarygetprototypeof
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn ordinary_get_prototype_of(
    obj: &JsObject,
    _context: &mut Context<'_>,
) -> JsResult<JsPrototype> {
    let _timer = Profiler::global().start_event("Object::ordinary_get_prototype_of", "object");

    // 1. Return O.[[Prototype]].
    Ok(obj.prototype().as_ref().cloned())
}

/// Abstract operation `OrdinarySetPrototypeOf`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarysetprototypeof
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn ordinary_set_prototype_of(
    obj: &JsObject,
    val: JsPrototype,
    _: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Assert: Either Type(V) is Object or Type(V) is Null.
    // 2. Let current be O.[[Prototype]].
    let current = obj.prototype();

    // 3. If SameValue(V, current) is true, return true.
    if val == current {
        return Ok(true);
    }

    // 4. Let extensible be O.[[Extensible]].
    // 5. If extensible is false, return false.
    if !obj.extensible() {
        return Ok(false);
    }

    // 6. Let p be V.
    let mut p = val.clone();

    // 7. Let done be false.
    // 8. Repeat, while done is false,
    // a. If p is null, set done to true.
    while let Some(proto) = p {
        // b. Else if SameValue(p, O) is true, return false.
        if &proto == obj {
            return Ok(false);
        }
        // c. Else,
        // i. If p.[[GetPrototypeOf]] is not the ordinary object internal method defined
        // in 10.1.1, set done to true.
        else if proto.vtable().__get_prototype_of__ as usize != ordinary_get_prototype_of as usize
        {
            break;
        }
        // ii. Else, set p to p.[[Prototype]].
        p = proto.prototype();
    }

    // 9. Set O.[[Prototype]] to V.
    obj.set_prototype(val);

    // 10. Return true.
    Ok(true)
}

/// Abstract operation `OrdinaryIsExtensible`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinaryisextensible
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn ordinary_is_extensible(obj: &JsObject, _context: &mut Context<'_>) -> JsResult<bool> {
    // 1. Return O.[[Extensible]].
    Ok(obj.borrow().extensible)
}

/// Abstract operation `OrdinaryPreventExtensions`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarypreventextensions
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn ordinary_prevent_extensions(
    obj: &JsObject,
    _context: &mut Context<'_>,
) -> JsResult<bool> {
    // 1. Set O.[[Extensible]] to false.
    obj.borrow_mut().extensible = false;

    // 2. Return true.
    Ok(true)
}

/// Abstract operation `OrdinaryGetOwnProperty`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarygetownproperty
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn ordinary_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    _context: &mut Context<'_>,
) -> JsResult<Option<PropertyDescriptor>> {
    let _timer = Profiler::global().start_event("Object::ordinary_get_own_property", "object");
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
    Ok(obj.borrow().properties.get(key))
}

/// Abstract operation `OrdinaryDefineOwnProperty`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinarydefineownproperty
pub(crate) fn ordinary_define_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let _timer = Profiler::global().start_event("Object::ordinary_define_own_property", "object");
    // 1. Let current be ? O.[[GetOwnProperty]](P).
    let current = obj.__get_own_property__(key, context)?;

    // 2. Let extensible be ? IsExtensible(O).
    let extensible = obj.__is_extensible__(context)?;

    // 3. Return ValidateAndApplyPropertyDescriptor(O, P, extensible, Desc, current).
    Ok(validate_and_apply_property_descriptor(
        Some((obj, key)),
        extensible,
        desc,
        current,
    ))
}

/// Abstract operation `OrdinaryHasProperty`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinaryhasproperty
pub(crate) fn ordinary_has_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let _timer = Profiler::global().start_event("Object::ordinary_has_property", "object");
    // 1. Assert: IsPropertyKey(P) is true.
    // 2. Let hasOwn be ? O.[[GetOwnProperty]](P).
    // 3. If hasOwn is not undefined, return true.
    if obj.__get_own_property__(key, context)?.is_some() {
        Ok(true)
    } else {
        // 4. Let parent be ? O.[[GetPrototypeOf]]().
        let parent = obj.__get_prototype_of__(context)?;

        parent
            // 5. If parent is not null, then
            // a. Return ? parent.[[HasProperty]](P).
            // 6. Return false.
            .map_or(Ok(false), |obj| obj.__has_property__(key, context))
    }
}

/// Abstract operation `OrdinaryGet`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinaryget
pub(crate) fn ordinary_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut Context<'_>,
) -> JsResult<JsValue> {
    let _timer = Profiler::global().start_event("Object::ordinary_get", "object");
    // 1. Assert: IsPropertyKey(P) is true.
    // 2. Let desc be ? O.[[GetOwnProperty]](P).
    match obj.__get_own_property__(key, context)? {
        // If desc is undefined, then
        None => {
            // a. Let parent be ? O.[[GetPrototypeOf]]().
            if let Some(parent) = obj.__get_prototype_of__(context)? {
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
pub(crate) fn ordinary_set(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let _timer = Profiler::global().start_event("Object::ordinary_set", "object");

    // 1. Assert: IsPropertyKey(P) is true.
    // 2. Let ownDesc be ? O.[[GetOwnProperty]](P).
    // 3. Return OrdinarySetWithOwnDescriptor(O, P, V, Receiver, ownDesc).

    // OrdinarySetWithOwnDescriptor ( O, P, V, Receiver, ownDesc )
    // https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-ordinarysetwithowndescriptor

    // 1. Assert: IsPropertyKey(P) is true.
    let own_desc = if let Some(desc) = obj.__get_own_property__(&key, context)? {
        desc
    }
    // 2. If ownDesc is undefined, then
    // a. Let parent be ? O.[[GetPrototypeOf]]().
    // b. If parent is not null, then
    else if let Some(parent) = obj.__get_prototype_of__(context)? {
        // i. Return ? parent.[[Set]](P, V, Receiver).
        return parent.__set__(key, value, receiver, context);
    }
    // c. Else,
    else {
        // i. Set ownDesc to the PropertyDescriptor { [[Value]]: undefined, [[Writable]]: true,
        // [[Enumerable]]: true, [[Configurable]]: true }.
        PropertyDescriptor::builder()
            .value(JsValue::undefined())
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

        // b. If Type(Receiver) is not Object, return false.
        let Some(receiver) = receiver.as_object() else {
            return Ok(false);
        };

        // c. Let existingDescriptor be ? Receiver.[[GetOwnProperty]](P).
        // d. If existingDescriptor is not undefined, then
        if let Some(ref existing_desc) = receiver.__get_own_property__(&key, context)? {
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
            return receiver.__define_own_property__(
                &key,
                PropertyDescriptor::builder().value(value).build(),
                context,
            );
        }
        // e. Else
        // i. Assert: Receiver does not currently have a property P.
        // ii. Return ? CreateDataProperty(Receiver, P, V).
        return receiver.create_data_property(key, value, context);
    }

    // 4. Assert: IsAccessorDescriptor(ownDesc) is true.
    debug_assert!(own_desc.is_accessor_descriptor());

    // 5. Let setter be ownDesc.[[Set]].
    match own_desc.set() {
        Some(set) if !set.is_undefined() => {
            // 7. Perform ? Call(setter, Receiver, « V »).
            set.call(&receiver, &[value], context)?;

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
pub(crate) fn ordinary_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context<'_>,
) -> JsResult<bool> {
    let _timer = Profiler::global().start_event("Object::ordinary_delete", "object");
    // 1. Assert: IsPropertyKey(P) is true.
    Ok(
        // 2. Let desc be ? O.[[GetOwnProperty]](P).
        match obj.__get_own_property__(key, context)? {
            // 4. If desc.[[Configurable]] is true, then
            Some(desc) if desc.expect_configurable() => {
                // a. Remove the own property with name P from O.
                obj.borrow_mut().remove(key);
                // b. Return true.
                true
            }
            // 5. Return false.
            Some(_) => false,
            // 3. If desc is undefined, return true.
            None => true,
        },
    )
}

/// Abstract operation `OrdinaryOwnPropertyKeys`.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinaryownpropertykeys
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn ordinary_own_property_keys(
    obj: &JsObject,
    _context: &mut Context<'_>,
) -> JsResult<Vec<PropertyKey>> {
    let _timer = Profiler::global().start_event("Object::ordinary_own_property_keys", "object");
    // 1. Let keys be a new empty List.
    let mut keys = Vec::new();

    let ordered_indexes = {
        let mut indexes: Vec<_> = obj.borrow().properties.index_property_keys().collect();
        indexes.sort_unstable();
        indexes
    };

    // 2. For each own property key P of O such that P is an array index, in ascending numeric index order, do
    // a. Add P as the last element of keys.
    keys.extend(ordered_indexes.into_iter().map(Into::into));

    // 3. For each own property key P of O such that Type(P) is String and P is not an array index, in ascending chronological order of property creation, do
    //     a. Add P as the last element of keys.
    //
    // 4. For each own property key P of O such that Type(P) is Symbol, in ascending chronological order of property creation, do
    //     a. Add P as the last element of keys.
    keys.extend(obj.borrow().properties.shape.keys());

    // 5. Return keys.
    Ok(keys)
}

/// Abstract operation `IsCompatiblePropertyDescriptor`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-iscompatiblepropertydescriptor
pub(crate) fn is_compatible_property_descriptor(
    extensible: bool,
    desc: PropertyDescriptor,
    current: Option<PropertyDescriptor>,
) -> bool {
    let _timer =
        Profiler::global().start_event("Object::is_compatible_property_descriptor", "object");

    // 1. Return ValidateAndApplyPropertyDescriptor(undefined, undefined, Extensible, Desc, Current).
    validate_and_apply_property_descriptor(None, extensible, desc, current)
}

/// Abstract operation `ValidateAndApplyPropertyDescriptor`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-validateandapplypropertydescriptor
pub(crate) fn validate_and_apply_property_descriptor(
    obj_and_key: Option<(&JsObject, &PropertyKey)>,
    extensible: bool,
    desc: PropertyDescriptor,
    current: Option<PropertyDescriptor>,
) -> bool {
    let _timer =
        Profiler::global().start_event("Object::validate_and_apply_property_descriptor", "object");
    // 1. Assert: If O is not undefined, then IsPropertyKey(P) is true.

    let Some(mut current) = current else {
        // 2. If current is undefined, then
        // a. If extensible is false, return false.
        if !extensible {
            return false;
        }

        // b. Assert: extensible is true.

        if let Some((obj, key)) = obj_and_key {
            obj.borrow_mut().properties.insert(
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
        }

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

        if obj_and_key.is_some() {
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
    if let Some((obj, key)) = obj_and_key {
        // a. For each field of Desc that is present, set the corresponding attribute of the
        // property named P of object O to the value of the field.
        current.fill_with(&desc);
        obj.borrow_mut().properties.insert(key, current);
    }

    // 10. Return true.
    true
}

/// Abstract operation `GetPrototypeFromConstructor`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-getprototypefromconstructor
#[track_caller]
pub(crate) fn get_prototype_from_constructor<F>(
    constructor: &JsValue,
    default: F,
    context: &mut Context<'_>,
) -> JsResult<JsObject>
where
    F: FnOnce(&StandardConstructors) -> &StandardConstructor,
{
    let _timer = Profiler::global().start_event("Object::get_prototype_from_constructor", "object");
    // 1. Assert: intrinsicDefaultProto is this specification's name of an intrinsic
    // object.
    // The corresponding object must be an intrinsic that is intended to be used
    // as the [[Prototype]] value of an object.
    // 2. Let proto be ? Get(constructor, "prototype").
    let realm = if let Some(constructor) = constructor.as_object() {
        if let Some(proto) = constructor.get(PROTOTYPE, context)?.as_object() {
            return Ok(proto.clone());
        }
        // 3. If Type(proto) is not Object, then
        // a. Let realm be ? GetFunctionRealm(constructor).
        constructor.get_function_realm(context)?
    } else {
        context.realm().clone()
    };
    // b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
    Ok(default(realm.intrinsics().constructors()).prototype())
}
