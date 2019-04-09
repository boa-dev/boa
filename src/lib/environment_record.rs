//! # Environment Records
//!
//! https://tc39.github.io/ecma262/#sec-environment-records
//! https://tc39.github.io/ecma262/#sec-lexical-environments
//!
//! Some environments are stored as JSObjects. This is for GC, i.e we want to keep an environment if a variable is closed-over (a closure is returned).   
//! All of the logic to handle scope/environment records are stored in here.
//!
//! There are 5 Environment record kinds. They all have methods in common, these are implemented as a the `EnvironmentRecordTrait`
//!

use crate::js::value::{Value, ValueData};
use gc::Gc;
use std::collections::HashMap;

struct DeclerativeEnvironmentRecordBinding {
    binding: Option<Value>,
    can_delete: bool,
    mutable: bool,
    strict: bool,
}

/// A declarative Environment Record binds the set of identifiers defined by the
/// declarations contained within its scope.
struct DeclerativeEnvironmentRecord {
    env_rec: HashMap<String, DeclerativeEnvironmentRecordBinding>,
}

impl EnvironmentRecordTrait for DeclerativeEnvironmentRecord {
    fn has_binding(&self, name: String) -> bool {
        self.env_rec.contains_key(&name)
    }

    fn create_mutable_binding(&mut self, name: String, deletion: bool) {
        if !self.env_rec.contains_key(&name) {
            // TODO: change this when error handling comes into play
            panic!("Identifier {} has already been declared", name);
        }

        self.env_rec.insert(
            name,
            DeclerativeEnvironmentRecordBinding {
                binding: None,
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
                binding: None,
                can_delete: true,
                mutable: false,
                strict: strict,
            },
        );
    }

    fn initialize_binding(&mut self, name: String, value: Value) {
        match self.env_rec.get_mut(&name) {
            Some(ref mut record) => {
                match record.binding {
                    Some(_) => {
                        // TODO: change this when error handling comes into play
                        panic!("Identifier {} has already been defined", name);
                    }
                    None => record.binding = Some(value),
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

        if record.binding.is_none() {
            // TODO: change this when error handling comes into play
            panic!("Reference Error: Cannot set mutable binding for {}", name);
        }

        if record.mutable {
            record.binding = Some(value);
        } else {
            if strict {
                // TODO: change this when error handling comes into play
                panic!("TypeError: Cannot mutate an immutable binding {}", name);
            }
        }
    }

    fn get_binding_value(&self, name: String, _strict: bool) -> Value {
        if self.env_rec.get(&name).is_some() && self.env_rec.get(&name).unwrap().binding.is_some() {
            let record: &DeclerativeEnvironmentRecordBinding = self.env_rec.get(&name).unwrap();
            record.binding.as_ref().unwrap().clone()
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
}

/// https://tc39.github.io/ecma262/#sec-environment-records
///
/// In the ECMAScript specification Environment Records are hierachical and have a base class with abstract methods.   
/// In this implementation we have a trait which represents the behaviour of all EnvironmentRecord types.
pub trait EnvironmentRecordTrait {
    /// Determine if an Environment Record has a binding for the String value N. Return true if it does and false if it does not.
    fn has_binding(&self, name: String) -> bool;

    /// Create a new but uninitialized mutable binding in an Environment Record. The String value N is the text of the bound name.
    /// If the Boolean argument deletion is true the binding may be subsequently deleted.
    fn create_mutable_binding(&mut self, name: String, deletion: bool);

    /// Create a new but uninitialized immutable binding in an Environment Record.
    /// The String value N is the text of the bound name.
    /// If strict is true then attempts to set it after it has been initialized will always throw an exception,
    /// regardless of the strict mode setting of operations that reference that binding.
    fn create_immutable_binding(&mut self, name: String, strict: bool);

    /// Set the value of an already existing but uninitialized binding in an Environment Record.
    /// The String value N is the text of the bound name.
    /// V is the value for the binding and is a value of any ECMAScript language type.
    fn initialize_binding(&mut self, name: String, value: Value);

    /// Set the value of an already existing mutable binding in an Environment Record.
    /// The String value `name` is the text of the bound name.
    /// value is the `value` for the binding and may be a value of any ECMAScript language type. S is a Boolean flag.
    /// If `strict` is true and the binding cannot be set throw a TypeError exception.
    fn set_mutable_binding(&mut self, name: String, value: Value, strict: bool);

    /// Returns the value of an already existing binding from an Environment Record.
    /// The String value N is the text of the bound name.
    /// S is used to identify references originating in strict mode code or that
    /// otherwise require strict mode reference semantics.
    fn get_binding_value(&self, name: String, strict: bool) -> Value;

    /// Delete a binding from an Environment Record.
    /// The String value name is the text of the bound name.
    /// If a binding for name exists, remove the binding and return true.
    /// If the binding exists but cannot be removed return false. If the binding does not exist return true.
    fn delete_binding(&mut self, name: String) -> bool;

    /// Determine if an Environment Record establishes a this binding.
    /// Return true if it does and false if it does not.
    fn has_this_binding(&self) -> bool;

    /// Determine if an Environment Record establishes a super method binding.
    /// Return true if it does and false if it does not.
    fn has_super_binding(&self) -> bool;

    /// If this Environment Record is associated with a with statement, return the with object.
    /// Otherwise, return undefined.
    fn with_base_object(&self) -> Value;
}
