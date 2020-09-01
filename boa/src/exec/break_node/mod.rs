use super::{Executable, Interpreter, InterpreterState};
use crate::{
    syntax::ast::node::{Break, Continue},
    Result, Value,
};

#[cfg(test)]
mod tests;

impl Executable for Break {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        interpreter.set_current_state(InterpreterState::Break(self.label().map(String::from)));

        Ok(Value::undefined())
    }
}

impl Executable for Continue {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        interpreter.set_current_state(InterpreterState::Continue(self.label().map(String::from)));

        Ok(Value::undefined())
    }
}
