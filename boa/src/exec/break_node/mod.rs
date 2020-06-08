use super::{Executable, Interpreter, InterpreterState};
use crate::{
    builtins::value::{ResultValue, Value},
    syntax::ast::node::Break,
};

#[cfg(test)]
mod tests;

impl Executable for Break {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        interpreter.set_current_state(InterpreterState::Break(self.label().map(String::from)));

        Ok(Value::undefined())
    }
}
