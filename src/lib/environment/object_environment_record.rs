//! # Object Records
//!
//! Each object Environment Record is associated with an object called its binding object.
//! An object Environment Record binds the set of string identifier names that directly
//! correspond to the property names of its binding object.
//! Property keys that are not strings in the form of an IdentifierName are not included in the set of bound identifiers.
//! More info:  [Object Records](https://tc39.github.io/ecma262/#sec-object-environment-records)

use crate::environment::environment_record::EnvironmentRecordTrait;
use crate::js::object::Property;
use crate::js::value::{Value, ValueData};
use gc::Gc;

pub struct ObjectEnvironmentRecord {
    bindings: Value,
    with_environment: bool,
    outer_env: Box<EnvironmentRecordTrait>,
}

impl EnvironmentRecordTrait for ObjectEnvironmentRecord {
    fn has_binding(&self, name: &String) -> bool {
        if !self.bindings.has_field(name.to_string()) {
            return false;
        }
        if !self.with_environment {
            return true;
        }

        // TODO: implement unscopables
        true
    }

    fn create_mutable_binding(&mut self, name: String, deletion: bool) {
        // TODO: could save time here and not bother generating a new undefined object,
        // only for it to be replace with the real value later. We could just add the name to a Vector instead
        let bindings = self.bindings;
        let uninitialized = Gc::new(ValueData::Undefined);
        let prop = Property::new(uninitialized);
        prop.enumerable = true;
        prop.writable = true;
        prop.configurable = deletion;
        bindings.set_prop(name, prop);
    }

    fn create_immutable_binding(&mut self, name: String, strict: bool) {}

    fn initialize_binding(&mut self, name: String, value: Value) {
        // We should never need to check if a binding has been created,
        // As all calls to create_mutable_binding are followed by initialized binding
        // The below is just a check.
        debug_assert!(self.has_binding(&name));
        return self.set_mutable_binding(name, value, false);
    }

    fn set_mutable_binding(&mut self, name: String, value: Value, strict: bool) {
        debug_assert!(value.is_object() || value.is_function());
        let result = value.update_prop(name, Some(value), None, None, None);
        // We should check something has been set on something otherwise its a bug
        debug_assert!(result.is_some());
    }

    fn get_binding_value(&self, name: String, strict: bool) -> Value {
        if self.bindings.has_field(name.clone()) {
            return self.bindings.get_field(name);
        }

        if !strict {
            return Gc::new(ValueData::Undefined);
        }

        // TODO: throw error here
        // Error handling not implemented yet
        Gc::new(ValueData::Undefined)
    }

    fn delete_binding(&mut self, name: String) -> bool {
        self.bindings.remove_prop(&name);
        true
    }

    fn has_this_binding(&self) -> bool {
        false
    }

    fn has_super_binding(&self) -> bool {
        false
    }

    fn with_base_object(&self) -> Value {
        // Object Environment Records return undefined as their
        // WithBaseObject unless their withEnvironment flag is true.
        if self.with_environment {
            return self.bindings;
        }

        Gc::new(ValueData::Undefined)
    }

    fn get_outer_environment(&self) -> Option<&Box<EnvironmentRecordTrait>> {
        Some(&self.outer_env)
    }
}
