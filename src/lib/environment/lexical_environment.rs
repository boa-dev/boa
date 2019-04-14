//! # Lexical Environment
//!
//! https://tc39.github.io/ecma262/#sec-lexical-environment-operations
//!
//! The following operations are used to operate upon lexical environments
//! This is the entrypoint to lexical environments.
//!

use crate::environment::declerative_environment_record::DeclerativeEnvironmentRecord;
use crate::environment::environment_record::EnvironmentRecordTrait;
use crate::environment::function_environment_record::{BindingStatus, FunctionEnvironmentRecord};
use crate::js::value::{Value, ValueData};
use gc::Gc;
use std::collections::hash_map::HashMap;
use std::debug_assert;

/// Recursively search the tree of environments to find the correct binding, otherwise return undefined
fn get_identifier_reference(
    lex: Option<&Box<EnvironmentRecordTrait>>,
    name: String,
    strict: bool,
) -> Value {
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
            let outer: Option<&Box<EnvironmentRecordTrait>> = env.get_outer_environment();
            get_identifier_reference(outer, name, strict)
        }
    };
}

fn new_declerative_environment(env: Box<EnvironmentRecordTrait>) -> Box<EnvironmentRecordTrait> {
    Box::new(DeclerativeEnvironmentRecord {
        env_rec: HashMap::new(),
        outer_env: env,
    })
}

fn new_function_environment(
    F: Value,
    new_target: Value,
    outer: Box<EnvironmentRecordTrait>,
) -> Box<EnvironmentRecordTrait> {
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
