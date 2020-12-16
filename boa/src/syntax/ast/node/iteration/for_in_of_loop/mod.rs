use crate::{
    builtins::iterable::get_iterator,
    builtins::ForInIterator,
    environment::lexical_environment::{new_declarative_environment, VariableScope},
    exec::{Executable, InterpreterState},
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
    BoaProfiler, Context, Result, Value,
};
use std::fmt;

use crate::builtins::iterable::IteratorRecord;
#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum IterationKind {
    Iterate,
    Enumerate,
}

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ForInOfLoop {
    variable: Box<Node>,
    expr: Box<Node>,
    body: Box<Node>,
    kind: IterationKind,
}

impl ForInOfLoop {
    pub fn new<V, I, B>(variable: V, expr: I, body: B, kind: IterationKind) -> Self
    where
        V: Into<Node>,
        I: Into<Node>,
        B: Into<Node>,
    {
        Self {
            variable: Box::new(variable.into()),
            expr: Box::new(expr.into()),
            body: Box::new(body.into()),
            kind,
        }
    }

    pub fn variable(&self) -> &Node {
        &self.variable
    }

    pub fn expr(&self) -> &Node {
        &self.expr
    }

    pub fn body(&self) -> &Node {
        &self.body
    }

    pub fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        write!(
            f,
            "for ({} {} {}) {{",
            self.variable,
            match self.kind {
                IterationKind::Iterate => "of",
                IterationKind::Enumerate => "in",
            },
            self.expr,
        )?;
        self.body().display(f, indentation + 1)?;
        f.write_str("}")
    }
}

impl fmt::Display for ForInOfLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<ForInOfLoop> for Node {
    fn from(for_in_of: ForInOfLoop) -> Node {
        Self::ForInOfLoop(for_in_of)
    }
}

impl Executable for ForInOfLoop {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("ForInOf", "exec");
        let object = self.expr().run(context)?;
        let mut result = Value::undefined();
        let iterator = match self.kind {
            IterationKind::Iterate => get_iterator(context, object)?,
            IterationKind::Enumerate => {
                if object.is_null_or_undefined() {
                    return Ok(result);
                }
                let object = object.to_object(context)?;
                let iterator = ForInIterator::create_for_in_iterator(context, Value::from(object))?;
                let next_function = iterator
                    .get_property("next")
                    .map(|p| p.as_data_descriptor().unwrap().value())
                    .ok_or_else(|| {
                        context.construct_type_error("Could not find property `next`")
                    })?;
                IteratorRecord::new(iterator, next_function)
            }
        };

        loop {
            {
                let env = &mut context.realm_mut().environment;
                env.push(new_declarative_environment(Some(
                    env.get_current_environment_ref().clone(),
                )));
            }
            let iterator_result = iterator.next(context)?;
            if iterator_result.is_done() {
                break;
            }
            let next_result = iterator_result.value();

            match self.variable() {
                Node::Identifier(ref name) => {
                    let environment = &mut context.realm_mut().environment;

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
                        let environment = &mut context.realm_mut().environment;

                        if var.init().is_some() {
                            return context.throw_syntax_error("a declaration in the head of a for-of loop can't have an initializer");
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
                    _ => return context.throw_syntax_error(
                        "only one variable can be declared in the head of a for-in or for-of loop",
                    ),
                },
                Node::LetDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        let environment = &mut context.realm_mut().environment;

                        if var.init().is_some() {
                            return context.throw_syntax_error("a declaration in the head of a for-of loop can't have an initializer");
                        }

                        environment.create_mutable_binding(
                            var.name().to_owned(),
                            false,
                            VariableScope::Block,
                        );
                        environment.initialize_binding(var.name(), next_result);
                    }
                    _ => {
                        return context.throw_syntax_error(
                            "only one variable can be declared in the head of a for-of loop",
                        )
                    }
                },
                Node::ConstDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        let environment = &mut context.realm_mut().environment;

                        if var.init().is_some() {
                            return context.throw_syntax_error("a declaration in the head of a for-of loop can't have an initializer");
                        }

                        environment.create_immutable_binding(
                            var.name().to_owned(),
                            false,
                            VariableScope::Block,
                        );
                        environment.initialize_binding(var.name(), next_result);
                    }
                    _ => {
                        return context.throw_syntax_error(
                            "only one variable can be declared in the head of a for-of loop",
                        )
                    }
                },
                Node::Assign(_) => {
                    return context.throw_syntax_error(
                        "a declaration in the head of a for-of loop can't have an initializer",
                    );
                }
                _ => {
                    return context
                        .throw_syntax_error("unknown left hand side in head of for-of loop")
                }
            }

            result = self.body().run(context)?;
            match context.executor().get_current_state() {
                InterpreterState::Break(_label) => {
                    // TODO break to label.

                    // Loops 'consume' breaks.
                    context
                        .executor()
                        .set_current_state(InterpreterState::Executing);
                    break;
                }
                InterpreterState::Continue(_label) => {
                    // TODO continue to label.
                    context
                        .executor()
                        .set_current_state(InterpreterState::Executing);
                    // after breaking out of the block, continue execution of the loop
                }
                InterpreterState::Return => return Ok(result),
                InterpreterState::Executing => {
                    // Continue execution.
                }
            }
            let _ = context.realm_mut().environment.pop();
        }
        Ok(result)
    }
}
