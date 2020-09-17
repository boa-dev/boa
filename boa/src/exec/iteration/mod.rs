//! Iteration node execution.

use super::{Context, Executable, InterpreterState};
use crate::{
    environment::lexical_environment::new_declarative_environment,
    syntax::ast::node::{DoWhileLoop, ForLoop, ForOfLoop, WhileLoop},
    BoaProfiler, Result, Value,
};

#[cfg(test)]
mod tests;

// Checking labels for break and continue is the same operation for `ForLoop`, `While` and `DoWhile`
macro_rules! handle_state_with_labels {
    ($self:ident, $label:ident, $interpreter:ident, $state:tt) => {{
        if let Some(brk_label) = $label {
            if let Some(stmt_label) = $self.label() {
                // Break from where we are, keeping "continue" set as the state
                if stmt_label != brk_label.as_ref() {
                    break;
                }
            } else {
                // if a label is set but the current block has no label, break
                break;
            }
        }

        $interpreter
            .executor()
            .set_current_state(InterpreterState::Executing);
    }};
}

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
                    handle_state_with_labels!(self, label, interpreter, break);
                    break;
                }
                InterpreterState::Continue(label) => {
                    handle_state_with_labels!(self, label, interpreter, continue);
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
                InterpreterState::Break(label) => {
                    handle_state_with_labels!(self, label, interpreter, break);
                    break;
                }
                InterpreterState::Continue(label) => {
                    handle_state_with_labels!(self, label, interpreter, continue)
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

impl Executable for ForOfLoop {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("ForOf", "exec");
        let iterable = self.iterable().run(interpreter)?;
        let iterator_function = iterable
            .get_property(
                interpreter
                    .get_well_known_symbol("iterator")
                    .ok_or_else(|| interpreter.construct_type_error("Not an iterable"))?,
            )
            .and_then(|mut p| p.value.take())
            .ok_or_else(|| interpreter.construct_type_error("Not an iterable"))?;
        let iterator_object = interpreter.call(&iterator_function, &iterable, &[])?;
        {
            let env = &mut interpreter.realm_mut().environment;
            env.push(new_declarative_environment(Some(
                env.get_current_environment_ref().clone(),
            )));
        }
        //let variable = self.variable().run(interpreter)?;
        let next_function = iterator_object
            .get_property("next")
            .and_then(|mut p| p.value.take())
            .ok_or_else(|| interpreter.construct_type_error("Could not find property `next`"))?;
        let mut result = Value::undefined();

        self.variable().run(interpreter)?;
        loop {
            let next = interpreter.call(&next_function, &iterator_object, &[])?;
            let done = next
                .get_property("done")
                .and_then(|mut p| p.value.take())
                .and_then(|v| v.as_boolean())
                .ok_or_else(|| {
                    interpreter.construct_type_error("Could not find property `done`")
                })?;
            if done {
                break;
            }
            let next_result = next
                .get_property("value")
                .and_then(|mut p| p.value.take())
                .ok_or_else(|| {
                    interpreter.construct_type_error("Could not find property `value`")
                })?;
            interpreter.set_value(self.variable(), next_result)?;
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
                    return interpreter.throw_syntax_error("return not in function")
                }
                InterpreterState::Executing => {
                    // Continue execution.
                }
            }
        }
        let _ = interpreter.realm_mut().environment.pop();
        Ok(result)
    }
}
