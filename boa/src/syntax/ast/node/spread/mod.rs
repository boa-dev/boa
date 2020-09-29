use crate::{exec::Executable, syntax::ast::node::Node, Context, Result, Value};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `spread` operator allows an iterable such as an array expression or string to be
/// expanded.
///
/// Syntax: `...x`
///
/// It expands array expressions or strings in places where zero or more arguments (for
/// function calls) or elements (for array literals)
/// are expected, or an object expression to be expanded in places where zero or more key-value
/// pairs (for object literals) are expected.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-SpreadElement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Spread_syntax
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Spread {
    val: Box<Node>,
}

impl Spread {
    pub fn val(&self) -> &Node {
        &self.val
    }

    /// Creates a `Spread` AST node.
    pub fn new<V>(val: V) -> Self
    where
        V: Into<Node>,
    {
        Self {
            val: Box::new(val.into()),
        }
    }
}

impl Executable for Spread {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        // TODO: for now we can do nothing but return the value as-is
        self.val().run(interpreter)
    }
}

impl fmt::Display for Spread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "...{}", self.val())
    }
}

impl From<Spread> for Node {
    fn from(spread: Spread) -> Node {
        Self::Spread(spread)
    }
}
