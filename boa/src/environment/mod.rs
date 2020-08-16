//! Environment handling, lexical, object, function and declaritive records

pub mod declarative_environment_record;
pub mod environment_record_trait;
pub mod function_environment_record;
pub mod global_environment_record;
pub mod lexical_environment;
pub mod object_environment_record;

#[derive(Debug)]
pub enum ErrorKind {
    ReferenceError(String),
    TypeError(String),
}

use crate::builtins::value::Value;
use crate::exec::Interpreter;

impl ErrorKind {
    pub fn to_error(&self, interpreter: &mut Interpreter) -> Value {
        match self {
            ErrorKind::ReferenceError(msg) => interpreter.construct_reference_error(msg),
            ErrorKind::TypeError(msg) => interpreter.construct_type_error(msg),
        }
    }
}
