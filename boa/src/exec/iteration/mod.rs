//! Iteration node execution.

use super::{Executable, Interpreter, InterpreterState};
use crate::{
    builtins::value::{ResultValue, Value},
    environment::lexical_environment::new_declarative_environment,
    syntax::ast::node::{DoWhileLoop, ForLoop, WhileLoop},
    BoaProfiler,
};
use std::borrow::Borrow;

#[cfg(test)]
mod tests;

impl Executable for ForLoop {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        // Create the block environment.
        let _timer = BoaProfiler::global().start_event("ForLoop", "exec");
        {
            let env = &mut interpreter.realm_mut().environment;
            env.push(new_declarative_environment(Some(
                env.get_current_environment_ref().clone(),
            )));
        }

        if let Some(init) = self.init() {
            init.run(interpreter)?;
        }

        while self
            .condition()
            .map(|cond| cond.run(interpreter).map(|v| v.to_boolean()))
            .transpose()?
            .unwrap_or(true)
        {
            let result = self.body().run(interpreter)?;

            match interpreter.get_current_state() {
                InterpreterState::Break(_label) => {
                    // TODO break to label.

                    // Loops 'consume' breaks.
                    interpreter.set_current_state(InterpreterState::Executing);
                    break;
                }
                InterpreterState::Return => {
                    return Ok(result);
                }
                _ => {
                    // Continue execution.
                }
            }

            if let Some(final_expr) = self.final_expr() {
                final_expr.run(interpreter)?;
            }
        }

        // pop the block env
        let _ = interpreter.realm_mut().environment.pop();

        Ok(Value::undefined())
    }
}

impl Executable for WhileLoop {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let mut result = Value::undefined();
        while self.cond().run(interpreter)?.borrow().to_boolean() {
            result = self.expr().run(interpreter)?;
            match interpreter.get_current_state() {
                InterpreterState::Break(_label) => {
                    // TODO break to label.

                    // Loops 'consume' breaks.
                    interpreter.set_current_state(InterpreterState::Executing);
                    break;
                }
                InterpreterState::Return => {
                    return Ok(result);
                }
                _ => {
                    // Continue execution.
                }
            }
        }
        Ok(result)
    }
}

impl Executable for DoWhileLoop {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let mut result = self.body().run(interpreter)?;
        match interpreter.get_current_state() {
            InterpreterState::Break(_label) => {
                // TODO break to label.

                // Loops 'consume' breaks.
                interpreter.set_current_state(InterpreterState::Executing);
                return Ok(result);
            }
            InterpreterState::Return => {
                return Ok(result);
            }
            _ => {
                // Continue execution.
            }
        }

        while self.cond().run(interpreter)?.borrow().to_boolean() {
            result = self.body().run(interpreter)?;
            match interpreter.get_current_state() {
                InterpreterState::Break(_label) => {
                    // TODO break to label.

                    // Loops 'consume' breaks.
                    interpreter.set_current_state(InterpreterState::Executing);
                    break;
                }
                InterpreterState::Return => {
                    return Ok(result);
                }
                _ => {
                    // Continue execution.
                }
            }
        }
        Ok(result)
    }
}
