//! This module defines the object internal methods.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots

use crate::{
    object::Object,
    property::{Attribute, DataDescriptor, PropertyDescriptor, PropertyKey},
    value::{same_value, Value},
    BoaProfiler, Context, Result,
};

impl Object {
    /// Check if object has property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
    pub fn has_property(&self, key: &PropertyKey) -> bool {
        let prop = self.get_own_property(key);
        if prop.is_none() {
            let parent = self.get_prototype_of();
            return if let Value::Object(ref object) = parent {
                object.borrow().has_property(key)
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
        self.extensible
    }

    /// Disable extensibility.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
    #[inline]
    pub fn prevent_extensions(&mut self) -> bool {
        self.extensible = false;
        true
    }

    /// Delete property.
    pub fn delete(&mut self, key: &PropertyKey) -> bool {
        match self.get_own_property(key) {
            Some(desc) if desc.configurable() => {
                self.remove_property(&key);
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
            Some(desc) => {
                if desc.is_data_descriptor() {
                    return desc.value.clone().expect("failed to extract value");
                }

                let getter = desc.get.clone();
                if getter.is_none() || getter.expect("Failed to get object").is_undefined() {
                    return Value::undefined();
                }

                // TODO: Call getter from here!
                Value::undefined()
            }
        }
    }

    /// [[Set]]
    /// <https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver>
    pub fn set(&mut self, key: PropertyKey, val: Value) -> bool {
        let _timer = BoaProfiler::global().start_event("Object::set", "object");

        // Fetch property key
        let mut own_desc = if let Some(desc) = self.get_own_property(&key) {
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

        // [3]
        if own_desc.is_data_descriptor() {
            if !own_desc.writable() {
                return false;
            }

            // Change value on the current descriptor
            own_desc.value = Some(val);
            return self.define_own_property(key, own_desc);
        }
        // [4]
        debug_assert!(own_desc.is_accessor_descriptor());
        match own_desc.set {
            None => false,
            Some(_) => {
                unimplemented!();
            }
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

        let mut current = if let Some(desc) = self.get_own_property(&key) {
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

        // 5
        if desc.is_generic_descriptor() {
            // 6
        } else if current.is_data_descriptor() != desc.is_data_descriptor() {
            // a
            if !current.configurable() {
                return false;
            }
            // b
            if current.is_data_descriptor() {
                // Convert to accessor
                current.value = None;
                current.attribute.remove(Attribute::WRITABLE);
            } else {
                // c
                // convert to data
                current.get = None;
                current.set = None;
            }

            self.insert(key, current);
            return true;
        // 7
        } else if current.is_data_descriptor() && desc.is_data_descriptor() {
            // a
            if !current.configurable() && !current.writable() {
                if desc.writable() {
                    return false;
                }

                if desc.value.is_some()
                    && !same_value(
                        &desc.value.clone().unwrap(),
                        &current.value.clone().unwrap(),
                    )
                {
                    return false;
                }

                return true;
            }
        // 8
        } else {
            if !current.configurable() {
                if desc.set.is_some()
                    && !same_value(&desc.set.clone().unwrap(), &current.set.clone().unwrap())
                {
                    return false;
                }

                if desc.get.is_some()
                    && !same_value(&desc.get.clone().unwrap(), &current.get.clone().unwrap())
                {
                    return false;
                }
            }

            return true;
        }
        // 9
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
    pub fn get_own_property(&self, key: &PropertyKey) -> Option<PropertyDescriptor> {
        let _timer = BoaProfiler::global().start_event("Object::get_own_property", "object");

        let property = match key {
            PropertyKey::Index(index) => self.indexed_properties.get(&index),
            PropertyKey::String(ref st) => self.string_properties.get(st),
            PropertyKey::Symbol(ref symbol) => self.symbol_properties.get(symbol),
        };

        property.cloned()
    }

    /// Essential internal method OwnPropertyKeys
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec](https://tc39.es/ecma262/#table-essential-internal-methods)
    pub fn own_property_keys(&self) -> Vec<PropertyKey> {
        self.keys().collect()
    }

    /// The abstract operation ObjectDefineProperties
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object.defineproperties
    pub fn define_properties(&mut self, props: Value, ctx: &mut Context) -> Result<()> {
        let props = props.to_object(ctx)?;
        let keys = props.borrow().own_property_keys();
        let mut descriptors: Vec<(PropertyKey, PropertyDescriptor)> = Vec::new();

        for next_key in keys {
            if let Some(prop_desc) = props.borrow().get_own_property(&next_key) {
                if prop_desc.enumerable() {
                    let desc_obj = props.borrow().get(&next_key);
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
        self.prototype.clone()
    }

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
    pub(crate) fn remove_property(&mut self, key: &PropertyKey) -> Option<PropertyDescriptor> {
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
