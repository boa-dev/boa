use super::{Executable, Interpreter, InterpreterState};
use crate::{
    builtins::Value,
    syntax::ast::node::{ConditionalOp, Continue, If},
    Result,
};
use std::borrow::Borrow;

#[cfg(test)]
mod tests;

impl Executable for If {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        Ok(if self.cond().run(interpreter)?.borrow().to_boolean() {
            self.body().run(interpreter)?
        } else if let Some(ref else_e) = self.else_node() {
            else_e.run(interpreter)?
        } else {
            Value::undefined()
        })
    }
}

impl Executable for ConditionalOp {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        if self.cond().run(interpreter)?.to_boolean() {
            self.if_true().run(interpreter)
        } else {
            self.if_false().run(interpreter)
        }
    }
}

impl Executable for Continue {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        interpreter.set_current_state(InterpreterState::Continue(self.label().map(String::from)));

        Ok(Value::undefined())
    }
}
