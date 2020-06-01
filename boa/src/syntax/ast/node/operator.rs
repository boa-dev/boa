use super::Node;
use crate::syntax::ast::op;
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An assignment operator assigns a value to its left operand based on the value of its right
/// operand.
///
/// Assignment operator (`=`), assigns the value of its right operand to its left operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Assign {
    lhs: Box<Node>,
    rhs: Box<Node>,
}

impl Assign {
    /// Creates an `Assign` AST node.
    pub(in crate::syntax) fn new<L, R>(lhs: L, rhs: R) -> Self
    where
        L: Into<Node>,
        R: Into<Node>,
    {
        Self {
            lhs: Box::new(lhs.into()),
            rhs: Box::new(rhs.into()),
        }
    }

    /// Gets the left hand side of the assignment operation.
    pub fn lhs(&self) -> &Node {
        &self.lhs
    }

    /// Gets the right hand side of the assignment operation.
    pub fn rhs(&self) -> &Node {
        &self.rhs
    }
}

impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.lhs, self.rhs)
    }
}

impl From<Assign> for Node {
    fn from(op: Assign) -> Self {
        Self::Assign(op)
    }
}

/// Binary operators requires two operands, one before the operator and one after the operator.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Operators
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
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

/// A unary operation is an operation with only one operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary_operators
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct UnaryOp {
    op: op::UnaryOp,
    target: Box<Node>,
}

impl UnaryOp {
    /// Creates a new `UnaryOp` AST node.
    pub(in crate::syntax) fn new<V>(op: op::UnaryOp, target: V) -> Self
    where
        V: Into<Node>,
    {
        Self {
            op,
            target: Box::new(target.into()),
        }
    }

    /// Gets the unary operation of the node.
    pub fn op(&self) -> op::UnaryOp {
        self.op
    }

    /// Gets the target of this unary operator.
    pub fn target(&self) -> &Node {
        self.target.as_ref()
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.op, self.target)
    }
}

impl From<UnaryOp> for Node {
    fn from(op: UnaryOp) -> Self {
        Self::UnaryOp(op)
    }
}
