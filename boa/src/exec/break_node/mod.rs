use super::{Executable, Interpreter};
use crate::{
    builtins::value::{ResultValue, Value},
    syntax::ast::node::Break,
};

impl Executable for Break {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        interpreter.is_break = true;
        Ok(Value::undefined())
    }
}
