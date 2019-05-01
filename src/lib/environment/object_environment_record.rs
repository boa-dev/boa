//! # Object Records
//!
//! Each object Environment Record is associated with an object called its binding object.
//! An object Environment Record binds the set of string identifier names that directly
//! correspond to the property names of its binding object.
//! Property keys that are not strings in the form of an IdentifierName are not included in the set of bound identifiers.
//! More info:  [Object Records](https://tc39.github.io/ecma262/#sec-object-environment-records)

use crate::environment::lexical_environment::Environment;
use crate::js::object::Property;
use crate::js::value::{Value, ValueData};
use gc::Gc;

#[derive(Trace, Finalize, Debug, Clone)]
pub struct ObjectEnvironmentRecord {
    pub bindings: Value,
    pub with_environment: bool,
    pub outer_env: Option<Environment>,
}

impl ObjectEnvironmentRecord {
    pub fn has_binding(&self, name: &String) -> bool {
        if !self.bindings.has_field(name.to_string()) {
            return false;
        }
        if !self.with_environment {
            return true;
        }

        // TODO: implement unscopables
        true
    }

    pub fn create_mutable_binding(&mut self, name: String, deletion: bool) {
        // TODO: could save time here and not bother generating a new undefined object,
        // only for it to be replace with the real value later. We could just add the name to a Vector instead
        let bindings = &mut self.bindings;
        let uninitialized = Gc::new(ValueData::Undefined);
        let mut prop = Property::new(uninitialized);
        prop.enumerable = true;
        prop.writable = true;
        prop.configurable = deletion;
        bindings.set_prop(name, prop);
    }

    pub fn create_immutable_binding(&mut self, _name: String, _strict: bool) {
        unimplemented!()
    }

    pub fn initialize_binding(&mut self, name: String, value: Value) {
        // We should never need to check if a binding has been created,
        // As all calls to create_mutable_binding are followed by initialized binding
        // The below is just a check.
        debug_assert!(self.has_binding(&name));
        return self.set_mutable_binding(name, value, false);
    }

    pub fn set_mutable_binding(&mut self, name: String, value: Value, strict: bool) {
        debug_assert!(value.is_object() || value.is_function());

        let bindings = &mut self.bindings;
        bindings.update_prop(name, Some(value.clone()), None, None, Some(strict));
    }

    pub fn get_binding_value(&self, name: String, strict: bool) -> Value {
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

    pub fn delete_binding(&mut self, name: String) -> bool {
        self.bindings.remove_prop(&name);
        true
    }

    pub fn has_this_binding(&self) -> bool {
        false
    }

    pub fn get_this_binding(&self) -> Option<Value> {
        None
    }

    pub fn has_super_binding(&self) -> bool {
        false
    }

    pub fn with_base_object(&self) -> Value {
        // Object Environment Records return undefined as their
        // WithBaseObject unless their withEnvironment flag is true.
        if self.with_environment {
            return self.bindings.clone();
        }

        Gc::new(ValueData::Undefined)
    }

    pub fn get_outer_environment(&self) -> Option<&Environment> {
        match &self.outer_env {
            Some(outer) => Some(&outer),
            None => None,
        }
    }

    pub fn set_outer_environment(&mut self, env: Environment) {
        self.outer_env = Some(env);
    }
}
