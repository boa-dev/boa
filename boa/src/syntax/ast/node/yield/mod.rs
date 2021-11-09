use crate::syntax::ast::node::Node;
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The `yield` keyword is used to pause and resume a generator function
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-YieldExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/yield
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Yield {
    expr: Option<Box<Node>>,
    delegate: bool,
}

impl Yield {
    pub fn expr(&self) -> Option<&Node> {
        self.expr.as_ref().map(Box::as_ref)
    }

    pub fn delegate(&self) -> bool {
        self.delegate
    }

    /// Creates a `Yield` AST node.
    pub fn new<E, OE>(expr: OE, delegate: bool) -> Self
    where
        E: Into<Node>,
        OE: Into<Option<E>>,
    {
        Self {
            expr: expr.into().map(E::into).map(Box::new),
            delegate,
        }
    }
}

impl From<Yield> for Node {
    fn from(r#yield: Yield) -> Node {
        Node::Yield(r#yield)
    }
}

impl fmt::Display for Yield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let y = if self.delegate { "yield*" } else { "yield" };
        match self.expr() {
            Some(ex) => write!(f, "{} {}", y, ex),
            None => write!(f, "{}", y),
        }
    }
}
