//! # Function Environment Records
//!
//! A function Environment Record is a declarative Environment Record that is used to represent
//! the top-level scope of a function and, if the function is not an `ArrowFunction`,
//! provides a `this` binding.
//! If a function is not an `ArrowFunction` function and references super,
//! its function Environment Record also contains the state that is used to perform super method invocations
//! from within the function.
//! More info: <https://tc39.es/ecma262/#sec-function-environment-records>

use crate::{
    environment::{
        declarative_environment_record::DeclarativeEnvironmentRecord,
        environment_record_trait::EnvironmentRecordTrait,
        lexical_environment::{Environment, EnvironmentType, VariableScope},
    },
    gc::{empty_trace, Finalize, Trace},
    object::GcObject,
    Context, Result, Value,
};

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
    pub declarative_record: DeclarativeEnvironmentRecord,
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
}

impl FunctionEnvironmentRecord {
    pub fn bind_this_value(&mut self, value: Value) -> Result<Value> {
        match self.this_binding_status {
            // You can not bind an arrow function, their `this` value comes from the lexical scope above
            BindingStatus::Lexical => {
                panic!("Cannot bind to an arrow function!");
            }
            // You can not bind a function twice
            BindingStatus::Initialized => {
                todo!();
                // context.throw_reference_error("Cannot bind to an initialised function!")
            }
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
        self.declarative_record.has_binding(name)
    }

    fn create_mutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        allow_name_reuse: bool,
        context: &mut Context,
    ) -> Result<()> {
        self.declarative_record
            .create_mutable_binding(name, deletion, allow_name_reuse, context)
    }

    fn create_immutable_binding(
        &mut self,
        name: String,
        strict: bool,
        context: &mut Context,
    ) -> Result<()> {
        self.declarative_record
            .create_immutable_binding(name, strict, context)
    }

    fn initialize_binding(
        &mut self,
        name: &str,
        value: Value,
        context: &mut Context,
    ) -> Result<()> {
        self.declarative_record
            .initialize_binding(name, value, context)
    }

    fn set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        strict: bool,
        context: &mut Context,
    ) -> Result<()> {
        self.declarative_record
            .set_mutable_binding(name, value, strict, context)
    }

    fn get_binding_value(&self, name: &str, strict: bool, context: &mut Context) -> Result<Value> {
        self.declarative_record
            .get_binding_value(name, strict, context)
    }

    fn delete_binding(&mut self, name: &str) -> bool {
        self.declarative_record.delete_binding(name)
    }

    fn has_this_binding(&self) -> bool {
        !matches!(self.this_binding_status, BindingStatus::Lexical)
    }

    fn get_this_binding(&self, context: &mut Context) -> Result<Value> {
        match self.this_binding_status {
            BindingStatus::Lexical => {
                panic!("There is no this for a lexical function record");
            }
            BindingStatus::Uninitialized => {
                context.throw_reference_error("Uninitialised binding for this function")
            }
            BindingStatus::Initialized => Ok(self.this_value.clone()),
        }
    }

    fn has_super_binding(&self) -> bool {
        if let BindingStatus::Lexical = self.this_binding_status {
            false
        } else {
            !self.home_object.is_undefined()
        }
    }

    fn with_base_object(&self) -> Option<GcObject> {
        None
    }

    fn get_outer_environment_ref(&self) -> Option<&Environment> {
        self.declarative_record.get_outer_environment_ref()
    }

    fn set_outer_environment(&mut self, env: Environment) {
        self.declarative_record.set_outer_environment(env)
    }

    fn get_environment_type(&self) -> EnvironmentType {
        EnvironmentType::Function
    }

    fn recursive_create_mutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        _scope: VariableScope,
        context: &mut Context,
    ) -> Result<()> {
        self.create_mutable_binding(name, deletion, false, context)
    }

    fn recursive_create_immutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        _scope: VariableScope,
        context: &mut Context,
    ) -> Result<()> {
        self.create_immutable_binding(name, deletion, context)
    }
}
