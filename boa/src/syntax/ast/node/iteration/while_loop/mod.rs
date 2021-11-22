use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
};
use std::fmt;

#[cfg(feature = "deser")]
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct WhileLoop {
    cond: Box<Node>,
    body: Box<Node>,
    label: Option<Box<str>>,
}

impl WhileLoop {
    pub fn cond(&self) -> &Node {
        &self.cond
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

    /// Creates a `WhileLoop` AST node.
    pub fn new<C, B>(condition: C, body: B) -> Self
    where
        C: Into<Node>,
        B: Into<Node>,
    {
        Self {
            cond: Box::new(condition.into()),
            body: Box::new(body.into()),
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
        write!(f, "while ({}) ", self.cond())?;
        self.body().display(f, indentation)
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
