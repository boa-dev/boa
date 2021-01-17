//! # Lexical Environment
//!
//! <https://tc39.es/ecma262/#sec-lexical-environment-operations>
//!
//! The following operations are used to operate upon lexical environments
//! This is the entrypoint to lexical environments.

use crate::{
    environment::{
        declarative_environment_record::DeclarativeEnvironmentRecord,
        environment_record_trait::EnvironmentRecordTrait,
        function_environment_record::{BindingStatus, FunctionEnvironmentRecord},
        global_environment_record::GlobalEnvironmentRecord,
        object_environment_record::ObjectEnvironmentRecord,
    },
    object::GcObject,
    BoaProfiler, Context, Result, Value,
};
use gc::{Gc, GcCell};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
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
    environment_stack: RefCell<VecDeque<Environment>>,
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
        let lexical_env = Self {
            environment_stack: RefCell::new(VecDeque::new()),
        };

        // lexical_env.push(global_env);
        lexical_env
            .environment_stack
            .borrow_mut()
            .push_back(global_env);
        lexical_env
    }

    pub fn push(&self, env: Environment) {
        let current_env: Environment = self.get_current_environment();
        env.borrow_mut().set_outer_environment(current_env);
        self.environment_stack.borrow_mut().push_back(env);
    }

    pub fn pop(&self) -> Option<Environment> {
        self.environment_stack.borrow_mut().pop_back()
    }

    pub fn get_global_object(&self) -> Option<Value> {
        self.environment_stack
            .borrow()
            .get(0)
            .expect("")
            .borrow()
            .get_global_object()
    }

    pub fn get_this_binding(&self, context: &Context) -> Result<Value> {
        Ok(self
            .environment_stack
            .borrow()
            .iter()
            .rev()
            .find(|env| env.borrow().has_this_binding())
            .map(|env| env.borrow().get_this_binding(context))
            .transpose()?
            .unwrap_or_else(Value::undefined))
    }

    pub fn create_mutable_binding(
        &self,
        name: String,
        deletion: bool,
        scope: VariableScope,
        context: &Context,
    ) -> Result<()> {
        match scope {
            VariableScope::Block => self
                .get_current_environment()
                .borrow_mut()
                .create_mutable_binding(name, deletion, false, context),
            VariableScope::Function => {
                // Find the first function or global environment (from the top of the stack)
                let stack = &self.environment_stack.borrow();
                let env = stack
                    .iter()
                    .rev()
                    .find(|env| {
                        matches!(
                            env.borrow().get_environment_type(),
                            EnvironmentType::Function | EnvironmentType::Global
                        )
                    })
                    .expect("No function or global environment");

                let mut env = env.borrow_mut();
                env.create_mutable_binding(name, deletion, false, context)
            }
        }
    }

    pub fn create_immutable_binding(
        &self,
        name: String,
        deletion: bool,
        scope: VariableScope,
        context: &Context,
    ) -> Result<()> {
        match scope {
            VariableScope::Block => self
                .get_current_environment()
                .borrow_mut()
                .create_immutable_binding(name, deletion, context),
            VariableScope::Function => {
                // Find the first function or global environment (from the top of the stack)
                let stack = self.environment_stack.borrow();
                let env = stack
                    .iter()
                    .rev()
                    .find(|env| {
                        matches!(
                            env.borrow().get_environment_type(),
                            EnvironmentType::Function | EnvironmentType::Global
                        )
                    })
                    .expect("No function or global environment");

                let res = env
                    .borrow_mut()
                    .create_immutable_binding(name, deletion, context);
                res
            }
        }
    }

    pub fn set_mutable_binding(
        &self,
        name: &str,
        value: Value,
        strict: bool,
        context: &Context,
    ) -> Result<()> {
        // Find the first environment which has the given binding
        let stack = &self.environment_stack.borrow();
        let find_environment_with_binding = || -> Result<Option<&Environment>> {
            for env in stack.iter().rev() {
                if env.borrow().has_binding(name, context)? {
                    return Ok(Some(env));
                }
            }
            Ok(None)
        };
        let env = find_environment_with_binding()?;

        let env = if let Some(env) = env {
            env
        } else {
            // global_env doesn't need has_binding to be satisfied in non strict mode
            stack.get(0).expect("Environment stack underflow")
        };
        let res = env
            .borrow_mut()
            .set_mutable_binding(name, value, strict, context);
        res
    }

    pub fn initialize_binding(&self, name: &str, value: Value, context: &Context) -> Result<()> {
        // Find the first environment which has the given binding
        let stack = &self.environment_stack.borrow();
        let find_environment_with_binding = || -> Result<Option<&Environment>> {
            for env in stack.iter().rev() {
                if env.borrow().has_binding(name, context)? {
                    return Ok(Some(env));
                }
            }
            Ok(None)
        };
        let env = find_environment_with_binding()?;
        let env = if let Some(env) = env {
            env
        } else {
            // global_env doesn't need has_binding to be satisfied in non strict mode
            stack.get(0).expect("Environment stack underflow")
        };
        let res = env.borrow_mut().initialize_binding(name, value, context);
        res
    }

    /// When need to clone an environment (linking it with another environment)
    /// cloning is more suited. The GC will remove the env once nothing is linking to it anymore
    pub fn get_current_environment(&self) -> Environment {
        self.environment_stack
            .borrow()
            .back()
            .cloned()
            .expect("Could not get mutable reference to back object")
    }

    pub fn has_binding(&self, name: &str, context: &Context) -> Result<bool> {
        for e in self.environment_stack.borrow().iter().rev() {
            if e.borrow().has_binding(name, context)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn get_binding_value(&self, name: &str, context: &Context) -> Result<Value> {
        let stack = &self.environment_stack.borrow();
        let find_environment_with_binding = || -> Result<Option<&Environment>> {
            for env in stack.iter().rev() {
                if env.borrow().has_binding(name, context)? {
                    return Ok(Some(env));
                }
            }
            Ok(None)
        };
        let env = find_environment_with_binding()?;
        if let Some(env) = env {
            env.borrow().get_binding_value(name, false, context)
        } else {
            context.throw_reference_error(format!("{} is not defined", name))
        }
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
    context: &Context,
) -> Environment {
    let mut func_env = FunctionEnvironmentRecord {
        env_rec: FxHashMap::default(),
        function: f,
        this_binding_status: binding_status,
        home_object: Value::undefined(),
        new_target,
        outer_env: outer, // this will come from Environment set as a private property of F - https://tc39.es/ecma262/#sec-ecmascript-function-objects
        this_value: Value::undefined(),
    };
    // If a `this` value has been passed, bind it to the environment
    if let Some(v) = this {
        func_env.bind_this_value(v, context).unwrap();
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
