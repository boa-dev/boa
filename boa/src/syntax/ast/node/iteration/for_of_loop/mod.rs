use crate::{
    builtins::iterable::get_iterator,
    environment::lexical_environment::{new_declarative_environment, VariableScope},
    exec::{Executable, InterpreterState},
    syntax::ast::node::Node,
    BoaProfiler, Context, Result, Value,
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ForOfLoop {
    variable: Box<Node>,
    iterable: Box<Node>,
    body: Box<Node>,
}

impl ForOfLoop {
    pub fn new<V, I, B>(variable: V, iterable: I, body: B) -> Self
    where
        V: Into<Node>,
        I: Into<Node>,
        B: Into<Node>,
    {
        Self {
            variable: Box::new(variable.into()),
            iterable: Box::new(iterable.into()),
            body: Box::new(body.into()),
        }
    }

    pub fn variable(&self) -> &Node {
        &self.variable
    }

    pub fn iterable(&self) -> &Node {
        &self.iterable
    }

    pub fn body(&self) -> &Node {
        &self.body
    }

    pub fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        write!(f, "for ({} of {}) {{", self.variable, self.iterable)?;
        self.body().display(f, indentation + 1)?;
        f.write_str("}")
    }
}

impl fmt::Display for ForOfLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<ForOfLoop> for Node {
    fn from(for_of: ForOfLoop) -> Node {
        Self::ForOfLoop(for_of)
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
