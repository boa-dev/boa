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
    gc::{empty_trace, Finalize, Gc, Trace},
    object::{JsObject, JsPrototype},
    Context, JsResult, JsValue,
};
use boa_interner::Sym;

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
    pub this_value: JsValue,
    /// If the value is "lexical", this is an ArrowFunction and does not have a local this value.
    pub this_binding_status: BindingStatus,
    /// The function object whose invocation caused this Environment Record to be created.
    pub function: JsObject,
    /// If the associated function has super property accesses and is not an ArrowFunction,
    /// `[[HomeObject]]` is the object that the function is bound to as a method.
    /// The default value for `[[HomeObject]]` is undefined.
    pub home_object: JsValue,
    /// If this Environment Record was created by the `[[Construct]]` internal method,
    /// `[[NewTarget]]` is the value of the `[[Construct]]` newTarget parameter.
    /// Otherwise, its value is undefined.
    pub new_target: JsValue,
}

impl FunctionEnvironmentRecord {
    pub fn new(
        f: JsObject,
        this: Option<JsValue>,
        outer: Option<Environment>,
        binding_status: BindingStatus,
        new_target: JsValue,
        context: &mut Context,
    ) -> JsResult<FunctionEnvironmentRecord> {
        let mut func_env = FunctionEnvironmentRecord {
            declarative_record: DeclarativeEnvironmentRecord::new(outer), // the outer environment will come from Environment set as a private property of F - https://tc39.es/ecma262/#sec-ecmascript-function-objects
            function: f,
            this_binding_status: binding_status,
            home_object: JsValue::undefined(),
            new_target,
            this_value: JsValue::undefined(),
        };
        // If a `this` value has been passed, bind it to the environment
        if let Some(v) = this {
            func_env.bind_this_value(v, context)?;
        }
        Ok(func_env)
    }

    /// `9.1.1.3.1 BindThisValue ( V )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bindthisvalue
    pub fn bind_this_value(&mut self, value: JsValue, context: &mut Context) -> JsResult<JsValue> {
        match self.this_binding_status {
            // 1. Assert: envRec.[[ThisBindingStatus]] is not lexical.
            BindingStatus::Lexical => {
                panic!("Cannot bind to an arrow function!");
            }
            // 2. If envRec.[[ThisBindingStatus]] is initialized, throw a ReferenceError exception.
            BindingStatus::Initialized => {
                context.throw_reference_error("Cannot bind to an initialized function!")
            }
            BindingStatus::Uninitialized => {
                // 3. Set envRec.[[ThisValue]] to V.
                self.this_value = value.clone();
                // 4. Set envRec.[[ThisBindingStatus]] to initialized.
                self.this_binding_status = BindingStatus::Initialized;
                // 5. Return V.
                Ok(value)
            }
        }
    }

    /// `9.1.1.3.5 GetSuperBase ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getsuperbase
    pub fn get_super_base(&self, context: &mut Context) -> JsResult<Option<JsPrototype>> {
        // 1. Let home be envRec.[[FunctionObject]].[[HomeObject]].
        let home = &self.home_object;

        // 2. If home has the value undefined, return undefined.
        if home.is_undefined() {
            Ok(None)
        } else {
            // 3. Assert: Type(home) is Object.
            assert!(home.is_object());

            // 4. Return ? home.[[GetPrototypeOf]]().
            Ok(Some(
                home.as_object()
                    .expect("home_object must be an Object")
                    .__get_prototype_of__(context)?,
            ))
        }
    }
}

impl EnvironmentRecordTrait for FunctionEnvironmentRecord {
    fn has_binding(&self, name: Sym, context: &mut Context) -> JsResult<bool> {
        self.declarative_record.has_binding(name, context)
    }

    fn create_mutable_binding(
        &self,
        name: Sym,
        deletion: bool,
        allow_name_reuse: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        self.declarative_record
            .create_mutable_binding(name, deletion, allow_name_reuse, context)
    }

    fn create_immutable_binding(
        &self,
        name: Sym,
        strict: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        self.declarative_record
            .create_immutable_binding(name, strict, context)
    }

    fn initialize_binding(&self, name: Sym, value: JsValue, context: &mut Context) -> JsResult<()> {
        self.declarative_record
            .initialize_binding(name, value, context)
    }

    fn set_mutable_binding(
        &self,
        name: Sym,
        value: JsValue,
        strict: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        self.declarative_record
            .set_mutable_binding(name, value, strict, context)
    }

    fn get_binding_value(
        &self,
        name: Sym,
        strict: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        self.declarative_record
            .get_binding_value(name, strict, context)
    }

    fn delete_binding(&self, name: Sym, context: &mut Context) -> JsResult<bool> {
        self.declarative_record.delete_binding(name, context)
    }

    /// `9.1.1.3.2 HasThisBinding ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-hasthisbinding
    fn has_this_binding(&self) -> bool {
        // 1. If envRec.[[ThisBindingStatus]] is lexical, return false; otherwise, return true.
        !matches!(self.this_binding_status, BindingStatus::Lexical)
    }

    /// `9.1.1.3.3 HasSuperBinding ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-hassuperbinding
    fn has_super_binding(&self) -> bool {
        // 1. If envRec.[[ThisBindingStatus]] is lexical, return false.
        // 2. If envRec.[[FunctionObject]].[[HomeObject]] has the value undefined, return false; otherwise, return true.
        if let BindingStatus::Lexical = self.this_binding_status {
            false
        } else {
            !self.home_object.is_undefined()
        }
    }

    /// `9.1.1.3.4 GetThisBinding ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-getthisbinding
    fn get_this_binding(&self, context: &mut Context) -> JsResult<JsValue> {
        match self.this_binding_status {
            // 1. Assert: envRec.[[ThisBindingStatus]] is not lexical.
            BindingStatus::Lexical => {
                panic!("There is no this for a lexical function record");
            }
            // 2. If envRec.[[ThisBindingStatus]] is uninitialized, throw a ReferenceError exception.
            BindingStatus::Uninitialized => {
                context.throw_reference_error("Uninitialized binding for this function")
            }
            // 3. Return envRec.[[ThisValue]].
            BindingStatus::Initialized => Ok(self.this_value.clone()),
        }
    }

    fn with_base_object(&self) -> Option<JsObject> {
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
        &self,
        name: Sym,
        deletion: bool,
        _scope: VariableScope,
        context: &mut Context,
    ) -> JsResult<()> {
        self.create_mutable_binding(name, deletion, false, context)
    }

    fn recursive_create_immutable_binding(
        &self,
        name: Sym,
        deletion: bool,
        _scope: VariableScope,
        context: &mut Context,
    ) -> JsResult<()> {
        self.create_immutable_binding(name, deletion, context)
    }
}

impl From<FunctionEnvironmentRecord> for Environment {
    fn from(env: FunctionEnvironmentRecord) -> Environment {
        Gc::new(Box::new(env))
    }
}
