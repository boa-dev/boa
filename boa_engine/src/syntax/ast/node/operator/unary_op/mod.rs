use crate::syntax::ast::{node::Node, op};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// A unary operation is an operation with only one operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary_operators
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
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

impl ToInternedString for UnaryOp {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("{}{}", self.op, self.target.to_interned_string(interner))
    }
}

impl From<UnaryOp> for Node {
    fn from(op: UnaryOp) -> Self {
        Self::UnaryOp(op)
    }
}
