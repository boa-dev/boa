//! This module defines the object internal methods.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots

use crate::builtins::{
    object::{Object, INSTANCE_PROTOTYPE, PROTOTYPE},
    property::Property,
    value::{same_value, Value, ValueData},
};
use crate::BoaProfiler;
use std::borrow::Borrow;
use std::ops::Deref;

impl Object {
    /// Check if object has property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
    pub fn has_property(&self, val: &Value) -> bool {
        debug_assert!(Property::is_property_key(val));
        let prop = self.get_own_property(val);
        if prop.value.is_none() {
            let parent: Value = self.get_prototype_of();
            if !parent.is_null() {
                // the parent value variant should be an object
                // In the unlikely event it isn't return false
                return match *parent {
                    ValueData::Object(ref obj) => (*obj).deref().borrow().has_property(val),
                    _ => false,
                };
            }
            return false;
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
        let val = self.get_internal_slot("extensible");
        match *val.deref().borrow() {
            ValueData::Boolean(b) => b,
            _ => false,
        }
    }

    /// Disable extensibility.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
    #[inline]
    pub fn prevent_extensions(&mut self) -> bool {
        self.set_internal_slot("extensible", Value::from(false));
        true
    }

    /// Delete property.
    pub fn delete(&mut self, prop_key: &Value) -> bool {
        debug_assert!(Property::is_property_key(prop_key));
        let desc = self.get_own_property(prop_key);
        if desc
            .value
            .clone()
            .expect("unable to get value")
            .is_undefined()
        {
            return true;
        }
        if desc.configurable.expect("unable to get value") {
            self.remove_property(&prop_key.to_string());
            return true;
        }

        false
    }

    // [[Get]]
    pub fn get(&self, val: &Value) -> Value {
        debug_assert!(Property::is_property_key(val));
        let desc = self.get_own_property(val);
        if desc.value.clone().is_none()
            || desc
                .value
                .clone()
                .expect("Failed to get object")
                .is_undefined()
        {
            // parent will either be null or an Object
            let parent = self.get_prototype_of();
            if parent.is_null() {
                return Value::undefined();
            }

            let parent_obj = Object::from(&parent).expect("Failed to get object");

            return parent_obj.get(val);
        }

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

    /// [[Set]]
    /// <https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver>
    pub fn set(&mut self, field: Value, val: Value) -> bool {
        let _timer = BoaProfiler::global().start_event("Object::set", "object");
        // [1]
        debug_assert!(Property::is_property_key(&field));

        // Fetch property key
        let mut own_desc = self.get_own_property(&field);
        // [2]
        if own_desc.is_none() {
            let parent = self.get_prototype_of();
            if !parent.is_null() {
                // TODO: come back to this
            }
            own_desc = Property::new()
                .writable(true)
                .enumerable(true)
                .configurable(true);
        }
        // [3]
        if own_desc.is_data_descriptor() {
            if !own_desc.writable.unwrap() {
                return false;
            }

            // Change value on the current descriptor
            own_desc = own_desc.value(val);
            return self.define_own_property(field.to_string(), own_desc);
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
    pub fn define_own_property(&mut self, property_key: String, desc: Property) -> bool {
        let _timer = BoaProfiler::global().start_event("Object::define_own_property", "object");

        let mut current = self.get_own_property(&Value::from(property_key.to_string()));
        let extensible = self.is_extensible();

        // https://tc39.es/ecma262/#sec-validateandapplypropertydescriptor
        // There currently isn't a property, lets create a new one
        if current.value.is_none() || current.value.as_ref().expect("failed").is_undefined() {
            if !extensible {
                return false;
            }

            self.insert_property(property_key, desc);
            return true;
        }
        // If every field is absent we don't need to set anything
        if desc.is_none() {
            return true;
        }

        // 4
        if !current.configurable.unwrap_or(false) {
            if desc.configurable.is_some() && desc.configurable.expect("unable to get prop desc") {
                return false;
            }

            if desc.enumerable.is_some()
                && (desc.enumerable.as_ref().expect("unable to get prop desc")
                    != current
                        .enumerable
                        .as_ref()
                        .expect("unable to get prop desc"))
            {
                return false;
            }
        }

        // 5
        if desc.is_generic_descriptor() {
            // 6
        } else if current.is_data_descriptor() != desc.is_data_descriptor() {
            // a
            if !current.configurable.expect("unable to get prop desc") {
                return false;
            }
            // b
            if current.is_data_descriptor() {
                // Convert to accessor
                current.value = None;
                current.writable = None;
            } else {
                // c
                // convert to data
                current.get = None;
                current.set = None;
            }

            self.insert_property(property_key.clone(), current);
        // 7
        } else if current.is_data_descriptor() && desc.is_data_descriptor() {
            // a
            if !current.configurable.expect("unable to get prop desc")
                && !current.writable.expect("unable to get prop desc")
            {
                if desc.writable.is_some() && desc.writable.expect("unable to get prop desc") {
                    return false;
                }

                if desc.value.is_some()
                    && !same_value(
                        &desc.value.clone().unwrap(),
                        &current.value.clone().unwrap(),
                        false,
                    )
                {
                    return false;
                }

                return true;
            }
        // 8
        } else {
            if !current.configurable.unwrap() {
                if desc.set.is_some()
                    && !same_value(
                        &desc.set.clone().unwrap(),
                        &current.set.clone().unwrap(),
                        false,
                    )
                {
                    return false;
                }

                if desc.get.is_some()
                    && !same_value(
                        &desc.get.clone().unwrap(),
                        &current.get.clone().unwrap(),
                        false,
                    )
                {
                    return false;
                }
            }

            return true;
        }
        // 9
        self.insert_property(property_key, desc);
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
    pub fn get_own_property(&self, prop: &Value) -> Property {
        let _timer = BoaProfiler::global().start_event("Object::get_own_property", "object");

        debug_assert!(Property::is_property_key(prop));
        // Prop could either be a String or Symbol
        match *(*prop) {
            ValueData::String(ref st) => {
                match self.properties.get(st) {
                    // If O does not have an own property with key P, return undefined.
                    // In this case we return a new empty Property
                    None => Property::default(),
                    Some(ref v) => {
                        let mut d = Property::default();
                        if v.is_data_descriptor() {
                            d.value = v.value.clone();
                            d.writable = v.writable;
                        } else {
                            debug_assert!(v.is_accessor_descriptor());
                            d.get = v.get.clone();
                            d.set = v.set.clone();
                        }
                        d.enumerable = v.enumerable;
                        d.configurable = v.configurable;
                        d
                    }
                }
            }
            ValueData::Symbol(ref symbol) => {
                match self.symbol_properties().get(&symbol.hash()) {
                    // If O does not have an own property with key P, return undefined.
                    // In this case we return a new empty Property
                    None => Property::default(),
                    Some(ref v) => {
                        let mut d = Property::default();
                        if v.is_data_descriptor() {
                            d.value = v.value.clone();
                            d.writable = v.writable;
                        } else {
                            debug_assert!(v.is_accessor_descriptor());
                            d.get = v.get.clone();
                            d.set = v.set.clone();
                        }
                        d.enumerable = v.enumerable;
                        d.configurable = v.configurable;
                        d
                    }
                }
            }
            _ => Property::default(),
        }
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
    pub fn set_prototype_of(&mut self, val: Value) -> bool {
        debug_assert!(val.is_object() || val.is_null());
        let current = self.get_internal_slot(PROTOTYPE);
        if same_value(&current, &val, false) {
            return true;
        }
        let extensible = self.get_internal_slot("extensible");
        if extensible.is_null() {
            return false;
        }
        let mut p = val.clone();
        let mut done = false;
        while !done {
            if p.is_null() {
                done = true
            } else if same_value(&Value::from(self.clone()), &p, false) {
                return false;
            } else {
                p = p.get_internal_slot(PROTOTYPE);
            }
        }
        self.set_internal_slot(PROTOTYPE, val);
        true
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
        self.get_internal_slot(INSTANCE_PROTOTYPE)
    }

    /// Helper function to get an immutable internal slot or `Null`.
    #[inline]
    pub fn get_internal_slot(&self, name: &str) -> Value {
        let _timer = BoaProfiler::global().start_event("Object::get_internal_slot", "object");
        match self.internal_slots.get(name) {
            Some(v) => v.clone(),
            None => Value::null(),
        }
    }

    /// Helper function to set an internal slot.
    #[inline]
    pub fn set_internal_slot(&mut self, name: &str, val: Value) {
        self.internal_slots.insert(name.to_string(), val);
    }

    /// Helper function for property insertion.
    #[inline]
    pub fn insert_property<N>(&mut self, name: N, p: Property)
    where
        N: Into<String>,
    {
        self.properties.insert(name.into(), p);
    }

    /// Helper function for property removal.
    #[inline]
    pub fn remove_property(&mut self, name: &str) {
        self.properties.remove(name);
    }

    #[inline]
    pub fn insert_field<N>(&mut self, name: N, value: Value) -> Option<Property>
    where
        N: Into<String>,
    {
        self.properties.insert(
            name.into(),
            Property::default()
                .value(value)
                .writable(true)
                .configurable(true)
                .enumerable(true),
        )
    }

    #[inline]
    pub fn get_field(&self, name: &str) -> Option<&Value> {
        self.properties.get(name).and_then(|x| x.value.as_ref())
    }
}
