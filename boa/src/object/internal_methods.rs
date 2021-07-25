//! This module defines the object internal methods.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots

use crate::{
    object::{GcObject, Object, ObjectData},
    property::{AccessorDescriptor, Attribute, DataDescriptor, PropertyDescriptor, PropertyKey},
    value::{Type, Value},
    BoaProfiler, Context, Result,
};

impl GcObject {
    /// Check if object has property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasproperty
    // NOTE: for now context is not used but it will in the future.
    #[inline]
    pub fn has_property<K>(&self, key: K, _context: &mut Context) -> Result<bool>
    where
        K: Into<PropertyKey>,
    {
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Return ? O.[[HasProperty]](P).
        Ok(self.__has_property__(&key.into()))
    }

    /// Check if it is extensible.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isextensible-o
    // NOTE: for now context is not used but it will in the future.
    #[inline]
    pub fn is_extensible(&self, _context: &mut Context) -> Result<bool> {
        // 1. Assert: Type(O) is Object.
        // 2. Return ? O.[[IsExtensible]]().
        Ok(self.__is_extensible__())
    }

    /// Delete property, if deleted return `true`.
    #[inline]
    pub fn delete<K>(&self, key: K) -> bool
    where
        K: Into<PropertyKey>,
    {
        self.__delete__(&key.into())
    }

    /// Defines the property or throws a `TypeError` if the operation fails.
    ///
    /// More information:
    /// - [EcmaScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-definepropertyorthrow
    #[inline]
    pub fn delete_property_or_throw<K>(&self, key: K, context: &mut Context) -> Result<bool>
    where
        K: Into<PropertyKey>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let success be ? O.[[Delete]](P).
        let success = self.__delete__(&key);
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
    pub fn has_own_property<K>(&self, key: K, _context: &mut Context) -> Result<bool>
    where
        K: Into<PropertyKey>,
    {
        let key = key.into();
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let desc be ? O.[[GetOwnProperty]](P).
        let desc = self.__get_own_property__(&key);
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
    pub fn get<K>(&self, key: K, context: &mut Context) -> Result<Value>
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
    pub fn set<K, V>(&self, key: K, value: V, throw: bool, context: &mut Context) -> Result<bool>
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
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
        &mut self,
        key: K,
        desc: P,
        context: &mut Context,
    ) -> Result<bool>
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
    ) -> Result<bool>
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        // 1. Assert: Type(O) is Object.
        // 2. Assert: IsPropertyKey(P) is true.
        // 3. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true }.
        let new_desc = DataDescriptor::new(
            value,
            Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
        );
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
    ) -> Result<bool>
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
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
    pub(crate) fn __has_property__(&self, key: &PropertyKey) -> bool {
        let prop = self.__get_own_property__(key);
        if prop.is_none() {
            let parent = self.__get_prototype_of__();
            return if let Value::Object(ref object) = parent {
                object.__has_property__(key)
            } else {
                false
            };
        }
        true
    }

    /// Check if it is extensible.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible
    #[inline]
    pub(crate) fn __is_extensible__(&self) -> bool {
        self.borrow().extensible
    }

    /// Disable extensibility.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
    #[inline]
    pub fn __prevent_extensions__(&mut self) -> bool {
        self.borrow_mut().extensible = false;
        true
    }

    /// Delete property.
    #[inline]
    pub(crate) fn __delete__(&self, key: &PropertyKey) -> bool {
        match self.__get_own_property__(key) {
            Some(desc) if desc.configurable() => {
                self.remove(key);
                true
            }
            Some(_) => false,
            None => true,
        }
    }

    /// `[[Get]]`
    pub fn __get__(
        &self,
        key: &PropertyKey,
        receiver: Value,
        context: &mut Context,
    ) -> Result<Value> {
        match self.__get_own_property__(key) {
            None => {
                // parent will either be null or an Object
                if let Some(parent) = self.__get_prototype_of__().as_object() {
                    Ok(parent.__get__(key, receiver, context)?)
                } else {
                    Ok(Value::undefined())
                }
            }
            Some(ref desc) => match desc {
                PropertyDescriptor::Data(desc) => Ok(desc.value()),
                PropertyDescriptor::Accessor(AccessorDescriptor { get: Some(get), .. }) => {
                    get.call(&receiver, &[], context)
                }
                _ => Ok(Value::undefined()),
            },
        }
    }

    /// `[[Set]]`
    pub fn __set__(
        &self,
        key: PropertyKey,
        value: Value,
        receiver: Value,
        context: &mut Context,
    ) -> Result<bool> {
        let _timer = BoaProfiler::global().start_event("Object::set", "object");

        // Fetch property key
        let own_desc = if let Some(desc) = self.__get_own_property__(&key) {
            desc
        } else if let Some(ref mut parent) = self.__get_prototype_of__().as_object() {
            return parent.__set__(key, value, receiver, context);
        } else {
            DataDescriptor::new(Value::undefined(), Attribute::all()).into()
        };

        match &own_desc {
            PropertyDescriptor::Data(desc) => {
                if !desc.writable() {
                    return Ok(false);
                }
                if let Some(ref mut receiver) = receiver.as_object() {
                    if let Some(ref existing_desc) = receiver.__get_own_property__(&key) {
                        match existing_desc {
                            PropertyDescriptor::Accessor(_) => Ok(false),
                            PropertyDescriptor::Data(existing_data_desc) => {
                                if !existing_data_desc.writable() {
                                    return Ok(false);
                                }
                                receiver.__define_own_property__(
                                    key,
                                    DataDescriptor::new(value, existing_data_desc.attributes())
                                        .into(),
                                    context,
                                )
                            }
                        }
                    } else {
                        receiver.__define_own_property__(
                            key,
                            DataDescriptor::new(value, Attribute::all()).into(),
                            context,
                        )
                    }
                } else {
                    Ok(false)
                }
            }
            PropertyDescriptor::Accessor(AccessorDescriptor { set: Some(set), .. }) => {
                set.call(&receiver, &[value], context)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// `[[defineOwnProperty]]`
    pub fn __define_own_property__(
        &self,
        key: PropertyKey,
        desc: PropertyDescriptor,
        context: &mut Context,
    ) -> Result<bool> {
        if self.is_array() {
            self.array_define_own_property(key, desc, context)
        } else {
            Ok(self.ordinary_define_own_property(key, desc))
        }
    }

    /// Define an own property for an ordinary object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinarydefineownproperty
    pub fn ordinary_define_own_property(&self, key: PropertyKey, desc: PropertyDescriptor) -> bool {
        let _timer = BoaProfiler::global().start_event("Object::define_own_property", "object");

        let extensible = self.__is_extensible__();

        let current = if let Some(desc) = self.__get_own_property__(&key) {
            desc
        } else {
            if !extensible {
                return false;
            }

            self.insert(key, desc);

            return true;
        };

        // 4
        if !current.configurable() {
            if desc.configurable() {
                return false;
            }

            if desc.enumerable() != current.enumerable() {
                return false;
            }
        }

        match (&current, &desc) {
            (
                PropertyDescriptor::Data(current),
                PropertyDescriptor::Accessor(AccessorDescriptor { get, set, .. }),
            ) => {
                // 6. b
                if !current.configurable() {
                    return false;
                }

                let current =
                    AccessorDescriptor::new(get.clone(), set.clone(), current.attributes());
                self.insert(key, current);
                return true;
            }
            (
                PropertyDescriptor::Accessor(current),
                PropertyDescriptor::Data(DataDescriptor { value, .. }),
            ) => {
                // 6. c
                if !current.configurable() {
                    return false;
                }

                self.insert(key, DataDescriptor::new(value, current.attributes()));

                return true;
            }
            (PropertyDescriptor::Data(current), PropertyDescriptor::Data(desc)) => {
                // 7.
                if !current.configurable() && !current.writable() {
                    if desc.writable() {
                        return false;
                    }

                    if !Value::same_value(&desc.value(), &current.value()) {
                        return false;
                    }
                }
            }
            (PropertyDescriptor::Accessor(current), PropertyDescriptor::Accessor(desc)) => {
                // 8.
                if !current.configurable() {
                    if let (Some(current_get), Some(desc_get)) = (current.getter(), desc.getter()) {
                        if !GcObject::equals(current_get, desc_get) {
                            return false;
                        }
                    }

                    if let (Some(current_set), Some(desc_set)) = (current.setter(), desc.setter()) {
                        if !GcObject::equals(current_set, desc_set) {
                            return false;
                        }
                    }
                }
            }
        }

        match (&current, &desc) {
            (PropertyDescriptor::Data(current_data), PropertyDescriptor::Data(desc_data)) => {
                if desc_data.has_value() {
                    self.insert(key, desc);
                } else {
                    self.insert(
                        key,
                        DataDescriptor::new(current_data.value.clone(), desc_data.attributes()),
                    );
                }
            }
            _ => {
                self.insert(key, desc);
            }
        }

        true
    }

    /// Define an own property for an array.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-array-exotic-objects-defineownproperty-p-desc
    fn array_define_own_property(
        &self,
        key: PropertyKey,
        desc: PropertyDescriptor,
        context: &mut Context,
    ) -> Result<bool> {
        match key {
            PropertyKey::String(ref s) if s == "length" => {
                match desc {
                    PropertyDescriptor::Accessor(_) => {
                        return Ok(self.ordinary_define_own_property("length".into(), desc))
                    }
                    PropertyDescriptor::Data(ref d) => {
                        if d.value().is_undefined() {
                            return Ok(self.ordinary_define_own_property("length".into(), desc));
                        }
                        let new_len = d.value().to_u32(context)?;
                        let number_len = d.value().to_number(context)?;
                        #[allow(clippy::float_cmp)]
                        if new_len as f64 != number_len {
                            return Err(context.construct_range_error("bad length for array"));
                        }
                        let mut new_len_desc =
                            PropertyDescriptor::Data(DataDescriptor::new(new_len, d.attributes()));
                        let old_len_desc = self.__get_own_property__(&"length".into()).unwrap();
                        let old_len_desc = old_len_desc.as_data_descriptor().unwrap();
                        let old_len = old_len_desc.value();
                        if new_len >= old_len.to_u32(context)? {
                            return Ok(
                                self.ordinary_define_own_property("length".into(), new_len_desc)
                            );
                        }
                        if !old_len_desc.writable() {
                            return Ok(false);
                        }
                        let new_writable = if new_len_desc.attributes().writable() {
                            true
                        } else {
                            let mut new_attributes = new_len_desc.attributes();
                            new_attributes.set_writable(true);
                            new_len_desc = PropertyDescriptor::Data(DataDescriptor::new(
                                new_len,
                                new_attributes,
                            ));
                            false
                        };
                        if !self.ordinary_define_own_property("length".into(), new_len_desc.clone())
                        {
                            return Ok(false);
                        }
                        let keys_to_delete = {
                            let obj = self.borrow();
                            let mut keys = obj
                                .index_property_keys()
                                .filter(|&&k| k >= new_len)
                                .cloned()
                                .collect::<Vec<_>>();
                            keys.sort_unstable();
                            keys
                        };
                        for key in keys_to_delete.into_iter().rev() {
                            if !self.__delete__(&key.into()) {
                                let mut new_len_desc_attribute = new_len_desc.attributes();
                                if !new_writable {
                                    new_len_desc_attribute.set_writable(false);
                                }
                                let new_len_desc = PropertyDescriptor::Data(DataDescriptor::new(
                                    key + 1,
                                    new_len_desc_attribute,
                                ));
                                self.ordinary_define_own_property("length".into(), new_len_desc);
                                return Ok(false);
                            }
                        }
                        if !new_writable {
                            let mut new_desc_attr = new_len_desc.attributes();
                            new_desc_attr.set_writable(false);
                            let new_desc = PropertyDescriptor::Data(DataDescriptor::new(
                                new_len,
                                new_desc_attr,
                            ));
                            self.ordinary_define_own_property("length".into(), new_desc);
                        }
                    }
                }
                Ok(true)
            }
            PropertyKey::Index(index) => {
                let old_len_desc = self.__get_own_property__(&"length".into()).unwrap();
                let old_len_data_desc = old_len_desc.as_data_descriptor().unwrap();
                let old_len = old_len_data_desc.value().to_u32(context)?;
                if index >= old_len && !old_len_data_desc.writable() {
                    return Ok(false);
                }
                if self.ordinary_define_own_property(key, desc) {
                    if index >= old_len && index < u32::MAX {
                        let desc = PropertyDescriptor::Data(DataDescriptor::new(
                            index + 1,
                            old_len_data_desc.attributes(),
                        ));
                        self.ordinary_define_own_property("length".into(), desc);
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(self.ordinary_define_own_property(key, desc)),
        }
    }

    /// Gets own property of 'Object'
    ///
    #[inline]
    pub fn __get_own_property__(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        let _timer = BoaProfiler::global().start_event("Object::get_own_property", "object");

        let object = self.borrow();
        match object.data {
            ObjectData::String(_) => self.string_exotic_get_own_property(key),
            _ => self.ordinary_get_own_property(key),
        }
    }

    /// StringGetOwnProperty abstract operation
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringgetownproperty
    #[inline]
    pub fn string_get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        let object = self.borrow();

        match key {
            PropertyKey::Index(index) => {
                let string = object.as_string().unwrap();
                let pos = *index as usize;

                if pos >= string.len() {
                    return None;
                }

                let result_str = string.encode_utf16().nth(pos).map(|utf16_val| {
                    char::from_u32(u32::from(utf16_val))
                        .map_or_else(|| Value::from(format!("\\u{:x}", utf16_val)), Value::from)
                })?;

                let desc = PropertyDescriptor::from(DataDescriptor::new(
                    result_str,
                    Attribute::READONLY | Attribute::ENUMERABLE | Attribute::PERMANENT,
                ));

                Some(desc)
            }
            _ => None,
        }
    }

    /// Gets own property of 'String' exotic object
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-string-exotic-objects-getownproperty-p
    #[inline]
    pub fn string_exotic_get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        let desc = self.ordinary_get_own_property(key);

        if desc.is_some() {
            desc
        } else {
            self.string_get_own_property(key)
        }
    }

    /// The specification returns a Property Descriptor or Undefined.
    ///
    /// These are 2 separate types and we can't do that here.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
    #[inline]
    pub fn ordinary_get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        let object = self.borrow();
        let property = match key {
            PropertyKey::Index(index) => object.indexed_properties.get(index),
            PropertyKey::String(ref st) => object.string_properties.get(st),
            PropertyKey::Symbol(ref symbol) => object.symbol_properties.get(symbol),
        };

        property.cloned()
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
        self.borrow().keys().collect()
    }

    /// The abstract operation ObjectDefineProperties
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.defineproperties
    #[inline]
    pub fn define_properties(&mut self, props: Value, context: &mut Context) -> Result<()> {
        let props = &props.to_object(context)?;
        let keys = props.own_property_keys();
        let mut descriptors: Vec<(PropertyKey, PropertyDescriptor)> = Vec::new();

        for next_key in keys {
            if let Some(prop_desc) = props.__get_own_property__(&next_key) {
                if prop_desc.enumerable() {
                    let desc_obj = props.__get__(&next_key, props.clone().into(), context)?;
                    let desc = desc_obj.to_property_descriptor(context)?;
                    descriptors.push((next_key, desc));
                }
            }
        }

        for (p, d) in descriptors {
            self.__define_own_property__(p, d, context)?;
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
    pub fn __set_prototype_of__(&mut self, val: Value) -> bool {
        debug_assert!(val.is_object() || val.is_null());
        let current = self.__get_prototype_of__();
        if Value::same_value(&current, &val) {
            return true;
        }
        if !self.__is_extensible__() {
            return false;
        }
        let mut p = val.clone();
        let mut done = false;
        while !done {
            if p.is_null() {
                done = true
            } else if Value::same_value(&Value::from(self.clone()), &p) {
                return false;
            } else {
                let prototype = p
                    .as_object()
                    .expect("prototype should be null or object")
                    .__get_prototype_of__();
                p = prototype;
            }
        }
        self.set_prototype_instance(val);
        true
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
    pub fn __get_prototype_of__(&self) -> Value {
        self.borrow().prototype.clone()
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
    pub fn insert_property<K, V>(
        &self,
        key: K,
        value: V,
        attribute: Attribute,
    ) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        self.insert(key.into(), DataDescriptor::new(value, attribute))
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

    /// Returns true if the GcObject is the global for a Realm
    pub fn is_global(&self) -> bool {
        matches!(self.borrow().data, ObjectData::Global)
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
    ) -> Result<Vec<Value>> {
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

    pub(crate) fn length_of_array_like(&self, context: &mut Context) -> Result<usize> {
        // 1. Assert: Type(obj) is Object.
        // 2. Return ℝ(? ToLength(? Get(obj, "length"))).
        self.get("length", context)?.to_length(context)
    }
}

impl Object {
    /// Helper function for property insertion.
    #[inline]
    pub(crate) fn insert<K, P>(&mut self, key: K, property: P) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        let property = property.into();
        match key.into() {
            PropertyKey::Index(index) => self.indexed_properties.insert(index, property),
            PropertyKey::String(ref string) => {
                self.string_properties.insert(string.clone(), property)
            }
            PropertyKey::Symbol(ref symbol) => {
                self.symbol_properties.insert(symbol.clone(), property)
            }
        }
    }

    /// Helper function for property removal.
    #[inline]
    pub(crate) fn remove(&mut self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        match key {
            PropertyKey::Index(index) => self.indexed_properties.remove(index),
            PropertyKey::String(ref string) => self.string_properties.remove(string),
            PropertyKey::Symbol(ref symbol) => self.symbol_properties.remove(symbol),
        }
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name that a `Some` is returned
    /// with that field, otherwise None is retuned.
    #[inline]
    pub fn insert_property<K, V>(
        &mut self,
        key: K,
        value: V,
        attribute: Attribute,
    ) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        V: Into<Value>,
    {
        self.insert(key.into(), DataDescriptor::new(value, attribute))
    }
}
