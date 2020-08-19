use super::*;
use crate::{
    exec::Executable,
    syntax::ast::{
        node::{Call, Identifier, New},
        Const,
    },
};

impl Interpreter {
    /// Constructs a `RangeError` with the specified message.
    pub fn construct_range_error<M>(&mut self, message: M) -> Value
    where
        M: Into<String>,
    {
        // Runs a `new RangeError(message)`.
        New::from(Call::new(
            Identifier::from("RangeError"),
            vec![Const::from(message.into()).into()],
        ))
        .run(self)
        .expect_err("RangeError should always throw")
    }

    /// Throws a `RangeError` with the specified message.
    pub fn throw_range_error<M>(&mut self, message: M) -> Result<Value>
    where
        M: Into<String>,
    {
        Err(self.construct_range_error(message))
    }

    /// Constructs a `TypeError` with the specified message.
    pub fn construct_type_error<M>(&mut self, message: M) -> Value
    where
        M: Into<String>,
    {
        // Runs a `new TypeError(message)`.
        New::from(Call::new(
            Identifier::from("TypeError"),
            vec![Const::from(message.into()).into()],
        ))
        .run(self)
        .expect_err("TypeError should always throw")
    }

    /// Throws a `TypeError` with the specified message.
    pub fn throw_type_error<M>(&mut self, message: M) -> Result<Value>
    where
        M: Into<String>,
    {
        Err(self.construct_type_error(message))
    }

    /// Constructs a `ReferenceError` with the specified message.
    pub fn construct_reference_error<M>(&mut self, message: M) -> Value
    where
        M: Into<String>,
    {
        New::from(Call::new(
            Identifier::from("ReferenceError"),
            vec![Const::from(message.into() + " is not defined").into()],
        ))
        .run(self)
        .expect_err("ReferenceError should always throw")
    }

    /// Throws a `ReferenceError` with the specified message.
    pub fn throw_reference_error<M>(&mut self, message: M) -> Result<Value>
    where
        M: Into<String>,
    {
        Err(self.construct_reference_error(message))
    }

    /// Constructs a `SyntaxError` with the specified message.
    pub fn construct_syntax_error<M>(&mut self, message: M) -> Value
    where
        M: Into<String>,
    {
        New::from(Call::new(
            Identifier::from("SyntaxError"),
            vec![Const::from(message.into()).into()],
        ))
        .run(self)
        .expect_err("SyntaxError should always throw")
    }

    /// Throws a `SyntaxError` with the specified message.
    pub fn throw_syntax_error<M>(&mut self, message: M) -> Result<Value>
    where
        M: Into<String>,
    {
        Err(self.construct_syntax_error(message))
    }
}
