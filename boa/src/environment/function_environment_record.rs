//! # Function Environment Records
//!
//! A function Environment Record is a declarative Environment Record that is used to represent
//! the top-level scope of a function and, if the function is not an `ArrowFunction`,
//! provides a `this` binding.
//! If a function is not an `ArrowFunction` function and references super,
//! its function Environment Record also contains the state that is used to perform super method invocations
//! from within the function.
//! More info: <https://tc39.es/ecma262/#sec-function-environment-records>

use super::ErrorKind;
use crate::{
    environment::{
        declarative_environment_record::DeclarativeEnvironmentRecordBinding,
        environment_record_trait::EnvironmentRecordTrait,
        lexical_environment::{Environment, EnvironmentType},
    },
    gc::{empty_trace, Finalize, Trace},
    object::GcObject,
    Value,
};
use rustc_hash::FxHashMap;

/// Different binding status for `this`.
/// Usually set on a function environment record
#[derive(Copy, Finalize, Debug, Clone)]
pub enum BindingStatus {
    /// If the value is "lexical", this is an ArrowFunction and does not have a local this value.
    Lexical,
    /// If initialized the function environment record has already been bound with a `this` value
    Initialized,
    /// If uninitialized the function environment record has not been bouned with a `this` value
    Uninitialized,
}

unsafe impl Trace for BindingStatus {
    empty_trace!();
}

/// <https://tc39.es/ecma262/#table-16>
#[derive(Debug, Trace, Finalize, Clone)]
pub struct FunctionEnvironmentRecord {
    pub env_rec: FxHashMap<String, DeclarativeEnvironmentRecordBinding>,
    /// This is the this value used for this invocation of the function.
    pub this_value: Value,
    /// If the value is "lexical", this is an ArrowFunction and does not have a local this value.
    pub this_binding_status: BindingStatus,
    /// The function object whose invocation caused this Environment Record to be created.
    pub function: GcObject,
    /// If the associated function has super property accesses and is not an ArrowFunction,
    /// `[[HomeObject]]` is the object that the function is bound to as a method.
    /// The default value for `[[HomeObject]]` is undefined.
    pub home_object: Value,
    /// If this Environment Record was created by the `[[Construct]]` internal method,
    /// `[[NewTarget]]` is the value of the `[[Construct]]` newTarget parameter.
    /// Otherwise, its value is undefined.
    pub new_target: Value,
    /// Reference to the outer environment to help with the scope chain
    /// Option type is needed as some environments can be created before we know what the outer env is
    pub outer_env: Option<Environment>,
}

impl FunctionEnvironmentRecord {
    pub fn bind_this_value(&mut self, value: Value) -> Result<Value, ErrorKind> {
        match self.this_binding_status {
            // You can not bind an arrow function, their `this` value comes from the lexical scope above
            BindingStatus::Lexical => {
                panic!("Cannot bind to an arrow function!");
            }
            // You can not bind a function twice
            BindingStatus::Initialized => Err(ErrorKind::new_reference_error(
                "Cannot bind to an initialised function!",
            )),

            BindingStatus::Uninitialized => {
                self.this_value = value.clone();
                self.this_binding_status = BindingStatus::Initialized;
                Ok(value)
            }
        }
    }

    pub fn get_super_base(&self) -> Value {
        let home = &self.home_object;
        if home.is_undefined() {
            Value::Undefined
        } else {
            assert!(home.is_object());
            home.as_object()
                .expect("home_object must be an Object")
                .prototype_instance()
        }
    }
}

impl EnvironmentRecordTrait for FunctionEnvironmentRecord {
    fn has_binding(&self, name: &str) -> bool {
        self.env_rec.contains_key(name)
    }

    fn create_mutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        allow_name_reuse: bool,
    ) -> Result<(), ErrorKind> {
        if !allow_name_reuse {
            assert!(
                !self.env_rec.contains_key(&name),
                "Identifier {} has already been declared",
                name
            );
        }

        self.env_rec.insert(
            name,
            DeclarativeEnvironmentRecordBinding {
                value: None,
                can_delete: deletion,
                mutable: true,
                strict: false,
            },
        );
        Ok(())
    }

    fn get_this_binding(&self) -> Result<Value, ErrorKind> {
        match self.this_binding_status {
            BindingStatus::Lexical => {
                panic!("There is no this for a lexical function record");
            }
            BindingStatus::Uninitialized => Err(ErrorKind::new_reference_error(
                "Uninitialised binding for this function",
            )),

            BindingStatus::Initialized => Ok(self.this_value.clone()),
        }
    }

    fn create_immutable_binding(&mut self, name: String, strict: bool) -> Result<(), ErrorKind> {
        assert!(
            !self.env_rec.contains_key(&name),
            "Identifier {} has already been declared",
            name
        );

        self.env_rec.insert(
            name,
            DeclarativeEnvironmentRecordBinding {
                value: None,
                can_delete: true,
                mutable: false,
                strict,
            },
        );
        Ok(())
    }

    fn initialize_binding(&mut self, name: &str, value: Value) -> Result<(), ErrorKind> {
        if let Some(ref mut record) = self.env_rec.get_mut(name) {
            if record.value.is_none() {
                record.value = Some(value);
                return Ok(());
            }
        }
        panic!("record must have binding for {}", name)
    }

    #[allow(clippy::else_if_without_else)]
    fn set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        mut strict: bool,
    ) -> Result<(), ErrorKind> {
        if self.env_rec.get(name).is_none() {
            if strict {
                return Err(ErrorKind::new_reference_error(format!(
                    "{} not found",
                    name
                )));
            }

            self.create_mutable_binding(name.to_owned(), true, false)?;
            self.initialize_binding(name, value)?;
            return Ok(());
        }

        let record: &mut DeclarativeEnvironmentRecordBinding = self.env_rec.get_mut(name).unwrap();
        if record.strict {
            strict = true
        }
        if record.value.is_none() {
            return Err(ErrorKind::new_reference_error(format!(
                "{} has not been initialized",
                name
            )));
        }
        if record.mutable {
            record.value = Some(value);
        } else if strict {
            return Err(ErrorKind::new_type_error(format!(
                "Cannot mutate an immutable binding {}",
                name
            )));
        }

        Ok(())
    }

    fn get_binding_value(&self, name: &str, _strict: bool) -> Result<Value, ErrorKind> {
        if let Some(binding) = self.env_rec.get(name) {
            if let Some(ref val) = binding.value {
                Ok(val.clone())
            } else {
                Err(ErrorKind::new_reference_error(format!(
                    "{} is an uninitialized binding",
                    name
                )))
            }
        } else {
            panic!("Cannot get binding value for {}", name);
        }
    }

    fn delete_binding(&mut self, name: &str) -> bool {
        match self.env_rec.get(name) {
            Some(binding) => {
                if binding.can_delete {
                    self.env_rec.remove(name);
                    true
                } else {
                    false
                }
            }
            None => panic!("env_rec has no binding for {}", name),
        }
    }

    fn has_super_binding(&self) -> bool {
        if let BindingStatus::Lexical = self.this_binding_status {
            false
        } else {
            !self.home_object.is_undefined()
        }
    }

    fn has_this_binding(&self) -> bool {
        !matches!(self.this_binding_status, BindingStatus::Lexical)
    }

    fn with_base_object(&self) -> Value {
        Value::undefined()
    }

    fn get_outer_environment(&self) -> Option<Environment> {
        match &self.outer_env {
            Some(outer) => Some(outer.clone()),
            None => None,
        }
    }

    fn set_outer_environment(&mut self, env: Environment) {
        self.outer_env = Some(env);
    }

    fn get_environment_type(&self) -> EnvironmentType {
        EnvironmentType::Function
    }

    fn get_global_object(&self) -> Option<Value> {
        match &self.outer_env {
            Some(ref outer) => outer.borrow().get_global_object(),
            None => None,
        }
    }
}
