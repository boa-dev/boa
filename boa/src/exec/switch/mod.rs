use super::{Executable, Interpreter, InterpreterState};
use crate::{builtins::value::Value, syntax::ast::node::Switch, Result};

#[cfg(test)]
mod tests;

impl Executable for Switch {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let default = self.default();
        let val = self.val().run(interpreter)?;
        let mut result = Value::null();
        let mut matched = false;
        interpreter.set_current_state(InterpreterState::Executing);

        // If a case block does not end with a break statement then subsequent cases will be run without
        // checking their conditions until a break is encountered.
        let mut fall_through: bool = false;

        for case in self.cases().iter() {
            let cond = case.condition();
            let block = case.body();
            if fall_through || val.strict_equals(&cond.run(interpreter)?) {
                matched = true;
                let result = block.run(interpreter)?;
                match interpreter.get_current_state() {
                    InterpreterState::Return => {
                        // Early return.
                        return Ok(result);
                    }
                    InterpreterState::Break(_label) => {
                        // Break statement encountered so therefore end switch statement.
                        interpreter.set_current_state(InterpreterState::Executing);
                        break;
                    }
                    InterpreterState::Continue(_label) => {
                        // TODO continue to label.

                        // Relinquish control to containing loop.
                        break;
                    }
                    _ => {
                        // Continuing execution / falling through to next case statement(s).
                        fall_through = true;
                    }
                }
            }
        }
        if !matched {
            if let Some(default) = default {
                result = default.run(interpreter)?;
            }
        }
        Ok(result)
    }
}
