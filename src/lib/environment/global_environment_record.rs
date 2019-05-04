//! # Global Environment Records
//!
//! A global Environment Record is used to represent the outer most scope that is shared by all
//! of the ECMAScript Script elements that are processed in a common realm.
//! A global Environment Record provides the bindings for built-in globals (clause 18),
//! properties of the global object, and for all top-level declarations (13.2.8, 13.2.10)
//! that occur within a Script.
//! More info:  https://tc39.github.io/ecma262/#sec-global-environment-records

use crate::environment::declerative_environment_record::DeclerativeEnvironmentRecord;
use crate::environment::environment_record_trait::EnvironmentRecordTrait;
use crate::environment::lexical_environment::{Environment, EnvironmentError, EnvironmentType};
use crate::environment::object_environment_record::ObjectEnvironmentRecord;
use crate::js::value::{Value, ValueData};
use gc::Gc;
use std::collections::HashSet;
use std::convert::TryFrom;

#[derive(Trace, Finalize, Clone)]
pub struct GlobalEnvironmentRecord {
    pub object_record: Box<ObjectEnvironmentRecord>,
    pub global_this_binding: Value,
    pub declerative_record: Box<DeclerativeEnvironmentRecord>,
    pub var_names: HashSet<String>,
}

impl GlobalEnvironmentRecord {
    pub fn get_this_binding(&self) -> Value {
        return self.global_this_binding.clone();
    }

    pub fn has_var_decleration(&self, name: &String) -> bool {
        return self.var_names.contains(name);
    }

    pub fn has_lexical_decleration(&self, name: &String) -> bool {
        self.declerative_record.has_binding(name)
    }

    pub fn has_restricted_global_property(&self, name: &String) -> bool {
        let global_object = &self.object_record.bindings;
        let existing_prop = global_object.get_prop(name.clone());
        match existing_prop {
            Some(prop) => {
                if prop.value.is_undefined() || prop.configurable == true {
                    return false;
                }
                true
            }
            None => false,
        }
    }

    pub fn create_global_var_binding(&mut self, name: String, deletion: bool) {
        let obj_rec = &mut self.object_record;
        let global_object = &obj_rec.bindings;
        let has_property = global_object.has_field(name.clone());
        let extensible = global_object.is_extensible();
        if !has_property && extensible {
            obj_rec.create_mutable_binding(name.clone(), deletion);
            obj_rec.initialize_binding(name.clone(), Gc::new(ValueData::Undefined));
        }

        let var_declared_names = &mut self.var_names;
        if !var_declared_names.contains(&name) {
            var_declared_names.insert(name.clone());
        }
    }

    pub fn create_global_function_binding(&mut self, name: String, value: Value, deletion: bool) {
        let global_object = &mut self.object_record.bindings;
        let existing_prop = global_object.get_prop(name.clone());
        match existing_prop {
            Some(prop) => {
                if prop.value.is_undefined() || prop.configurable {
                    global_object.update_prop(
                        name,
                        Some(value),
                        Some(true),
                        Some(true),
                        Some(deletion),
                    );
                }
            }
            None => {
                global_object.update_prop(
                    name,
                    Some(value),
                    Some(true),
                    Some(true),
                    Some(deletion),
                );
            }
        }
    }
}

impl EnvironmentRecordTrait for GlobalEnvironmentRecord {
    fn has_binding(&self, name: &String) -> bool {
        if self.declerative_record.has_binding(name) {
            return true;
        }
        self.object_record.has_binding(name)
    }

    fn create_mutable_binding(&mut self, name: String, deletion: bool) {
        if self.declerative_record.has_binding(&name) {
            // TODO: change to exception
            panic!("Binding already exists!");
        }

        self.declerative_record
            .create_mutable_binding(name.clone(), deletion)
    }

    fn create_immutable_binding(&mut self, name: String, strict: bool) {
        if self.declerative_record.has_binding(&name) {
            // TODO: change to exception
            panic!("Binding already exists!");
        }
        self.declerative_record
            .create_immutable_binding(name.clone(), strict)
    }

    fn initialize_binding(&mut self, name: String, value: Value) {
        if self.declerative_record.has_binding(&name) {
            // TODO: assert binding is in the object environment record
            return self
                .declerative_record
                .initialize_binding(name.clone(), value);
        }
    }

    fn set_mutable_binding(&mut self, name: String, value: Value, strict: bool) {
        if self.declerative_record.has_binding(&name) {
            return self
                .declerative_record
                .set_mutable_binding(name, value, strict);
        }
        self.object_record.set_mutable_binding(name, value, strict)
    }

    fn get_binding_value(&self, name: String, strict: bool) -> Value {
        if self.declerative_record.has_binding(&name) {
            return self.declerative_record.get_binding_value(name, strict);
        }
        return self.object_record.get_binding_value(name, strict);
    }

    fn delete_binding(&mut self, name: String) -> bool {
        if self.declerative_record.has_binding(&name) {
            return self.declerative_record.delete_binding(name.clone());
        }

        let global: &Value = &self.object_record.bindings;
        if global.has_field(name.clone()) {
            let status = self.object_record.delete_binding(name.clone());
            if status {
                let var_names = &mut self.var_names;
                if var_names.contains(&name) {
                    var_names.remove(&name);
                    return status;
                }
            }
        }
        true
    }

    fn has_this_binding(&self) -> bool {
        true
    }

    fn has_super_binding(&self) -> bool {
        false
    }

    fn with_base_object(&self) -> Value {
        Gc::new(ValueData::Undefined)
    }

    fn get_outer_environment(&self) -> Option<Environment> {
        None
    }

    fn set_outer_environment(&mut self, env: Environment) {
        unimplemented!()
    }

    fn get_environment_type(&self) -> EnvironmentType {
        return EnvironmentType::Global;
    }

    fn get_global_object(&self) -> Option<Value> {
        Some(self.global_this_binding.clone())
    }
}
