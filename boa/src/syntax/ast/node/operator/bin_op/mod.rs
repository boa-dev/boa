use crate::syntax::ast::{node::Node, op};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// Binary operators requires two operands, one before the operator and one after the operator.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Operators
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct BinOp {
    op: op::BinOp,
    lhs: Box<Node>,
    rhs: Box<Node>,
}

impl BinOp {
    /// Creates a `BinOp` AST node.
    pub(in crate::syntax) fn new<O, L, R>(op: O, lhs: L, rhs: R) -> Self
    where
        O: Into<op::BinOp>,
        L: Into<Node>,
        R: Into<Node>,
    {
        Self {
            op: op.into(),
            lhs: Box::new(lhs.into()),
            rhs: Box::new(rhs.into()),
        }
    }

    /// Gets the binary operation of the node.
    pub fn op(&self) -> op::BinOp {
        self.op
    }

    /// Gets the left hand side of the binary operation.
    pub fn lhs(&self) -> &Node {
        &self.lhs
    }

    /// Gets the right hand side of the binary operation.
    pub fn rhs(&self) -> &Node {
        &self.rhs
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.lhs, self.op, self.rhs)
    }
}

impl From<BinOp> for Node {
    fn from(op: BinOp) -> Self {
        Self::BinOp(op)
    }
}
