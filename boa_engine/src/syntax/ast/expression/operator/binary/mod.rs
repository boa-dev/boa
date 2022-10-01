pub mod op;

use boa_interner::{Interner, ToInternedString};

use crate::syntax::ast::{expression::Expression, ContainsSymbol};

/// Binary operations require two operands, one before the operator and one after the operator.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Operators
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Binary {
    op: op::BinaryOp,
    lhs: Box<Expression>,
    rhs: Box<Expression>,
}

impl Binary {
    /// Creates a `BinOp` AST Expression.
    pub(in crate::syntax) fn new(op: op::BinaryOp, lhs: Expression, rhs: Expression) -> Self {
        Self {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Gets the binary operation of the Expression.
    pub fn op(&self) -> op::BinaryOp {
        self.op
    }

    /// Gets the left hand side of the binary operation.
    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    /// Gets the right hand side of the binary operation.
    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.lhs.contains_arguments() || self.rhs.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.lhs.contains(symbol) || self.rhs.contains(symbol)
    }
}

impl ToInternedString for Binary {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{} {} {}",
            self.lhs.to_interned_string(interner),
            self.op,
            self.rhs.to_interned_string(interner)
        )
    }
}

impl From<Binary> for Expression {
    fn from(op: Binary) -> Self {
        Self::Binary(op)
    }
}
