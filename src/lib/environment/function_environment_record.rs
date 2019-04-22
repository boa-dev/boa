//! # Function Environment Records
//!
//! A function Environment Record is a declarative Environment Record that is used to represent
//! the top-level scope of a function and, if the function is not an ArrowFunction,
//! provides a `this` binding.
//! If a function is not an ArrowFunction function and references super,
//! its function Environment Record also contains the state that is used to perform super method invocations
//! from within the function.
//! More info:  https://tc39.github.io/ecma262/#sec-function-environment-records

use crate::environment::declerative_environment_record::DeclerativeEnvironmentRecordBinding;
use crate::environment::environment_record::EnvironmentRecordTrait;
use crate::js::value::{Value, ValueData};
use gc::Gc;
use std::collections::hash_map::HashMap;

/// Different binding status for `this`.
/// Usually set on a function environment record
pub enum BindingStatus {
    /// If the value is "lexical", this is an ArrowFunction and does not have a local this value.
    Lexical,
    /// If initialized the function environment record has already been bound with a `this` value
    Initialized,
    /// If uninitialized the function environment record has not been bouned with a `this` value
    Uninitialized,
}

/// https://tc39.github.io/ecma262/#table-16
pub struct FunctionEnvironmentRecord {
    pub env_rec: HashMap<String, DeclerativeEnvironmentRecordBinding>,
    /// This is the this value used for this invocation of the function.
    pub this_value: Value,
    /// If the value is "lexical", this is an ArrowFunction and does not have a local this value.
    pub this_binding_status: BindingStatus,
    /// The function object whose invocation caused this Environment Record to be created.
    pub function_object: Value,
    /// If the associated function has super property accesses and is not an ArrowFunction,
    /// [[HomeObject]] is the object that the function is bound to as a method.
    /// The default value for [[HomeObject]] is undefined.
    pub home_object: Value,
    /// If this Environment Record was created by the [[Construct]] internal method,
    /// [[NewTarget]] is the value of the [[Construct]] newTarget parameter.
    /// Otherwise, its value is undefined.
    pub new_target: Value,
    /// Reference to the outer environment to help with the scope chain
    /// Option type is needed as some environments can be created before we know what the outer env is
    pub outer_env: Option<Box<EnvironmentRecordTrait>>,
}

impl FunctionEnvironmentRecord {
    fn bind_this_value(&mut self, value: Value) {
        match self.this_binding_status {
            // You can not bind an arrow function, their `this` value comes from the lexical scope above
            BindingStatus::Lexical => {
                // TODO: change this when error handling comes into play
                panic!("Cannot bind to an arrow function!");
            }
            // You can not bind a function twice
            BindingStatus::Initialized => {
                // TODO: change this when error handling comes into play
                panic!("Reference Error: Cannot bind to an initialised function!");
            }

            BindingStatus::Uninitialized => {
                self.this_value = value;
                self.this_binding_status = BindingStatus::Initialized;
            }
        }
    }

    fn get_this_binding(&self) -> Value {
        match self.this_binding_status {
            BindingStatus::Lexical => {
                // TODO: change this when error handling comes into play
                panic!("There is no this for a lexical function record");
            }
            BindingStatus::Uninitialized => {
                // TODO: change this when error handling comes into play
                panic!("Reference Error: Unitialised binding for this function");
            }

            BindingStatus::Initialized => self.this_value.clone(),
        }
    }
    // TODO: get_super_base can't implement until GetPrototypeof is implemented on object
}

impl EnvironmentRecordTrait for FunctionEnvironmentRecord {
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

    fn has_super_binding(&self) -> bool {
        match self.this_binding_status {
            BindingStatus::Lexical => false,
            _ => {
                if self.home_object.is_undefined() {
                    false
                } else {
                    true
                }
            }
        }
    }

    fn has_this_binding(&self) -> bool {
        match self.this_binding_status {
            BindingStatus::Lexical => false,
            _ => true,
        }
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
