use crate::{
    exec::{Executable, InterpreterState},
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
    Context, JsResult, JsValue,
};
use std::fmt;

#[cfg(feature = "deser")]
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
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

    pub fn set_label(&mut self, label: Box<str>) {
        self.label = Some(label);
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
        if let Some(ref label) = self.label {
            write!(f, "{}: ", label)?;
        }
        write!(f, "do ")?;
        self.body().display(f, indentation)?;
        write!(f, " while ({})", self.cond())
    }
}

impl Executable for DoWhileLoop {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        let mut result;
        loop {
            result = self.body().run(context)?;
            match context.executor().get_current_state() {
                InterpreterState::Break(label) => {
                    handle_state_with_labels!(self, label, context, break);
                    break;
                }
                InterpreterState::Continue(label) => {
                    handle_state_with_labels!(self, label, context, continue);
                }
                InterpreterState::Return => {
                    return Ok(result);
                }
                InterpreterState::Executing => {
                    // Continue execution.
                }
            }
            if !self.cond().run(context)?.to_boolean() {
                break;
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
