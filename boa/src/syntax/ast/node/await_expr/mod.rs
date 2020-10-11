//! Await expression node.

use super::Node;
use crate::{exec::Executable, BoaProfiler, Context, Result, Value};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An await expression is used within an async function to pause execution and wait for a
/// promise to resolve.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AwaitExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct AwaitExpr {
    expr: Box<Node>,
}

impl Executable for AwaitExpr {
    fn run(&self, _: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("AwaitExpression", "exec");
        // unimplemented!("Await expression execution");
        Ok(Value::Undefined)
    }
}

impl AwaitExpr {
    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        writeln!(f, "await ")?;
        self.expr.display(f, indentation)
    }
}

impl<T> From<T> for AwaitExpr
where
    T: Into<Box<Node>>,
{
    fn from(e: T) -> Self {
        Self { expr: e.into() }
    }
}

impl fmt::Display for AwaitExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<AwaitExpr> for Node {
    fn from(awaitexpr: AwaitExpr) -> Self {
        Self::AwaitExpr(awaitexpr)
    }
}
