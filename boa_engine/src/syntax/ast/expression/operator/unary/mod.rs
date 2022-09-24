pub mod op;

use boa_interner::{Interner, ToInternedString};

use crate::syntax::ast::expression::Expression;

/// A unary operation is an operation with only one operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary_operators
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Unary {
    op: op::UnaryOp,
    target: Box<Expression>,
}

impl Unary {
    /// Creates a new `UnaryOp` AST Expression.
    pub(in crate::syntax) fn new(op: op::UnaryOp, target: Expression) -> Self {
        Self {
            op,
            target: Box::new(target),
        }
    }

    /// Gets the unary operation of the Expression.
    pub fn op(&self) -> op::UnaryOp {
        self.op
    }

    /// Gets the target of this unary operator.
    pub fn target(&self) -> &Expression {
        self.target.as_ref()
    }
}

impl ToInternedString for Unary {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let space = match self.op {
            op::UnaryOp::TypeOf | op::UnaryOp::Delete | op::UnaryOp::Void => " ",
            _ => "",
        };
        format!(
            "{}{space}{}",
            self.op,
            self.target.to_interned_string(interner)
        )
    }
}

impl From<Unary> for Expression {
    fn from(op: Unary) -> Self {
        Self::Unary(op)
    }
}
