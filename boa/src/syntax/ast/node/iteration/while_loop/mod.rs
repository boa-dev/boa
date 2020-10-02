use crate::{
    exec::{Executable, InterpreterState},
    syntax::ast::node::Node,
    Context, Result, Value,
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `while` statement creates a loop that executes a specified statement as long as the
/// test condition evaluates to `true`.
///
/// The condition is evaluated before executing the statement.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-grammar-notation-WhileStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/while
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct WhileLoop {
    cond: Box<Node>,
    expr: Box<Node>,
    label: Option<Box<str>>,
}

impl WhileLoop {
    pub fn cond(&self) -> &Node {
        &self.cond
    }

    pub fn expr(&self) -> &Node {
        &self.expr
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Box::as_ref)
    }

    /// Creates a `WhileLoop` AST node.
    pub fn new<C, B>(condition: C, body: B) -> Self
    where
        C: Into<Node>,
        B: Into<Node>,
    {
        Self {
            cond: Box::new(condition.into()),
            expr: Box::new(body.into()),
            label: None,
        }
    }

    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        write!(f, "while ({}) ", self.cond())?;
        self.expr().display(f, indentation)
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

impl fmt::Display for WhileLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<WhileLoop> for Node {
    fn from(while_loop: WhileLoop) -> Self {
        Self::WhileLoop(while_loop)
    }
}
