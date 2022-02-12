//! Await expression node.

use super::Node;
use crate::gc::{Finalize, Trace};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// An await expression is used within an async function to pause execution and wait for a
/// promise to resolve.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AwaitExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct AwaitExpr {
    expr: Box<Node>,
}

impl<T> From<T> for AwaitExpr
where
    T: Into<Box<Node>>,
{
    fn from(e: T) -> Self {
        Self { expr: e.into() }
    }
}

impl ToInternedString for AwaitExpr {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("await {}", self.expr.to_indented_string(interner, 0))
    }
}

impl From<AwaitExpr> for Node {
    fn from(awaitexpr: AwaitExpr) -> Self {
        Self::AwaitExpr(awaitexpr)
    }
}
