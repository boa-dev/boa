//! This module defines the `ObjectInternalMethods` trait.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots

use crate::builtins::{
    object::{Object, INSTANCE_PROTOTYPE},
    property::Property,
    value::{to_value, Value, ValueData},
};
use gc::Gc;
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
                    ValueData::Object(ref obj) => obj.borrow().has_property(val),
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
        self.set_internal_slot("extensible", to_value(false));
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
                return Gc::new(ValueData::Undefined);
            }

            let parent_obj = Object::from(&parent).expect("Failed to get object");

            return parent_obj.get(val);
        }

        if desc.is_data_descriptor() {
            return desc.value.clone().expect("failed to extract value");
        }

        let getter = desc.get.clone();
        if getter.is_none() || getter.expect("Failed to get object").is_undefined() {
            return Gc::new(ValueData::Undefined);
        }

        // TODO!!!!! Call getter from here
        Gc::new(ValueData::Undefined)
    }

    /// [[Set]]
    /// <https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver>
    fn set(&mut self, field: Value, val: Value) -> bool {
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

    fn define_own_property(&mut self, property_key: String, desc: Property) -> bool;

    /// Utility function to get an immutable internal slot or Null
    fn get_internal_slot(&self, name: &str) -> Value;

    fn set_internal_slot(&mut self, name: &str, val: Value);

    fn insert_property(&mut self, name: String, p: Property);

    fn remove_property(&mut self, name: &str);
}
