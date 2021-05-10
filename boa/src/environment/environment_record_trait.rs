//! # Environment Records
//!
//! <https://tc39.es/ecma262/#sec-environment-records>
//! <https://tc39.es/ecma262/#sec-lexical-environments>
//!
//! Some environments are stored as `JSObjects`. This is for GC, i.e we want to keep an environment if a variable is closed-over (a closure is returned).
//! All of the logic to handle scope/environment records are stored in here.
//!
//! There are 5 Environment record kinds. They all have methods in common, these are implemented as a the `EnvironmentRecordTrait`
//!

use crate::{environment::lexical_environment::VariableScope, object::GcObject};
use crate::{
    environment::lexical_environment::{Environment, EnvironmentType},
    gc::{Finalize, Trace},
    Context, Result, Value,
};
use std::fmt::Debug;

/// <https://tc39.es/ecma262/#sec-environment-records>
///
/// In the ECMAScript specification Environment Records are hierachical and have a base class with abstract methods.
/// In this implementation we have a trait which represents the behaviour of all `EnvironmentRecord` types.
pub trait EnvironmentRecordTrait: Debug + Trace + Finalize {
    /// Determine if an Environment Record has a binding for the String value N.
    /// Return true if it does and false if it does not.
    fn has_binding(&self, name: &str) -> bool;

    /// Create a new but uninitialized mutable binding in an Environment Record. The String value N is the text of the bound name.
    /// If the Boolean argument deletion is true the binding may be subsequently deleted.
    ///
    /// * `allow_name_reuse` - specifies whether or not reusing binding names is allowed.
    ///
    /// Most variable names cannot be reused, but functions in JavaScript are allowed to have multiple
    /// paraments with the same name.
    fn create_mutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        allow_name_reuse: bool,
        context: &mut Context,
    ) -> Result<()>;

    /// Create a new but uninitialized immutable binding in an Environment Record.
    /// The String value N is the text of the bound name.
    /// If strict is true then attempts to set it after it has been initialized will always throw an exception,
    /// regardless of the strict mode setting of operations that reference that binding.
    fn create_immutable_binding(
        &mut self,
        name: String,
        strict: bool,
        context: &mut Context,
    ) -> Result<()>;

    /// Set the value of an already existing but uninitialized binding in an Environment Record.
    /// The String value N is the text of the bound name.
    /// V is the value for the binding and is a value of any ECMAScript language type.
    fn initialize_binding(&mut self, name: &str, value: Value, context: &mut Context)
        -> Result<()>;

    /// Set the value of an already existing mutable binding in an Environment Record.
    /// The String value `name` is the text of the bound name.
    /// value is the `value` for the binding and may be a value of any ECMAScript language type. S is a Boolean flag.
    /// If `strict` is true and the binding cannot be set throw a TypeError exception.
    fn set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        strict: bool,
        context: &mut Context,
    ) -> Result<()>;

    /// Returns the value of an already existing binding from an Environment Record.
    /// The String value N is the text of the bound name.
    /// S is used to identify references originating in strict mode code or that
    /// otherwise require strict mode reference semantics.
    fn get_binding_value(&self, name: &str, strict: bool, context: &mut Context) -> Result<Value>;

    /// Delete a binding from an Environment Record.
    /// The String value name is the text of the bound name.
    /// If a binding for name exists, remove the binding and return true.
    /// If the binding exists but cannot be removed return false. If the binding does not exist return true.
    fn delete_binding(&mut self, name: &str) -> bool;

    /// Determine if an Environment Record establishes a this binding.
    /// Return true if it does and false if it does not.
    fn has_this_binding(&self) -> bool;

    /// Return the `this` binding from the environment
    fn get_this_binding(&self, context: &mut Context) -> Result<Value>;

    /// Determine if an Environment Record establishes a super method binding.
    /// Return true if it does and false if it does not.
    fn has_super_binding(&self) -> bool;

    /// If this Environment Record is associated with a with statement, return the with object.
    /// Otherwise, return None.
    fn with_base_object(&self) -> Option<GcObject>;

    /// Get the next environment up
    fn get_outer_environment_ref(&self) -> Option<&Environment>;
    fn get_outer_environment(&self) -> Option<Environment> {
        self.get_outer_environment_ref().cloned()
    }

    /// Set the next environment up
    fn set_outer_environment(&mut self, env: Environment);

    /// Get the type of environment this is
    fn get_environment_type(&self) -> EnvironmentType;

    /// Return the `this` binding from the environment or try to get it from outer environments
    fn recursive_get_this_binding(&self, context: &mut Context) -> Result<Value> {
        if self.has_this_binding() {
            self.get_this_binding(context)
        } else {
            match self.get_outer_environment_ref() {
                Some(outer) => outer.borrow().recursive_get_this_binding(context),
                None => Ok(Value::Undefined),
            }
        }
    }

    /// Create mutable binding while handling outer environments
    fn recursive_create_mutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        scope: VariableScope,
        context: &mut Context,
    ) -> Result<()> {
        match scope {
            VariableScope::Block => self.create_mutable_binding(name, deletion, false, context),
            VariableScope::Function => self
                .get_outer_environment_ref()
                .expect("No function or global environment")
                .borrow_mut()
                .recursive_create_mutable_binding(name, deletion, scope, context),
        }
    }

    /// Create immutable binding while handling outer environments
    fn recursive_create_immutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        scope: VariableScope,
        context: &mut Context,
    ) -> Result<()> {
        match scope {
            VariableScope::Block => self.create_immutable_binding(name, deletion, context),
            VariableScope::Function => self
                .get_outer_environment_ref()
                .expect("No function or global environment")
                .borrow_mut()
                .recursive_create_immutable_binding(name, deletion, scope, context),
        }
    }

    /// Set mutable binding while handling outer environments
    fn recursive_set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        strict: bool,
        context: &mut Context,
    ) -> Result<()> {
        if self.has_binding(name) {
            self.set_mutable_binding(name, value, strict, context)
        } else {
            self.get_outer_environment_ref()
                .expect("Environment stack underflow")
                .borrow_mut()
                .recursive_set_mutable_binding(name, value, strict, context)
        }
    }

    /// Initialize binding while handling outer environments
    fn recursive_initialize_binding(
        &mut self,
        name: &str,
        value: Value,
        context: &mut Context,
    ) -> Result<()> {
        if self.has_binding(name) {
            self.initialize_binding(name, value, context)
        } else {
            self.get_outer_environment_ref()
                .expect("Environment stack underflow")
                .borrow_mut()
                .recursive_initialize_binding(name, value, context)
        }
    }

    /// Check if a binding exists in current or any outer environment
    fn recursive_has_binding(&self, name: &str) -> bool {
        self.has_binding(name)
            || match self.get_outer_environment_ref() {
                Some(outer) => outer.borrow().recursive_has_binding(name),
                None => false,
            }
    }

    /// Retrieve binding from current or any outer environment
    fn recursive_get_binding_value(&self, name: &str, context: &mut Context) -> Result<Value> {
        if self.has_binding(name) {
            self.get_binding_value(name, false, context)
        } else {
            match self.get_outer_environment_ref() {
                Some(outer) => outer.borrow().recursive_get_binding_value(name, context),
                None => context.throw_reference_error(format!("{} is not defined", name)),
            }
        }
    }
}
