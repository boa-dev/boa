//! Binary expression nodes.
//!
//! A Binary expression comprises any operation between two expressions (excluding assignments),
//! such as:
//! - [Logic operations][logic] (`||`, `&&`).
//! - [Relational math][relat] (`==`, `<`).
//! - [Bit manipulation][bit] (`^`, `|`).
//! - [Arithmetic][arith] (`+`, `%`).
//! - The [comma operator][comma] (`,`)
//!
//! [logic]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#binary_logical_operators
//! [relat]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#relational_operators
//! [bit]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#binary_bitwise_operators
//! [arith]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#arithmetic_operators
//! [comma]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comma_Operator

mod op;
pub use op::*;

use boa_interner::{Interner, ToInternedString};

use crate::syntax::ast::{expression::Expression, ContainsSymbol};

/// Binary operations require two operands, one before the operator and one after the operator.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Binary {
    op: BinaryOp,
    lhs: Box<Expression>,
    rhs: Box<Expression>,
}

impl Binary {
    /// Creates a `BinOp` AST Expression.
    pub(in crate::syntax) fn new(op: BinaryOp, lhs: Expression, rhs: Expression) -> Self {
        Self {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Gets the binary operation of the Expression.
    #[inline]
    pub fn op(&self) -> BinaryOp {
        self.op
    }

    /// Gets the left hand side of the binary operation.
    #[inline]
    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    /// Gets the right hand side of the binary operation.
    #[inline]
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
    #[inline]
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
    #[inline]
    fn from(op: Binary) -> Self {
        Self::Binary(op)
    }
}
