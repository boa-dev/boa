//! Iteration node execution.

use super::{Context, Executable, InterpreterState};
use crate::{
    environment::lexical_environment::new_declarative_environment,
    syntax::ast::node::{DoWhileLoop, ForLoop, WhileLoop},
    BoaProfiler, Result, Value,
};

#[cfg(test)]
mod tests;

impl Executable for ForLoop {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
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

            match interpreter.executor().get_current_state() {
                InterpreterState::Break(label) => {
                    // If a label is set we want to break the current block and still keep state as Break if the label is a block above
                    if let Some(stmt_label) = &self.label {
                        if let Some(brk_label) = label {
                            // We have a label, but not for the current statement
                            // break without resetting to executings
                            if stmt_label.to_string() != *brk_label {
                                break;
                            } else {
                                interpreter.set_current_state(InterpreterState::Executing);
                                break;
                            }
                        }
                    }

                    // Loops 'consume' breaks.
                    interpreter
                        .executor()
                        .set_current_state(InterpreterState::Executing);
                    break;
                    //         }
                    //     }
                    // }

                    interpreter.set_current_state(InterpreterState::Executing);
                    break;
                }
                InterpreterState::Continue(_label) => {
                    // TODO continue to label.
                    interpreter
                        .executor()
                        .set_current_state(InterpreterState::Executing);
                    // after breaking out of the block, continue execution of the loop
                }
                InterpreterState::Return => {
                    return Ok(result);
                }
                InterpreterState::Executing => {
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
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let mut result = Value::undefined();
        while self.cond().run(interpreter)?.to_boolean() {
            result = self.expr().run(interpreter)?;
            match interpreter.executor().get_current_state() {
                InterpreterState::Break(_label) => {
                    // TODO break to label.

                    // Loops 'consume' breaks.
                    interpreter
                        .executor()
                        .set_current_state(InterpreterState::Executing);
                    break;
                }
                InterpreterState::Continue(_label) => {
                    // TODO continue to label.
                    interpreter
                        .executor()
                        .set_current_state(InterpreterState::Executing);
                    // after breaking out of the block, continue execution of the loop
                }
                InterpreterState::Return => {
                    return Ok(result);
                }
                InterpreterState::Executing => {
                    // Continue execution.
                }
            }
        }
        Ok(result)
    }
}

impl Executable for DoWhileLoop {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let mut result = self.body().run(interpreter)?;
        match interpreter.executor().get_current_state() {
            InterpreterState::Break(_label) => {
                // TODO break to label.

                // Loops 'consume' breaks.
                interpreter
                    .executor()
                    .set_current_state(InterpreterState::Executing);
                return Ok(result);
            }
            InterpreterState::Continue(_label) => {
                // TODO continue to label;
                interpreter
                    .executor()
                    .set_current_state(InterpreterState::Executing);
                // after breaking out of the block, continue execution of the loop
            }
            InterpreterState::Return => {
                return Ok(result);
            }
            InterpreterState::Executing => {
                // Continue execution.
            }
        }

        while self.cond().run(interpreter)?.to_boolean() {
            result = self.body().run(interpreter)?;
            match interpreter.executor().get_current_state() {
                InterpreterState::Break(_label) => {
                    // TODO break to label.

                    // Loops 'consume' breaks.
                    interpreter
                        .executor()
                        .set_current_state(InterpreterState::Executing);
                    break;
                }
                InterpreterState::Continue(_label) => {
                    // TODO continue to label.
                    interpreter
                        .executor()
                        .set_current_state(InterpreterState::Executing);
                    // after breaking out of the block, continue execution of the loop
                }
                InterpreterState::Return => {
                    return Ok(result);
                }
                InterpreterState::Executing => {
                    // Continue execution.
                }
            }
        }
        Ok(result)
    }
}
