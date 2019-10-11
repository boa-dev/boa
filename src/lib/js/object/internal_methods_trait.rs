use crate::js::{
    object::{Object, PROTOTYPE},
    property::Property,
    value::{same_value, to_value, Value, ValueData},
};
use gc::Gc;
use std::borrow::Borrow;
use std::ops::Deref;

/// Here lies the internal methods for ordinary objects.   
/// Most objects can make use of these methods, including exotic objects like functions.   
/// So thats why this is in a trait
/// <https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots>
pub trait ObjectInternalMethods {
    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
    fn set_prototype_of(&mut self, val: Value) -> bool;

    /// Returns either the prototype or null
    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
    fn get_prototype_of(&self) -> Value {
        let val = self.get_internal_slot(PROTOTYPE);
        match *val.deref().borrow() {
            ValueData::Object(v) => val.clone(),
            _ => Gc::new(ValueData::Null),
        }
    }

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible
    fn is_extensible(&self) -> bool {
        let val = self.get_internal_slot("extensible");
        match *val.deref().borrow() {
            ValueData::Boolean(b) => b,
            _ => false,
        }
    }

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-preventextensions
    fn prevent_extensions(&mut self) -> bool {
        self.set_internal_slot("extensible", to_value(false));
        true
    }
    /// <https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p>
    /// The specification returns a Property Descriptor or Undefined. These are 2 separate types and we can't do that here.
    fn get_own_property(&self, prop: &Value) -> Property;

    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
    fn has_property(&self, val: &Value) -> bool;

    #[allow(clippy::option_unwrap_used)]
    fn define_own_property(&mut self, property_key: String, desc: Property) -> bool {
        let mut current = self.get_own_property(&to_value(property_key.to_string()));
        let extensible = self.is_extensible();

        // https://tc39.es/ecma262/#sec-validateandapplypropertydescriptor
        // There currently isn't a property, lets create a new one
        if current.value.is_none() || current.value.as_ref().expect("failed").is_undefined() {
            if !extensible {
                return false;
            }

            let mut p = Property::new();
            if desc.is_generic_descriptor() || desc.is_data_descriptor() {
                p.value = Some(desc.value.clone().unwrap_or_default());
                p.writable = Some(desc.writable.unwrap_or_default());
                p.configurable = Some(desc.configurable.unwrap_or_default());
                p.enumerable = Some(desc.enumerable.unwrap_or_default());
            } else {
                p.get = Some(desc.get.clone().unwrap_or_default());
                p.set = Some(desc.set.clone().unwrap_or_default());
                p.configurable = Some(desc.configurable.unwrap_or_default());
                p.enumerable = Some(desc.enumerable.unwrap_or_default());
            };
            self.insert_property(property_key, p);
            return true;
        }
        // If every field is absent we don't need to set anything
        if desc.is_none() {
            return true;
        }

        // 4
        if current.configurable.unwrap_or(false) {
            if desc.configurable.is_some() && desc.configurable.unwrap() {
                return false;
            }

            if desc.enumerable.is_some()
                && (desc.enumerable.as_ref().unwrap() == current.enumerable.as_ref().unwrap())
            {
                return false;
            }
        }

        // 5
        if desc.is_generic_descriptor() {

            // 6
        } else if current.is_data_descriptor() != desc.is_data_descriptor() {
            // a
            if !current.configurable.unwrap() {
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

            self.insert_property(property_key, current.clone());
        // 7
        } else if current.is_data_descriptor() && desc.is_data_descriptor() {
            // a
            if !current.configurable.unwrap() && !current.writable.unwrap() {
                if desc.writable.is_some() && desc.writable.unwrap() {
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
        Property::assign(&mut current, &desc);
        true
    }

    // [[Delete]]
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
            self.remove_property(prop_key.to_string());
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

    // Boa Extensions
    fn clone(&self) -> Self;

    fn get_internal_slot(&self, name: &str) -> Value;

    fn set_internal_slot(&mut self, name: &str, val: Value);

    fn insert_property(&self, name: String, p: Property);

    fn remove_property(&self, name: String);
}
