use crate::{
    exec::{Executable, InterpreterState},
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
    Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `do...while` statement creates a loop that executes a specified statement until the
/// test condition evaluates to false.
///
/// The condition is evaluated after executing the statement, resulting in the specified
/// statement executing at least once.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-do-while-statement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct DoWhileLoop {
    body: Box<Node>,
    cond: Box<Node>,
    label: Option<Box<str>>,
}

impl DoWhileLoop {
    pub fn body(&self) -> &Node {
        &self.body
    }

    pub fn cond(&self) -> &Node {
        &self.cond
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Box::as_ref)
    }

    /// Creates a `DoWhileLoop` AST node.
    pub fn new<B, C>(body: B, condition: C) -> Self
    where
        B: Into<Node>,
        C: Into<Node>,
    {
        Self {
            body: Box::new(body.into()),
            cond: Box::new(condition.into()),
            label: None,
        }
    }

    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        write!(f, "do")?;
        self.body().display(f, indentation)?;
        write!(f, "while ({})", self.cond())
    }
}

impl Executable for DoWhileLoop {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let mut result = self.body().run(context)?;
        match context.executor().get_current_state() {
            InterpreterState::Break(_label) => {
                // TODO break to label.

                // Loops 'consume' breaks.
                context
                    .executor()
                    .set_current_state(InterpreterState::Executing);
                return Ok(result);
            }
            InterpreterState::Continue(_label) => {
                // TODO continue to label;
                context
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

        while self.cond().run(context)?.to_boolean() {
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

impl fmt::Display for DoWhileLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<DoWhileLoop> for Node {
    fn from(do_while: DoWhileLoop) -> Self {
        Self::DoWhileLoop(do_while)
    }
}
