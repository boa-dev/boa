//! # Lexical Environment
//!
//! https://tc39.github.io/ecma262/#sec-lexical-environment-operations
//!
//! The following operations are used to operate upon lexical environments
//! This is the entrypoint to lexical environments.
//!

use crate::environment::declerative_environment_record::DeclerativeEnvironmentRecord;
use crate::environment::function_environment_record::{BindingStatus, FunctionEnvironmentRecord};
use crate::environment::global_environment_record::GlobalEnvironmentRecord;
use crate::environment::object_environment_record::ObjectEnvironmentRecord;
use crate::js::value::{Value, ValueData};
use gc::Gc;
use std::collections::hash_map::HashMap;
use std::collections::{HashSet, VecDeque};
use std::debug_assert;

pub type Environment = Gc<EnvironmentData>;

#[derive(Trace, Finalize, Debug, Clone)]
pub enum EnvironmentData {
    Declerative(DeclerativeEnvironmentRecord),
    Function(FunctionEnvironmentRecord),
    Global(GlobalEnvironmentRecord),
    Object(ObjectEnvironmentRecord),
}

pub struct LexicalEnvironment {
    environment_stack: VecDeque<Environment>,
}

impl LexicalEnvironment {
    pub fn new(global: Value) -> LexicalEnvironment {
        let global_env = new_global_environment(global.clone(), global.clone());
        let lexical_env = LexicalEnvironment {
            environment_stack: VecDeque::new(),
        };

        lexical_env
    }

    pub fn push(&mut self, env: Environment) {
        self.environment_stack.push_back(env);
    }

    pub fn pop(&mut self, env: Environment) {
        self.environment_stack.pop_back();
    }

    pub fn get_current_environment(&self) -> &Environment {
        &self
            .environment_stack
            .get(self.environment_stack.len() - 1)
            .unwrap()
    }

    pub fn get_current_environent_type(&self) -> EnvironmentType {
        self.environment_stack
            .get(self.environment_stack.len() - 1)
            .unwrap()
            .get_environment_type()
    }

    pub fn get_binding_value(&self, name: String, strict: bool) -> Option<Value> {
        for &env in self.environment_stack.iter().rev() {
            // Environment found
            let exists = env.has_binding(&name);
            // Binding found in this environment
            if exists {
                return Some(env.get_binding_value(name, strict));
            }
        }

        None
    }
}

/// Recursively search the tree of environments to find the correct binding, otherwise return undefined
pub fn get_identifier_reference(lex: Option<&Environment>, name: String, strict: bool) -> Value {
    return match lex {
        None => Gc::new(ValueData::Undefined),
        Some(env) => {
            // Environment found
            let exists = env.has_binding(&name);
            // Binding found in this environment
            if exists {
                return env.get_binding_value(name, strict);
            }
            // Env found but no binding, it may be in the next env up
            let outer: Option<&Environment> = env.get_outer_environment();
            get_identifier_reference(outer, name, strict)
        }
    };
}

pub fn new_declerative_environment(env: Option<Environment>) -> Box<DeclerativeEnvironmentRecord> {
    Box::new(DeclerativeEnvironmentRecord {
        env_rec: HashMap::new(),
        outer_env: env,
    })
}

pub fn new_function_environment(
    F: Value,
    new_target: Value,
    outer: Option<Environment>,
) -> Box<FunctionEnvironmentRecord> {
    debug_assert!(F.is_function());
    debug_assert!(new_target.is_object() || new_target.is_undefined());
    Box::new(FunctionEnvironmentRecord {
        env_rec: HashMap::new(),
        function_object: F.clone(),
        this_binding_status: BindingStatus::Uninitialized, // hardcoding to unitialized for now until short functions are properly supported
        home_object: Gc::new(ValueData::Undefined),
        new_target: new_target,
        outer_env: outer, // this will come from Environment set as a private property of F - https://tc39.github.io/ecma262/#sec-ecmascript-function-objects
        this_value: Gc::new(ValueData::Undefined), // TODO: this_value should start as an Option as its not always there to begin with
    })
}

pub fn new_object_environment(
    object: Value,
    environment: Option<Environment>,
) -> Box<ObjectEnvironmentRecord> {
    Box::new(ObjectEnvironmentRecord {
        bindings: object,
        outer_env: environment,
        /// Object Environment Records created for with statements (13.11)
        /// can provide their binding object as an implicit this value for use in function calls.
        /// The capability is controlled by a withEnvironment Boolean value that is associated
        /// with each object Environment Record. By default, the value of withEnvironment is false
        /// for any object Environment Record.
        with_environment: false,
    })
}

pub fn new_global_environment(global: Value, this_value: Value) -> Box<GlobalEnvironmentRecord> {
    let obj_rec = new_object_environment(global, None);
    let dcl_rec = new_declerative_environment(None);
    Box::new(GlobalEnvironmentRecord {
        object_record: obj_rec,
        global_this_binding: this_value,
        declerative_record: dcl_rec,
        var_names: HashSet::new(),
    })
}
