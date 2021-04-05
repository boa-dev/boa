//! # Lexical Environment
//!
//! <https://tc39.es/ecma262/#sec-lexical-environment-operations>
//!
//! The following operations are used to operate upon lexical environments
//! This is the entrypoint to lexical environments.

use super::ErrorKind;
use crate::{
    environment::{
        declarative_environment_record::DeclarativeEnvironmentRecord,
        environment_record_trait::EnvironmentRecordTrait,
        function_environment_record::{BindingStatus, FunctionEnvironmentRecord},
        global_environment_record::GlobalEnvironmentRecord,
        object_environment_record::ObjectEnvironmentRecord,
    },
    object::GcObject,
    BoaProfiler, Value,
};
use gc::{Gc, GcCell};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{collections::VecDeque, error, fmt};

/// Environments are wrapped in a Box and then in a GC wrapper
pub type Environment = Gc<GcCell<Box<dyn EnvironmentRecordTrait>>>;

/// Give each environment an easy way to declare its own type
/// This helps with comparisons
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EnvironmentType {
    Declarative,
    Function,
    Global,
    Object,
}

/// The scope of a given variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VariableScope {
    /// The variable declaration is scoped to the current block (`let` and `const`)
    Block,
    /// The variable declaration is scoped to the current function (`var`)
    Function,
}

#[derive(Debug, Clone)]
pub struct LexicalEnvironment {
    environment_stack: VecDeque<Environment>,
}

/// An error that occurred during lexing or compiling of the source input.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnvironmentError {
    details: String,
}

impl EnvironmentError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl error::Error for EnvironmentError {
    fn description(&self) -> &str {
        &self.details
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl LexicalEnvironment {
    pub fn new(global: Value) -> Self {
        let _timer = BoaProfiler::global().start_event("LexicalEnvironment::new", "env");
        let global_env = new_global_environment(global.clone(), global);
        let mut lexical_env = Self {
            environment_stack: VecDeque::new(),
        };

        // lexical_env.push(global_env);
        lexical_env.environment_stack.push_back(global_env);
        lexical_env
    }

    pub fn push(&mut self, env: Environment) {
        self.environment_stack.push_back(env);
    }

    pub fn pop(&mut self) -> Option<Environment> {
        self.environment_stack.pop_back()
    }

    pub fn get_global_object(&self) -> Option<Value> {
        self.get_current_environment_ref()
            .borrow()
            .get_global_object()
    }

    pub fn get_this_binding(&self) -> Result<Value, ErrorKind> {
        self.get_current_environment_ref()
            .borrow()
            .recursive_get_this_binding()
    }

    pub fn create_mutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        scope: VariableScope,
    ) -> Result<(), ErrorKind> {
        self.get_current_environment()
            .borrow_mut()
            .recursive_create_mutable_binding(name, deletion, scope)
    }

    pub fn create_immutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        scope: VariableScope,
    ) -> Result<(), ErrorKind> {
        self.get_current_environment()
            .borrow_mut()
            .recursive_create_immutable_binding(name, deletion, scope)
    }

    pub fn set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        strict: bool,
    ) -> Result<(), ErrorKind> {
        self.get_current_environment()
            .borrow_mut()
            .recursive_set_mutable_binding(name, value, strict)
    }

    pub fn initialize_binding(&mut self, name: &str, value: Value) -> Result<(), ErrorKind> {
        self.get_current_environment()
            .borrow_mut()
            .recursive_initialize_binding(name, value)
    }

    /// get_current_environment_ref is used when you only need to borrow the environment
    /// (you only need to add a new variable binding, or you want to fetch a value)
    pub fn get_current_environment_ref(&self) -> &Environment {
        self.environment_stack
            .back()
            .expect("Could not get current environment")
    }

    /// When neededing to clone an environment (linking it with another environnment)
    /// cloning is more suited. The GC will remove the env once nothing is linking to it anymore
    pub fn get_current_environment(&mut self) -> &mut Environment {
        self.environment_stack
            .back_mut()
            .expect("Could not get mutable reference to back object")
    }

    pub fn has_binding(&self, name: &str) -> bool {
        self.get_current_environment_ref()
            .borrow()
            .recursive_has_binding(name)
    }

    pub fn get_binding_value(&self, name: &str) -> Result<Value, ErrorKind> {
        self.get_current_environment_ref()
            .borrow()
            .recursive_get_binding_value(name)
    }
}

pub fn new_declarative_environment(env: Option<Environment>) -> Environment {
    let _timer = BoaProfiler::global().start_event("new_declarative_environment", "env");
    let boxed_env = Box::new(DeclarativeEnvironmentRecord {
        env_rec: FxHashMap::default(),
        outer_env: env,
    });

    Gc::new(GcCell::new(boxed_env))
}

pub fn new_function_environment(
    f: GcObject,
    this: Option<Value>,
    outer: Option<Environment>,
    binding_status: BindingStatus,
    new_target: Value,
) -> Environment {
    let mut func_env = FunctionEnvironmentRecord {
        declarative_record: DeclarativeEnvironmentRecord {
            env_rec: FxHashMap::default(),
            outer_env: outer, // this will come from Environment set as a private property of F - https://tc39.es/ecma262/#sec-ecmascript-function-objects
        },
        function: f,
        this_binding_status: binding_status,
        home_object: Value::undefined(),
        new_target,
        this_value: Value::undefined(),
    };
    // If a `this` value has been passed, bind it to the environment
    if let Some(v) = this {
        func_env.bind_this_value(v).unwrap();
    }
    Gc::new(GcCell::new(Box::new(func_env)))
}

pub fn new_object_environment(object: Value, environment: Option<Environment>) -> Environment {
    Gc::new(GcCell::new(Box::new(ObjectEnvironmentRecord {
        bindings: object,
        outer_env: environment,
        /// Object Environment Records created for with statements (13.11)
        /// can provide their binding object as an implicit this value for use in function calls.
        /// The capability is controlled by a withEnvironment Boolean value that is associated
        /// with each object Environment Record. By default, the value of withEnvironment is false
        /// for any object Environment Record.
        with_environment: false,
    })))
}

pub fn new_global_environment(global: Value, this_value: Value) -> Environment {
    let obj_rec = ObjectEnvironmentRecord {
        bindings: global,
        outer_env: None,
        /// Object Environment Records created for with statements (13.11)
        /// can provide their binding object as an implicit this value for use in function calls.
        /// The capability is controlled by a withEnvironment Boolean value that is associated
        /// with each object Environment Record. By default, the value of withEnvironment is false
        /// for any object Environment Record.
        with_environment: false,
    };

    let dcl_rec = DeclarativeEnvironmentRecord {
        env_rec: FxHashMap::default(),
        outer_env: None,
    };

    Gc::new(GcCell::new(Box::new(GlobalEnvironmentRecord {
        object_record: obj_rec,
        global_this_binding: this_value,
        declarative_record: dcl_rec,
        var_names: FxHashSet::default(),
    })))
}

#[cfg(test)]
mod tests {
    use crate::exec;

    #[test]
    fn let_is_blockscoped() {
        let scenario = r#"
          {
            let bar = "bar";
          }

          try{
            bar;
          } catch (err) {
            err.message
          }
        "#;

        assert_eq!(&exec(scenario), "\"bar is not defined\"");
    }

    #[test]
    fn const_is_blockscoped() {
        let scenario = r#"
          {
            const bar = "bar";
          }

          try{
            bar;
          } catch (err) {
            err.message
          }
        "#;

        assert_eq!(&exec(scenario), "\"bar is not defined\"");
    }

    #[test]
    fn var_not_blockscoped() {
        let scenario = r#"
          {
            var bar = "bar";
          }
          bar == "bar";
        "#;

        assert_eq!(&exec(scenario), "true");
    }

    #[test]
    fn functions_use_declaration_scope() {
        let scenario = r#"
          function foo() {
            try {
                bar;
            } catch (err) {
                return err.message;
            }
          }
          {
            let bar = "bar";
            foo();
          }
        "#;

        assert_eq!(&exec(scenario), "\"bar is not defined\"");
    }

    #[test]
    fn set_outer_var_in_blockscope() {
        let scenario = r#"
          var bar;
          {
            bar = "foo";
          }
          bar == "foo";
        "#;

        assert_eq!(&exec(scenario), "true");
    }

    #[test]
    fn set_outer_let_in_blockscope() {
        let scenario = r#"
          let bar;
          {
            bar = "foo";
          }
          bar == "foo";
        "#;

        assert_eq!(&exec(scenario), "true");
    }
}
