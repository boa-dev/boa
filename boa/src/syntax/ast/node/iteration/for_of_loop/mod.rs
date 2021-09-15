use crate::{
    builtins::iterable::get_iterator,
    environment::{
        declarative_environment_record::DeclarativeEnvironmentRecord,
        lexical_environment::VariableScope,
    },
    exec::{Executable, InterpreterState},
    gc::{Finalize, Trace},
    syntax::ast::node::{Declaration, Node},
    BoaProfiler, Context, JsResult, JsValue,
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ForOfLoop {
    variable: Box<Node>,
    iterable: Box<Node>,
    body: Box<Node>,
    label: Option<Box<str>>,
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
            label: None,
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

    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Box::as_ref)
    }

    pub fn set_label(&mut self, label: Box<str>) {
        self.label = Some(label);
    }

    pub fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        if let Some(ref label) = self.label {
            write!(f, "{}: ", label)?;
        }
        write!(f, "for ({} of {}) ", self.variable, self.iterable)?;
        self.body().display(f, indentation)
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
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("ForOf", "exec");
        let iterable = self.iterable().run(context)?;
        let iterator = get_iterator(&iterable, context)?;
        let mut result = JsValue::undefined();

        loop {
            {
                let env = context.get_current_environment();
                context.push_environment(DeclarativeEnvironmentRecord::new(Some(env)));
            }
            let iterator_result = iterator.next(context)?;
            if iterator_result.done {
                context.pop_environment();
                break;
            }
            let next_result = iterator_result.value;

            match self.variable() {
                Node::Identifier(ref name) => {
                    if context.has_binding(name.as_ref())? {
                        // Binding already exists
                        context.set_mutable_binding(
                            name.as_ref(),
                            next_result.clone(),
                            context.strict(),
                        )?;
                    } else {
                        context.create_mutable_binding(
                            name.as_ref(),
                            true,
                            VariableScope::Function,
                        )?;
                        context.initialize_binding(name.as_ref(), next_result)?;
                    }
                }
                Node::VarDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        if var.init().is_some() {
                            return context.throw_syntax_error("a declaration in the head of a for-of loop can't have an initializer");
                        }

                        match &var {
                            Declaration::Identifier { ident, .. } => {
                                if context.has_binding(ident.as_ref())? {
                                    context.set_mutable_binding(
                                        ident.as_ref(),
                                        next_result,
                                        context.strict(),
                                    )?;
                                } else {
                                    context.create_mutable_binding(
                                        ident.as_ref(),
                                        false,
                                        VariableScope::Function,
                                    )?;
                                    context.initialize_binding(ident.as_ref(), next_result)?;
                                }
                            }
                            Declaration::Pattern(p) => {
                                for (ident, value) in p.run(Some(next_result), context)? {
                                    if context.has_binding(ident.as_ref())? {
                                        context.set_mutable_binding(
                                            ident.as_ref(),
                                            value,
                                            context.strict(),
                                        )?;
                                    } else {
                                        context.create_mutable_binding(
                                            ident.as_ref(),
                                            false,
                                            VariableScope::Function,
                                        )?;
                                        context.initialize_binding(ident.as_ref(), value)?;
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        return context.throw_syntax_error(
                            "only one variable can be declared in the head of a for-of loop",
                        )
                    }
                },
                Node::LetDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        if var.init().is_some() {
                            return context.throw_syntax_error("a declaration in the head of a for-of loop can't have an initializer");
                        }

                        match &var {
                            Declaration::Identifier { ident, .. } => {
                                context.create_mutable_binding(
                                    ident.as_ref(),
                                    false,
                                    VariableScope::Block,
                                )?;
                                context.initialize_binding(ident.as_ref(), next_result)?;
                            }
                            Declaration::Pattern(p) => {
                                for (ident, value) in p.run(Some(next_result), context)? {
                                    context.create_mutable_binding(
                                        ident.as_ref(),
                                        false,
                                        VariableScope::Block,
                                    )?;
                                    context.initialize_binding(ident.as_ref(), value)?;
                                }
                            }
                        }
                    }
                    _ => {
                        return context.throw_syntax_error(
                            "only one variable can be declared in the head of a for-of loop",
                        )
                    }
                },
                Node::ConstDeclList(ref list) => match list.as_ref() {
                    [var] => {
                        if var.init().is_some() {
                            return context.throw_syntax_error("a declaration in the head of a for-of loop can't have an initializer");
                        }

                        match &var {
                            Declaration::Identifier { ident, .. } => {
                                context.create_immutable_binding(
                                    ident.as_ref(),
                                    false,
                                    VariableScope::Block,
                                )?;
                                context.initialize_binding(ident.as_ref(), next_result)?;
                            }
                            Declaration::Pattern(p) => {
                                for (ident, value) in p.run(Some(next_result), context)? {
                                    context.create_immutable_binding(
                                        ident.as_ref(),
                                        false,
                                        VariableScope::Block,
                                    )?;
                                    context.initialize_binding(ident.as_ref(), value)?;
                                }
                            }
                        }
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
            let _ = context.pop_environment();
        }
        Ok(result)
    }
}
