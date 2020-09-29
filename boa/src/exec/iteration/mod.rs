//! Iteration node execution.

use super::{Context, Executable, InterpreterState};
use crate::{
    builtins::iterable::get_iterator,
    environment::lexical_environment::{new_declarative_environment, VariableScope},
    syntax::ast::{
        node::{DoWhileLoop, ForLoop, ForOfLoop, WhileLoop},
        Node,
    },
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
        let iterator = get_iterator(interpreter, iterable)?;
        let mut result = Value::undefined();

        loop {
            {
                let env = &mut interpreter.realm_mut().environment;
                env.push(new_declarative_environment(Some(
                    env.get_current_environment_ref().clone(),
                )));
            }
            let iterator_result = iterator.next(interpreter)?;
            if iterator_result.is_done() {
                break;
            }
            let next_result = iterator_result.value();

            match self.variable() {
                Node::Identifier(ref name) => {
                    let environment = &mut interpreter.realm_mut().environment;

                    if environment.has_binding(name.as_ref()) {
                        // Binding already exists
                        environment.set_mutable_binding(name.as_ref(), next_result.clone(), true);
                    } else {
                        environment.create_mutable_binding(
                            name.as_ref().to_owned(),
                            true,
                            VariableScope::Function,
                        );
                        environment.initialize_binding(name.as_ref(), next_result.clone());
                    }
                }
                Node::VarDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        let environment = &mut interpreter.realm_mut().environment;

                        if var.init().is_some() {
                            return interpreter.throw_syntax_error("a declaration in the head of a for-of loop can't have an initializer");
                        }

                        if environment.has_binding(var.name()) {
                            environment.set_mutable_binding(var.name(), next_result, true);
                        } else {
                            environment.create_mutable_binding(
                                var.name().to_owned(),
                                false,
                                VariableScope::Function,
                            );
                            environment.initialize_binding(var.name(), next_result);
                        }
                    }
                    _ => {
                        return interpreter.throw_syntax_error(
                            "only one variable can be declared in the head of a for-of loop",
                        )
                    }
                },
                Node::LetDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        let environment = &mut interpreter.realm_mut().environment;

                        if var.init().is_some() {
                            return interpreter.throw_syntax_error("a declaration in the head of a for-of loop can't have an initializer");
                        }

                        environment.create_mutable_binding(
                            var.name().to_owned(),
                            false,
                            VariableScope::Block,
                        );
                        environment.initialize_binding(var.name(), next_result);
                    }
                    _ => {
                        return interpreter.throw_syntax_error(
                            "only one variable can be declared in the head of a for-of loop",
                        )
                    }
                },
                Node::ConstDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        let environment = &mut interpreter.realm_mut().environment;

                        if var.init().is_some() {
                            return interpreter.throw_syntax_error("a declaration in the head of a for-of loop can't have an initializer");
                        }

                        environment.create_immutable_binding(
                            var.name().to_owned(),
                            false,
                            VariableScope::Block,
                        );
                        environment.initialize_binding(var.name(), next_result);
                    }
                    _ => {
                        return interpreter.throw_syntax_error(
                            "only one variable can be declared in the head of a for-of loop",
                        )
                    }
                },
                Node::Assign(_) => {
                    return interpreter.throw_syntax_error(
                        "a declaration in the head of a for-of loop can't have an initializer",
                    );
                }
                _ => {
                    return interpreter
                        .throw_syntax_error("unknown left hand side in head of for-of loop")
                }
            }

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
                InterpreterState::Return => return Ok(result),
                InterpreterState::Executing => {
                    // Continue execution.
                }
            }
            let _ = interpreter.realm_mut().environment.pop();
        }
        Ok(result)
    }
}
