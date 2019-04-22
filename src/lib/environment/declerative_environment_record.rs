//! # Declerative Records
//!
//! Each declarative Environment Record is associated with an ECMAScript program scope containing variable,
//! `constant`, `let`, `class`, `module`, `import`, and/or function declarations.
//! A declarative Environment Record binds the set of identifiers defined by the declarations contained within its scope.
//! More info:  [ECMA-262 sec-declarative-environment-records](https://tc39.github.io/ecma262/#sec-declarative-environment-records)

use crate::environment::environment_record::EnvironmentRecordTrait;
use crate::js::value::{Value, ValueData};
use gc::Gc;
use std::collections::hash_map::HashMap;

/// Declerative Bindings have a few properties for book keeping purposes, such as mutability (const vs let).
/// Can it be deleted? and strict mode.
///
/// So we need to create a struct to hold these values.
/// From this point onwards, a binding is referring to one of these structures.
pub struct DeclerativeEnvironmentRecordBinding {
    pub value: Option<Value>,
    pub can_delete: bool,
    pub mutable: bool,
    pub strict: bool,
}

/// A declarative Environment Record binds the set of identifiers defined by the
/// declarations contained within its scope.
pub struct DeclerativeEnvironmentRecord {
    pub env_rec: HashMap<String, DeclerativeEnvironmentRecordBinding>,
    pub outer_env: Option<Box<EnvironmentRecordTrait>>,
}

impl EnvironmentRecordTrait for DeclerativeEnvironmentRecord {
    fn has_binding(&self, name: &String) -> bool {
        self.env_rec.contains_key(name)
    }

    fn create_mutable_binding(&mut self, name: String, deletion: bool) {
        if !self.env_rec.contains_key(&name) {
            // TODO: change this when error handling comes into play
            panic!("Identifier {} has already been declared", name);
        }

        self.env_rec.insert(
            name,
            DeclerativeEnvironmentRecordBinding {
                value: None,
                can_delete: deletion,
                mutable: true,
                strict: false,
            },
        );
    }

    fn create_immutable_binding(&mut self, name: String, strict: bool) {
        if !self.env_rec.contains_key(&name) {
            // TODO: change this when error handling comes into play
            panic!("Identifier {} has already been declared", name);
        }

        self.env_rec.insert(
            name,
            DeclerativeEnvironmentRecordBinding {
                value: None,
                can_delete: true,
                mutable: false,
                strict: strict,
            },
        );
    }

    fn initialize_binding(&mut self, name: String, value: Value) {
        match self.env_rec.get_mut(&name) {
            Some(ref mut record) => {
                match record.value {
                    Some(_) => {
                        // TODO: change this when error handling comes into play
                        panic!("Identifier {} has already been defined", name);
                    }
                    None => record.value = Some(value),
                }
            }
            None => {}
        }
    }

    fn set_mutable_binding(&mut self, name: String, value: Value, mut strict: bool) {
        if self.env_rec.get(&name).is_none() {
            if strict == true {
                // TODO: change this when error handling comes into play
                panic!("Reference Error: Cannot set mutable binding for {}", name);
            }

            self.create_mutable_binding(name.clone(), true);
            self.initialize_binding(name.clone(), value);
            return;
        }

        let record: &mut DeclerativeEnvironmentRecordBinding = self.env_rec.get_mut(&name).unwrap();
        if record.strict {
            strict = true
        }

        if record.value.is_none() {
            // TODO: change this when error handling comes into play
            panic!("Reference Error: Cannot set mutable binding for {}", name);
        }

        if record.mutable {
            record.value = Some(value);
        } else {
            if strict {
                // TODO: change this when error handling comes into play
                panic!("TypeError: Cannot mutate an immutable binding {}", name);
            }
        }
    }

    fn get_binding_value(&self, name: String, _strict: bool) -> Value {
        if self.env_rec.get(&name).is_some() && self.env_rec.get(&name).unwrap().value.is_some() {
            let record: &DeclerativeEnvironmentRecordBinding = self.env_rec.get(&name).unwrap();
            record.value.as_ref().unwrap().clone()
        } else {
            // TODO: change this when error handling comes into play
            panic!("ReferenceError: Cannot get binding value for {}", name);
        }
    }

    fn delete_binding(&mut self, name: String) -> bool {
        if self.env_rec.get(&name).is_some() {
            if self.env_rec.get(&name).unwrap().can_delete {
                self.env_rec.remove(&name);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn has_this_binding(&self) -> bool {
        false
    }

    fn has_super_binding(&self) -> bool {
        false
    }

    fn with_base_object(&self) -> Value {
        Gc::new(ValueData::Undefined)
    }

    fn get_outer_environment(&self) -> Option<&Box<EnvironmentRecordTrait>> {
        match &self.outer_env {
            Some(outer) => Some(&outer),
            None => None,
        }
    }

    fn set_outer_environment(&mut self, env: Box<EnvironmentRecordTrait>) {
        self.outer_env = Some(env);
    }
}
