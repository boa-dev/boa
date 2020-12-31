//! Environment handling, lexical, object, function and declaritive records

pub mod declarative_environment_record;
pub mod environment_record_trait;
pub mod function_environment_record;
pub mod global_environment_record;
pub mod lexical_environment;
pub mod object_environment_record;

#[derive(Debug)]
pub enum ErrorKind {
    ReferenceError(Box<str>),
    TypeError(Box<str>),
}

use crate::value::Value;
use crate::Context;

impl ErrorKind {
    pub fn to_error(&self, ctx: &mut Context) -> Value {
        match self {
            ErrorKind::ReferenceError(msg) => ctx.construct_reference_error(msg.clone()),
            ErrorKind::TypeError(msg) => ctx.construct_type_error(msg.clone()),
        }
    }

    pub fn new_reference_error<M>(msg: M) -> Self
    where
        M: Into<Box<str>>,
    {
        Self::ReferenceError(msg.into())
    }

    pub fn new_type_error<M>(msg: M) -> Self
    where
        M: Into<Box<str>>,
    {
        Self::TypeError(msg.into())
    }
}
