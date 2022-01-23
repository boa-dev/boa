use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
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

impl ToInternedString for Assign {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{} = {}",
            self.lhs.to_interned_string(interner),
            self.rhs.to_interned_string(interner)
        )
    }
}

impl From<Assign> for Node {
    fn from(op: Assign) -> Self {
        Self::Assign(op)
    }
}
