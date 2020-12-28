use crate::{
    builtins::{iterable::IteratorRecord, ForInIterator},
    environment::lexical_environment::{new_declarative_environment, VariableScope},
    exec::{Executable, InterpreterState},
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
    BoaProfiler, Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ForInLoop {
    variable: Box<Node>,
    expr: Box<Node>,
    body: Box<Node>,
    label: Option<Box<str>>,
}

impl ForInLoop {
    pub fn new<V, I, B>(variable: V, expr: I, body: B) -> Self
    where
        V: Into<Node>,
        I: Into<Node>,
        B: Into<Node>,
    {
        Self {
            variable: Box::new(variable.into()),
            expr: Box::new(expr.into()),
            body: Box::new(body.into()),
            label: None,
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

    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Box::as_ref)
    }

    pub fn set_label(&mut self, label: Box<str>) {
        self.label = Some(label);
    }

    pub fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        write!(f, "for ({} in {}) {{", self.variable, self.expr,)?;
        self.body().display(f, indentation + 1)?;
        f.write_str("}")
    }
}

impl fmt::Display for ForInLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<ForInLoop> for Node {
    fn from(for_in: ForInLoop) -> Node {
        Self::ForInLoop(for_in)
    }
}

impl Executable for ForInLoop {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("ForIn", "exec");
        let object = self.expr().run(context)?;
        let mut result = Value::undefined();

        if object.is_null_or_undefined() {
            return Ok(result);
        }
        let object = object.to_object(context)?;
        let for_in_iterator = ForInIterator::create_for_in_iterator(context, Value::from(object))?;
        let next_function = for_in_iterator
            .get_property("next")
            .map(|p| p.as_data_descriptor().unwrap().value())
            .ok_or_else(|| context.construct_type_error("Could not find property `next`"))?;
        let iterator = IteratorRecord::new(for_in_iterator, next_function);

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
                        environment
                            .set_mutable_binding(name.as_ref(), next_result.clone(), true)
                            .map_err(|e| e.to_error(context))?;
                    } else {
                        environment
                            .create_mutable_binding(
                                name.as_ref().to_owned(),
                                true,
                                VariableScope::Function,
                            )
                            .map_err(|e| e.to_error(context))?;
                        let environment = &mut context.realm_mut().environment;
                        environment
                            .initialize_binding(name.as_ref(), next_result.clone())
                            .map_err(|e| e.to_error(context))?;
                    }
                }
                Node::VarDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        let environment = &mut context.realm_mut().environment;

                        if var.init().is_some() {
                            return context.throw_syntax_error("a declaration in the head of a for-in loop can't have an initializer");
                        }

                        if environment.has_binding(var.name()) {
                            environment
                                .set_mutable_binding(var.name(), next_result, true)
                                .map_err(|e| e.to_error(context))?;
                        } else {
                            environment
                                .create_mutable_binding(
                                    var.name().to_owned(),
                                    false,
                                    VariableScope::Function,
                                )
                                .map_err(|e| e.to_error(context))?;
                            let environment = &mut context.realm_mut().environment;
                            environment
                                .initialize_binding(var.name(), next_result)
                                .map_err(|e| e.to_error(context))?;
                        }
                    }
                    _ => {
                        return context.throw_syntax_error(
                            "only one variable can be declared in the head of a for-in loop",
                        )
                    }
                },
                Node::LetDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        let environment = &mut context.realm_mut().environment;

                        if var.init().is_some() {
                            return context.throw_syntax_error("a declaration in the head of a for-in loop can't have an initializer");
                        }

                        environment
                            .create_mutable_binding(
                                var.name().to_owned(),
                                false,
                                VariableScope::Block,
                            )
                            .map_err(|e| e.to_error(context))?;
                        let environment = &mut context.realm_mut().environment;
                        environment
                            .initialize_binding(var.name(), next_result)
                            .map_err(|e| e.to_error(context))?;
                    }
                    _ => {
                        return context.throw_syntax_error(
                            "only one variable can be declared in the head of a for-in loop",
                        )
                    }
                },
                Node::ConstDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        let environment = &mut context.realm_mut().environment;

                        if var.init().is_some() {
                            return context.throw_syntax_error("a declaration in the head of a for-in loop can't have an initializer");
                        }

                        environment
                            .create_immutable_binding(
                                var.name().to_owned(),
                                false,
                                VariableScope::Block,
                            )
                            .map_err(|e| e.to_error(context))?;
                        let environment = &mut context.realm_mut().environment;
                        environment
                            .initialize_binding(var.name(), next_result)
                            .map_err(|e| e.to_error(context))?;
                    }
                    _ => {
                        return context.throw_syntax_error(
                            "only one variable can be declared in the head of a for-in loop",
                        )
                    }
                },
                Node::Assign(_) => {
                    return context.throw_syntax_error(
                        "a declaration in the head of a for-in loop can't have an initializer",
                    );
                }
                _ => {
                    return context
                        .throw_syntax_error("unknown left hand side in head of for-in loop")
                }
            }

            result = self.body().run(context)?;
            match context.executor().get_current_state() {
                InterpreterState::Break(label) => {
                    handle_state_with_labels!(self, label, context, break);
                    break;
                }
                InterpreterState::Continue(label) => {
                    handle_state_with_labels!(self, label, context, continue);
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
