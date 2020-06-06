use super::*;
use crate::{
    exec::Executable,
    syntax::ast::{
        node::{Call, Identifier, New},
        Const,
    },
};

impl Interpreter {
    /// Throws a `RangeError` with the specified message.
    pub fn throw_range_error<M>(&mut self, message: M) -> ResultValue
    where
        M: Into<String>,
    {
        // Runs a `new RangeError(message)`.
        New::from(Call::new(
            Identifier::from("RangeError"),
            vec![Const::from(message.into()).into()],
        ))
        .run(self)
    }

    /// Throws a `TypeError` with the specified message.
    pub fn throw_type_error<M>(&mut self, message: M) -> ResultValue
    where
        M: Into<String>,
    {
        // Runs a `new TypeError(message)`.
        New::from(Call::new(
            Identifier::from("TypeError"),
            vec![Const::from(message.into()).into()],
        ))
        .run(self)
    }
}
