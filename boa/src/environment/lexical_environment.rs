//! # Lexical Environment
//!
//! <https://tc39.es/ecma262/#sec-lexical-environment-operations>
//!
//! The following operations are used to operate upon lexical environments
//! This is the entrypoint to lexical environments.

use super::global_environment_record::GlobalEnvironmentRecord;
use crate::{
    environment::environment_record_trait::EnvironmentRecordTrait, object::GcObject, BoaProfiler,
    Context, Result, Value,
};
use gc::{Gc, GcCell};
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

impl error::Error for EnvironmentError {}

impl LexicalEnvironment {
    pub fn new(global: GcObject) -> Self {
        let _timer = BoaProfiler::global().start_event("LexicalEnvironment::new", "env");
        let global_env = GlobalEnvironmentRecord::new(global.clone(), global);
        let mut lexical_env = Self {
            environment_stack: VecDeque::new(),
        };

        // lexical_env.push(global_env);
        lexical_env.environment_stack.push_back(global_env);
        lexical_env
    }
}

impl Context {
    pub(crate) fn push_environment(&mut self, env: Environment) {
        self.realm.environment.environment_stack.push_back(env);
    }

    pub(crate) fn pop_environment(&mut self) -> Option<Environment> {
        self.realm.environment.environment_stack.pop_back()
    }

    pub(crate) fn get_this_binding(&mut self) -> Result<Value> {
        self.get_current_environment()
            .borrow()
            .recursive_get_this_binding(self)
    }

    pub(crate) fn create_mutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        scope: VariableScope,
    ) -> Result<()> {
        self.get_current_environment()
            .borrow_mut()
            .recursive_create_mutable_binding(name, deletion, scope, self)
    }

    pub(crate) fn create_immutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        scope: VariableScope,
    ) -> Result<()> {
        self.get_current_environment()
            .borrow_mut()
            .recursive_create_immutable_binding(name, deletion, scope, self)
    }

    pub(crate) fn set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        strict: bool,
    ) -> Result<()> {
        self.get_current_environment()
            .borrow_mut()
            .recursive_set_mutable_binding(name, value, strict, self)
    }

    pub(crate) fn initialize_binding(&mut self, name: &str, value: Value) -> Result<()> {
        self.get_current_environment()
            .borrow_mut()
            .recursive_initialize_binding(name, value, self)
    }

    /// When neededing to clone an environment (linking it with another environnment)
    /// cloning is more suited. The GC will remove the env once nothing is linking to it anymore
    pub(crate) fn get_current_environment(&mut self) -> Environment {
        self.realm
            .environment
            .environment_stack
            .back_mut()
            .expect("Could not get mutable reference to back object")
            .clone()
    }

    pub(crate) fn has_binding(&mut self, name: &str) -> bool {
        self.get_current_environment()
            .borrow()
            .recursive_has_binding(name)
    }

    pub(crate) fn get_binding_value(&mut self, name: &str) -> Result<Value> {
        self.get_current_environment()
            .borrow()
            .recursive_get_binding_value(name, self)
    }
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
