//! This module defines the object internal methods.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots

use crate::{
    builtins::Array,
    object::{JsObject, Object, ObjectData, ObjectKind},
    property::{DescriptorKind, PropertyDescriptor, PropertyKey, PropertyNameKind},
    value::{JsValue, Type},
    BoaProfiler, Context, JsResult,
};

pub(crate) mod array;
pub(crate) mod string;

impl JsObject {
    /// Check if object has property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasproperty
    // NOTE: for now context is not used but it will in the future.
    #[inline]
    pub fn has_property<K>(&self, key: K, context: &mut Context) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
    {
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Return ? O.[[HasProperty]](P).
        self.__has_property__(&key.into(), context)
    }

    /// Check if it is extensible.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isextensible-o
    #[inline]
    pub fn is_extensible(&self, context: &mut Context) -> JsResult<bool> {
        // 1. Assert: Type(O) is Object.
        // 2. Return ? O.[[IsExtensible]]().
        self.__is_extensible__(context)
    }

    /// Defines the property or throws a `TypeError` if the operation fails.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-definepropertyorthrow
    #[inline]
    pub fn delete_property_or_throw<K>(&self, key: K, context: &mut Context) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let success be ? O.[[Delete]](P).
        let success = self.__delete__(&key, context)?;
        // 4. If success is false, throw a TypeError exception.
        if !success {
            return Err(context.construct_type_error(format!("cannot delete property: {}", key)));
        }
        // 5. Return success.
        Ok(success)
    }

    /// Check if object has an own property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasownproperty
    #[inline]
    pub fn has_own_property<K>(&self, key: K, context: &mut Context) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let desc be ? O.[[GetOwnProperty]](P).
        let desc = self.__get_own_property__(&key, context)?;
        // 4. If desc is undefined, return false.
        // 5. Return true.
        Ok(desc.is_some())
    }

    /// Get property from object or throw.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-o-p
    #[inline]
    pub fn get<K>(&self, key: K, context: &mut Context) -> JsResult<JsValue>
    where
        K: Into<PropertyKey>,
    {
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Return ? O.[[Get]](P, O).
        self.__get__(&key.into(), self.clone().into(), context)
    }

    /// set property of object or throw if bool flag is passed.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-set-o-p-v-throw
    #[inline]
    pub fn set<K, V>(&self, key: K, value: V, throw: bool, context: &mut Context) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Assert: Type(Throw) is Boolean.
        // 4. Let success be ? O.[[Set]](P, V, O).
        let success = self.__set__(key.clone(), value.into(), self.clone().into(), context)?;
        // 5. If success is false and Throw is true, throw a TypeError exception.
        if !success && throw {
            return Err(
                context.construct_type_error(format!("cannot set non-writable property: {}", key))
            );
        }
        // 6. Return success.
        Ok(success)
    }

    /// Define property or throw.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-definepropertyorthrow
    #[inline]
    pub fn define_property_or_throw<K, P>(
        &self,
        key: K,
        desc: P,
        context: &mut Context,
    ) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let success be ? O.[[DefineOwnProperty]](P, desc).
        let success = self.__define_own_property__(key.clone(), desc.into(), context)?;
        // 4. If success is false, throw a TypeError exception.
        if !success {
            return Err(context.construct_type_error(format!("cannot redefine property: {}", key)));
        }
        // 5. Return success.
        Ok(success)
    }

    /// Create data property
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-deletepropertyorthrow
    pub fn create_data_property<K, V>(
        &self,
        key: K,
        value: V,
        context: &mut Context,
    ) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true }.
        let new_desc = PropertyDescriptor::builder()
            .value(value)
            .writable(true)
            .enumerable(true)
            .configurable(true);
        // 4. Return ? O.[[DefineOwnProperty]](P, newDesc).
        self.__define_own_property__(key.into(), new_desc.into(), context)
    }

    /// Create data property or throw
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-deletepropertyorthrow
    pub fn create_data_property_or_throw<K, V>(
        &self,
        key: K,
        value: V,
        context: &mut Context,
    ) -> JsResult<bool>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let success be ? CreateDataProperty(O, P, V).
        let success = self.create_data_property(key.clone(), value, context)?;
        // 4. If success is false, throw a TypeError exception.
        if !success {
            return Err(context.construct_type_error(format!("cannot redefine property: {}", key)));
        }
        // 5. Return success.
        Ok(success)
    }

    /// `[[hasProperty]]`
    #[inline]
    pub(crate) fn __has_property__(
        &self,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__has_property__;
        func(self, key, context)
    }

    /// Check if it is extensible.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible
    #[inline]
    pub(crate) fn __is_extensible__(&self, context: &mut Context) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__is_extensible__;
        func(self, context)
    }

    /// Disable extensibility.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
    #[inline]
    pub fn __prevent_extensions__(&mut self, context: &mut Context) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__prevent_extensions__;
        func(self, context)
    }

    /// Delete property.
    #[inline]
    pub(crate) fn __delete__(&self, key: &PropertyKey, context: &mut Context) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__delete__;
        func(self, key, context)
    }

    /// `[[Get]]`
    pub fn __get__(
        &self,
        key: &PropertyKey,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let func = self.borrow().data.internal_methods.__get__;
        func(self, key, receiver, context)
    }

    /// `[[Set]]`
    pub fn __set__(
        &self,
        key: PropertyKey,
        value: JsValue,
        receiver: JsValue,
        context: &mut Context,
    ) -> JsResult<bool> {
        let _timer = BoaProfiler::global().start_event("Object::set", "object");
        let func = self.borrow().data.internal_methods.__set__;
        func(self, key, value, receiver, context)
    }

    /// `[[defineOwnProperty]]`
    pub fn __define_own_property__(
        &self,
        key: PropertyKey,
        desc: PropertyDescriptor,
        context: &mut Context,
    ) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__define_own_property__;
        func(self, key, desc, context)
    }

    /// Gets own property of 'Object'
    ///
    #[inline]
    pub fn __get_own_property__(
        &self,
        key: &PropertyKey,
        context: &mut Context,
    ) -> JsResult<Option<PropertyDescriptor>> {
        let _timer = BoaProfiler::global().start_event("Object::get_own_property", "object");
        let func = self.borrow().data.internal_methods.__get_own_property__;
        func(self, key, context)
    }

    /// Essential internal method OwnPropertyKeys
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#table-essential-internal-methods
    #[inline]
    #[track_caller]
    pub fn own_property_keys(&self) -> Vec<PropertyKey> {
        self.borrow().properties.keys().collect()
    }

    /// The abstract operation ObjectDefineProperties
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.defineproperties
    #[inline]
    pub fn define_properties(&mut self, props: JsValue, context: &mut Context) -> JsResult<()> {
        let props = &props.to_object(context)?;
        let keys = props.own_property_keys();
        let mut descriptors: Vec<(PropertyKey, PropertyDescriptor)> = Vec::new();

        for next_key in keys {
            if let Some(prop_desc) = props.__get_own_property__(&next_key, context)? {
                if prop_desc.expect_enumerable() {
                    let desc_obj = props.get(next_key.clone(), context)?;
                    let desc = desc_obj.to_property_descriptor(context)?;
                    descriptors.push((next_key, desc));
                }
            }
        }

        for (p, d) in descriptors {
            self.define_property_or_throw(p, d, context)?;
        }

        Ok(())
    }

    /// `Object.setPropertyOf(obj, prototype)`
    ///
    /// This method sets the prototype (i.e., the internal `[[Prototype]]` property)
    /// of a specified object to another object or `null`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/setPrototypeOf
    #[inline]
    pub fn __set_prototype_of__(&mut self, val: JsValue, context: &mut Context) -> JsResult<bool> {
        let func = self.borrow().data.internal_methods.__set_prototype_of__;
        func(self, val, context)
    }

    /// Returns either the prototype or null
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getPrototypeOf
    #[inline]
    #[track_caller]
    pub fn __get_prototype_of__(&self, context: &mut Context) -> JsResult<JsValue> {
        let func = self.borrow().data.internal_methods.__get_prototype_of__;
        func(self, context)
    }

    /// Helper function for property insertion.
    #[inline]
    #[track_caller]
    pub(crate) fn insert<K, P>(&self, key: K, property: P) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.borrow_mut().insert(key, property)
    }

    /// Helper function for property removal.
    #[inline]
    #[track_caller]
    pub(crate) fn remove(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        self.borrow_mut().remove(key)
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name that a `Some` is returned
    /// with that field, otherwise None is returned.
    #[inline]
    pub fn insert_property<K, P>(&self, key: K, property: P) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.insert(key.into(), property)
    }

    /// It determines if Object is a callable function with a `[[Call]]` internal method.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-iscallable
    #[inline]
    #[track_caller]
    pub fn is_callable(&self) -> bool {
        self.borrow().is_callable()
    }

    /// It determines if Object is a function object with a `[[Construct]]` internal method.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isconstructor
    #[inline]
    #[track_caller]
    pub fn is_constructable(&self) -> bool {
        self.borrow().is_constructable()
    }

    /// Returns true if the JsObject is the global for a Realm
    pub fn is_global(&self) -> bool {
        matches!(
            self.borrow().data,
            ObjectData {
                kind: ObjectKind::Global,
                ..
            }
        )
    }

    /// It is used to create List value whose elements are provided by the indexed properties of
    /// self.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createlistfromarraylike
    pub(crate) fn create_list_from_array_like(
        &self,
        element_types: &[Type],
        context: &mut Context,
    ) -> JsResult<Vec<JsValue>> {
        // 1. If elementTypes is not present, set elementTypes to « Undefined, Null, Boolean, String, Symbol, Number, BigInt, Object ».
        let types = if element_types.is_empty() {
            &[
                Type::Undefined,
                Type::Null,
                Type::Boolean,
                Type::String,
                Type::Symbol,
                Type::Number,
                Type::BigInt,
                Type::Object,
            ]
        } else {
            element_types
        };

        // TODO: 2. If Type(obj) is not Object, throw a TypeError exception.

        // 3. Let len be ? LengthOfArrayLike(obj).
        let len = self.length_of_array_like(context)?;

        // 4. Let list be a new empty List.
        let mut list = Vec::with_capacity(len);

        // 5. Let index be 0.
        // 6. Repeat, while index < len,
        for index in 0..len {
            // a. Let indexName be ! ToString(𝔽(index)).
            // b. Let next be ? Get(obj, indexName).
            let next = self.get(index, context)?;
            // c. If Type(next) is not an element of elementTypes, throw a TypeError exception.
            if !types.contains(&next.get_type()) {
                return Err(context.construct_type_error("bad type"));
            }
            // d. Append next as the last element of list.
            list.push(next.clone());
            // e. Set index to index + 1.
        }

        // 7. Return list.
        Ok(list)
    }

    /// It is used to iterate over names of object's keys.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-enumerableownpropertynames
    pub(crate) fn enumerable_own_property_names(
        &self,
        kind: PropertyNameKind,
        context: &mut Context,
    ) -> JsResult<Vec<JsValue>> {
        // 1. Assert: Type(O) is Object.
        // 2. Let ownKeys be ? O.[[OwnPropertyKeys]]().
        let own_keys = self.own_property_keys();
        // 3. Let properties be a new empty List.
        let mut properties = vec![];

        // 4. For each element key of ownKeys, do
        for key in own_keys {
            // a. If Type(key) is String, then
            let key_str = match &key {
                PropertyKey::String(s) => Some(s.clone()),
                PropertyKey::Index(i) => Some(i.to_string().into()),
                _ => None,
            };

            if let Some(key_str) = key_str {
                // i. Let desc be ? O.[[GetOwnProperty]](key).
                let desc = self.__get_own_property__(&key, context)?;
                // ii. If desc is not undefined and desc.[[Enumerable]] is true, then
                if let Some(desc) = desc {
                    if desc.expect_enumerable() {
                        match kind {
                            // 1. If kind is key, append key to properties.
                            PropertyNameKind::Key => properties.push(key_str.into()),
                            // 2. Else,
                            // a. Let value be ? Get(O, key).
                            // b. If kind is value, append value to properties.
                            PropertyNameKind::Value => {
                                properties.push(self.get(key.clone(), context)?)
                            }
                            // c. Else,
                            // i. Assert: kind is key+value.
                            // ii. Let entry be ! CreateArrayFromList(« key, value »).
                            // iii. Append entry to properties.
                            PropertyNameKind::KeyAndValue => properties.push(
                                Array::create_array_from_list(
                                    [key_str.into(), self.get(key.clone(), context)?],
                                    context,
                                )
                                .into(),
                            ),
                        }
                    }
                }
            }
        }

        // 5. Return properties.
        Ok(properties)
    }

    pub(crate) fn length_of_array_like(&self, context: &mut Context) -> JsResult<usize> {
        // 1. Assert: Type(obj) is Object.
        // 2. Return ℝ(? ToLength(? Get(obj, "length"))).
        self.get("length", context)?.to_length(context)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct InternalObjectMethods {
    pub(crate) __get_prototype_of__: fn(&JsObject, &mut Context) -> JsResult<JsValue>,
    pub(crate) __set_prototype_of__: fn(&JsObject, JsValue, &mut Context) -> JsResult<bool>,
    pub(crate) __is_extensible__: fn(&JsObject, &mut Context) -> JsResult<bool>,
    pub(crate) __prevent_extensions__: fn(&JsObject, &mut Context) -> JsResult<bool>,
    pub(crate) __get_own_property__:
        fn(&JsObject, &PropertyKey, &mut Context) -> JsResult<Option<PropertyDescriptor>>,
    pub(crate) __define_own_property__:
        fn(&JsObject, PropertyKey, PropertyDescriptor, &mut Context) -> JsResult<bool>,
    pub(crate) __has_property__: fn(&JsObject, &PropertyKey, &mut Context) -> JsResult<bool>,
    pub(crate) __get__: fn(&JsObject, &PropertyKey, JsValue, &mut Context) -> JsResult<JsValue>,
    pub(crate) __set__:
        fn(&JsObject, PropertyKey, JsValue, JsValue, &mut Context) -> JsResult<bool>,
    pub(crate) __delete__: fn(&JsObject, &PropertyKey, &mut Context) -> JsResult<bool>,
    pub(crate) __own_property_keys__: fn(&JsObject, &mut Context) -> JsResult<Vec<PropertyKey>>,
}

impl Default for InternalObjectMethods {
    fn default() -> Self {
        Self {
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
        }
    }
}

/// Returns either the prototype or null
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getPrototypeOf
#[inline]
pub(crate) fn ordinary_get_prototype_of(
    obj: &JsObject,
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(obj.borrow().prototype.clone())
}

/// `Object.setPropertyOf(obj, prototype)`
///
/// This method sets the prototype (i.e., the internal `[[Prototype]]` property)
/// of a specified object to another object or `null`.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/setPrototypeOf
#[inline]
pub(crate) fn ordinary_set_prototype_of(
    obj: &JsObject,
    val: JsValue,
    context: &mut Context,
) -> JsResult<bool> {
    debug_assert!(val.is_object() || val.is_null());
    let current = obj.__get_prototype_of__(context)?;
    if JsValue::same_value(&current, &val) {
        return Ok(true);
    }
    if !obj.__is_extensible__(context)? {
        return Ok(false);
    }
    let mut p = val.clone();
    let mut done = false;
    while !done {
        match p {
            JsValue::Null => done = true,
            JsValue::Object(ref proto) => {
                if JsObject::equals(proto, obj) {
                    return Ok(false);
                } else if proto.borrow().data.internal_methods.__get_prototype_of__ as usize
                    != ordinary_get_prototype_of as usize
                {
                    done = true;
                } else {
                    p = proto.__get_prototype_of__(context)?;
                }
            }
            _ => unreachable!(),
        }
    }
    obj.borrow_mut().prototype = val;
    Ok(true)
}

/// Check if it is extensible.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible
// NOTE: for now context is not used but it will in the future.
#[inline]
pub(crate) fn ordinary_is_extensible(obj: &JsObject, _context: &mut Context) -> JsResult<bool> {
    Ok(obj.borrow().extensible)
}

/// Disable extensibility.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
#[inline]
pub(crate) fn ordinary_prevent_extensions(
    obj: &JsObject,
    _context: &mut Context,
) -> JsResult<bool> {
    obj.borrow_mut().extensible = false;
    Ok(true)
}

/// Get property of object without checking its prototype.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
#[inline]
pub(crate) fn ordinary_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    _context: &mut Context,
) -> JsResult<Option<PropertyDescriptor>> {
    Ok(obj.borrow().properties.get(key).cloned())
}

/// Define property of object.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc
#[inline]
pub(crate) fn ordinary_define_own_property(
    obj: &JsObject,
    key: PropertyKey,
    desc: PropertyDescriptor,
    context: &mut Context,
) -> JsResult<bool> {
    let extensible = obj.__is_extensible__(context)?;

    let mut current = if let Some(own) = obj.__get_own_property__(&key, context)? {
        own
    } else {
        if !extensible {
            return Ok(false);
        }

        obj.borrow_mut().properties.insert(
            key,
            if desc.is_generic_descriptor() || desc.is_data_descriptor() {
                desc.into_data_defaulted()
            } else {
                desc.into_accessor_defaulted()
            },
        );

        return Ok(true);
    };

    // 3
    if desc.is_empty() {
        return Ok(true);
    }

    // 4
    if !current.expect_configurable() {
        if matches!(desc.configurable(), Some(true)) {
            return Ok(false);
        }

        if matches!(desc.enumerable(), Some(desc_enum) if desc_enum != current.expect_enumerable())
        {
            return Ok(false);
        }
    }

    // 5
    if desc.is_generic_descriptor() {
        // no further validation required
    } else if current.is_data_descriptor() != desc.is_data_descriptor() {
        if !current.expect_configurable() {
            return Ok(false);
        }
        if current.is_data_descriptor() {
            current = current.into_accessor_defaulted();
        } else {
            current = current.into_data_defaulted();
        }
    } else if current.is_data_descriptor() && desc.is_data_descriptor() {
        if !current.expect_configurable() && !current.expect_writable() {
            if matches!(desc.writable(), Some(true)) {
                return Ok(false);
            }
            if matches!(desc.value(), Some(value) if !JsValue::same_value(value, current.expect_value()))
            {
                return Ok(false);
            }
            return Ok(true);
        }
    } else if !current.expect_configurable() {
        if matches!(desc.set(), Some(set) if !JsValue::same_value(set, current.expect_set())) {
            return Ok(false);
        }
        if matches!(desc.get(), Some(get) if !JsValue::same_value(get, current.expect_get())) {
            return Ok(false);
        }
        return Ok(true);
    }

    current.fill_with(desc);
    obj.borrow_mut().properties.insert(key, current);

    Ok(true)
}

// Check if object has property.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
#[inline]
pub(crate) fn ordinary_has_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context,
) -> JsResult<bool> {
    let prop = obj.__get_own_property__(key, context)?;
    if prop.is_none() {
        let parent = obj.__get_prototype_of__(context)?;
        return if let JsValue::Object(ref object) = parent {
            object.__has_property__(key, context)
        } else {
            Ok(false)
        };
    }
    Ok(true)
}

/// `OrdinaryGet`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver
#[inline]
pub(crate) fn ordinary_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut Context,
) -> JsResult<JsValue> {
    match obj.__get_own_property__(key, context)? {
        None => {
            // parent will either be null or an Object
            if let Some(parent) = obj.__get_prototype_of__(context)?.as_object() {
                parent.__get__(key, receiver, context)
            } else {
                Ok(JsValue::undefined())
            }
        }
        Some(ref desc) => match desc.kind() {
            DescriptorKind::Data {
                value: Some(value), ..
            } => Ok(value.clone()),
            DescriptorKind::Accessor { get: Some(get), .. } if !get.is_undefined() => {
                context.call(get, &receiver, &[])
            }
            _ => Ok(JsValue::undefined()),
        },
    }
}

/// `[[Set]]`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver
#[inline]
pub(crate) fn ordinary_set(
    obj: &JsObject,
    key: PropertyKey,
    value: JsValue,
    receiver: JsValue,
    context: &mut Context,
) -> JsResult<bool> {
    // Fetch property key
    let own_desc = if let Some(desc) = obj.__get_own_property__(&key, context)? {
        desc
    } else if let Some(ref mut parent) = obj.__get_prototype_of__(context)?.as_object() {
        return parent.__set__(key, value, receiver, context);
    } else {
        PropertyDescriptor::builder()
            .value(JsValue::undefined())
            .writable(true)
            .enumerable(true)
            .configurable(true)
            .build()
    };

    if own_desc.is_data_descriptor() {
        if !own_desc.expect_writable() {
            return Ok(false);
        }

        let receiver = match receiver.as_object() {
            Some(obj) => obj,
            _ => return Ok(false),
        };

        if let Some(ref existing_desc) = receiver.__get_own_property__(&key, context)? {
            if existing_desc.is_accessor_descriptor() {
                return Ok(false);
            }
            if !existing_desc.expect_writable() {
                return Ok(false);
            }
            return receiver.__define_own_property__(
                key,
                PropertyDescriptor::builder().value(value).build(),
                context,
            );
        } else {
            return receiver.create_data_property(key, value, context);
        }
    }

    match own_desc.set() {
        Some(set) if !set.is_undefined() => {
            context.call(set, &receiver, &[value])?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Delete property.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-delete-p
#[inline]
pub(crate) fn ordinary_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut Context,
) -> JsResult<bool> {
    Ok(match obj.__get_own_property__(key, context)? {
        Some(desc) if desc.expect_configurable() => {
            obj.borrow_mut().remove(key);
            true
        }
        Some(_) => false,
        None => true,
    })
}

/// Essential internal method `[[OwnPropertyKeys]]`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-ownpropertykeys
#[inline]
pub(crate) fn ordinary_own_property_keys(
    obj: &JsObject,
    _context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    Ok(obj.borrow().properties.keys().collect())
}

impl Object {
    /// Helper function for property insertion.
    #[inline]
    pub(crate) fn insert<K, P>(&mut self, key: K, property: P) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.properties.insert(key.into(), property.into())
    }

    /// Helper function for property removal.
    #[inline]
    pub(crate) fn remove(&mut self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        self.properties.remove(key)
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name that a `Some` is returned
    /// with that field, otherwise None is retuned.
    #[inline]
    pub fn insert_property<K, P>(&mut self, key: K, property: P) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.insert(key, property)
    }
}
