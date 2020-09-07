use super::{Executable, Interpreter, InterpreterState};
use crate::{syntax::ast::node::Switch, Result, Value};

#[cfg(test)]
mod tests;

impl Executable for Switch {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
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
                        // TODO, break to a label.
                        // Break statement encountered so therefore end switch statement.
                        interpreter.set_current_state(InterpreterState::Executing);
                        break;
                    }
                    InterpreterState::Continue(_label) => {
                        // TODO, continue to a label.
                        break;
                    }
                    InterpreterState::Executing => {
                        // Continuing execution / falling through to next case statement(s).
                        fall_through = true;
                    }
                }
            }
        }

        if !matched {
            if let Some(default) = self.default() {
                interpreter.set_current_state(InterpreterState::Executing);
                for (i, item) in default.iter().enumerate() {
                    let val = item.run(interpreter)?;
                    match interpreter.get_current_state() {
                        InterpreterState::Return => {
                            // Early return.
                            result = val;
                            break;
                        }
                        InterpreterState::Break(_label) => {
                            // TODO, break to a label.

                            // Early break.
                            break;
                        }
                        _ => {
                            // Continue execution
                        }
                    }
                    if i == default.len() - 1 {
                        result = val;
                    }
                }
            }
        }

        Ok(result)
    }
}
