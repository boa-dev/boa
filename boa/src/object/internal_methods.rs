//! This module defines the object internal methods.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots

use crate::{
    object::{GcObject, Object},
    property::{AccessorDescriptor, Attribute, DataDescriptor, PropertyDescriptor, PropertyKey},
    value::{same_value, Value},
    BoaProfiler, Context, Result,
};

impl GcObject {
    /// Check if object has property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
    #[inline]
    pub fn has_property(&self, key: &PropertyKey) -> bool {
        let prop = self.get_own_property(key);
        if prop.is_none() {
            let parent = self.get_prototype_of();
            return if let Value::Object(ref object) = parent {
                object.has_property(key)
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
    pub fn is_extensible(&self) -> bool {
        self.borrow().extensible
    }

    /// Disable extensibility.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
    #[inline]
    pub fn prevent_extensions(&mut self) -> bool {
        self.borrow_mut().extensible = false;
        true
    }

    /// Delete property.
    #[inline]
    pub fn delete(&mut self, key: &PropertyKey) -> bool {
        match self.get_own_property(key) {
            Some(desc) if desc.configurable() => {
                self.remove(&key);
                true
            }
            Some(_) => false,
            None => true,
        }
    }

    /// [[Get]]
    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver
    pub fn get(&self, key: &PropertyKey) -> Value {
        match self.get_own_property(key) {
            None => {
                // parent will either be null or an Object
                let parent = self.get_prototype_of();
                if parent.is_null() {
                    return Value::undefined();
                }

                parent.get_field(key.clone())
            }
            Some(ref desc) => match desc {
                PropertyDescriptor::Accessor(_) => todo!(),
                PropertyDescriptor::Data(desc) => desc.value(),
            },
        }
    }

    /// [[Set]]
    /// <https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver>
    pub fn set(&mut self, key: PropertyKey, val: Value) -> bool {
        let _timer = BoaProfiler::global().start_event("Object::set", "object");

        // Fetch property key
        let own_desc = if let Some(desc) = self.get_own_property(&key) {
            desc
        } else {
            let parent = self.get_prototype_of();
            if !parent.is_null() {
                // TODO: come back to this
            }
            DataDescriptor::new(
                Value::undefined(),
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .into()
        };

        match &own_desc {
            PropertyDescriptor::Data(desc) => {
                if !desc.writable() {
                    return false;
                }

                let desc = DataDescriptor::new(val, own_desc.attributes()).into();
                self.define_own_property(key, desc)
            }
            PropertyDescriptor::Accessor(_) => todo!(),
        }
    }

    /// Define an own property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc
    pub fn define_own_property<K>(&mut self, key: K, desc: PropertyDescriptor) -> bool
    where
        K: Into<PropertyKey>,
    {
        let _timer = BoaProfiler::global().start_event("Object::define_own_property", "object");

        let key = key.into();
        let extensible = self.is_extensible();

        let current = if let Some(desc) = self.get_own_property(&key) {
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
            (PropertyDescriptor::Data(current), PropertyDescriptor::Data(desc)) => {
                if !current.configurable() && !current.writable() {
                    if desc.writable() {
                        return false;
                    }

                    if !same_value(&desc.value(), &current.value()) {
                        return false;
                    }
                }
            }
            (PropertyDescriptor::Data(current), PropertyDescriptor::Accessor(_)) => {
                if !current.configurable() {
                    return false;
                }

                let current = AccessorDescriptor::new(None, None, current.attributes());
                self.insert(key, current);
                return true;
            }
            (PropertyDescriptor::Accessor(current), PropertyDescriptor::Data(_)) => {
                if !current.configurable() {
                    return false;
                }

                let current = DataDescriptor::new(Value::undefined(), current.attributes());
                self.insert(key, current);
                return true;
            }
            (PropertyDescriptor::Accessor(current), PropertyDescriptor::Accessor(desc)) => {
                if !current.configurable() {
                    if let (Some(current_get), Some(desc_get)) = (current.getter(), desc.getter()) {
                        if !GcObject::equals(&current_get, &desc_get) {
                            return false;
                        }
                    }

                    if let (Some(current_set), Some(desc_set)) = (current.setter(), desc.setter()) {
                        if !GcObject::equals(&current_set, &desc_set) {
                            return false;
                        }
                    }
                }
            }
        }

        self.insert(key, desc);
        true
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
    pub fn get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        let _timer = BoaProfiler::global().start_event("Object::get_own_property", "object");

        let object = self.borrow();
        let property = match key {
            PropertyKey::Index(index) => object.indexed_properties.get(&index),
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
    /// [spec](https://tc39.es/ecma262/#table-essential-internal-methods)
    #[inline]
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
    pub fn define_properties(&mut self, props: Value, ctx: &mut Context) -> Result<()> {
        let props = props.to_object(ctx)?;
        let keys = props.own_property_keys();
        let mut descriptors: Vec<(PropertyKey, PropertyDescriptor)> = Vec::new();

        for next_key in keys {
            if let Some(prop_desc) = props.get_own_property(&next_key) {
                if prop_desc.enumerable() {
                    let desc_obj = props.get(&next_key);
                    let desc = desc_obj.to_property_descriptor(ctx)?;
                    descriptors.push((next_key, desc));
                }
            }
        }

        descriptors.into_iter().for_each(|(p, d)| {
            self.define_own_property(p, d);
        });

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
    pub fn set_prototype_of(&mut self, _val: Value) -> bool {
        // debug_assert!(val.is_object() || val.is_null());
        // let current = self.prototype.clone();
        // if same_value(&current, &val) {
        //     return true;
        // }
        // if !self.is_extensible() {
        //     return false;
        // }
        // let mut p = val.clone();
        // let mut done = false;
        // while !done {
        //     if p.is_null() {
        //         done = true
        //     } else if same_value(&Value::from(self.clone()), &p) {
        //         return false;
        //     } else {
        //         let prototype = p
        //             .as_object()
        //             .expect("prototype should be null or object")
        //             .prototype
        //             .clone();
        //         p = prototype;
        //     }
        // }
        // self.prototype = val;
        // true
        todo!("Object.setPropertyOf(obj, prototype)")
    }

    /// Returns either the prototype or null
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
    #[inline]
    pub fn get_prototype_of(&self) -> Value {
        self.borrow().prototype.clone()
    }

    /// Helper function for property insertion.
    #[inline]
    pub(crate) fn insert<K, P>(&mut self, key: K, property: P) -> Option<PropertyDescriptor>
    where
        K: Into<PropertyKey>,
        P: Into<PropertyDescriptor>,
    {
        self.borrow_mut().insert(key, property)
    }

    /// Helper function for property removal.
    #[inline]
    pub(crate) fn remove(&mut self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        self.borrow_mut().remove(key)
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name that a `Some` is returned
    /// with that field, otherwise None is returned.
    #[inline]
    pub(crate) fn insert_property<K, V>(
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

    /// It determines if Object is a callable function with a [[Call]] internal method.
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

    /// It determines if Object is a function object with a [[Construct]] internal method.
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
            PropertyKey::Index(index) => self.indexed_properties.remove(&index),
            PropertyKey::String(ref string) => self.string_properties.remove(string),
            PropertyKey::Symbol(ref symbol) => self.symbol_properties.remove(symbol),
        }
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name that a `Some` is returned
    /// with that field, otherwise None is retuned.
    #[inline]
    pub(crate) fn insert_property<K, V>(
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
