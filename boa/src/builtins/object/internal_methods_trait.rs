//! This module defines the `ObjectInternalMethods` trait.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots

use crate::{
    builtins::{
        object::{Object, INSTANCE_PROTOTYPE},
        property::Property,
        value::{same_value, Value, ValueData},
    },
    BoaProfiler,
};
use std::borrow::Borrow;
use std::ops::Deref;

/// Here lies the internal methods for ordinary objects.
///
/// Most objects make use of these methods, including exotic objects like functions.
/// So thats why this is a trait
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots
pub trait ObjectInternalMethods {
    /// Check if has property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
    fn has_property(&self, val: &Value) -> bool {
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
    fn is_extensible(&self) -> bool {
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
    fn prevent_extensions(&mut self) -> bool {
        self.set_internal_slot("extensible", Value::from(false));
        true
    }

    /// Delete property.
    fn delete(&mut self, prop_key: &Value) -> bool {
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
    fn get(&self, val: &Value) -> Value {
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

        // TODO!!!!! Call getter from here
        Value::undefined()
    }

    /// [[Set]]
    /// <https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver>
    fn set(&mut self, field: Value, val: Value) -> bool {
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

    fn define_own_property(&mut self, property_key: String, desc: Property) -> bool {
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

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
    /// The specification returns a Property Descriptor or Undefined. These are 2 separate types and we can't do that here.
    fn get_own_property(&self, prop: &Value) -> Property;

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
    fn set_prototype_of(&mut self, val: Value) -> bool;

    /// Returns either the prototype or null
    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
    fn get_prototype_of(&self) -> Value {
        self.get_internal_slot(INSTANCE_PROTOTYPE)
    }

    /// Utility function to get an immutable internal slot or Null
    fn get_internal_slot(&self, name: &str) -> Value;

    fn set_internal_slot(&mut self, name: &str, val: Value);

    fn insert_property(&mut self, name: String, p: Property);

    fn remove_property(&mut self, name: &str);
}
