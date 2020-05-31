use super::Node;
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `throw` statement throws a user-defined exception.
///
/// Syntax: `throw expression;`
///
/// Execution of the current function will stop (the statements after throw won't be executed),
/// and control will be passed to the first catch block in the call stack. If no catch block
/// exists among caller functions, the program will terminate.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ThrowStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/throw
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Throw {
    expr: Box<Node>,
}

impl Throw {
    pub fn expr(&self) -> &Node {
        &self.expr
    }

    /// Creates a `Throw` AST node.
    pub fn new<V>(val: V) -> Self
    where
        V: Into<Node>,
    {
        Self {
            expr: Box::new(val.into()),
        }
    }
}

impl fmt::Display for Throw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "throw {}", self.expr)
    }
}

impl From<Throw> for Node {
    fn from(trw: Throw) -> Node {
        Self::Throw(trw)
    }
}
