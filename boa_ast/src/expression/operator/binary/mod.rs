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

use core::ops::ControlFlow;
pub use op::*;

use boa_interner::{Interner, ToInternedString};

use crate::{
    expression::Expression,
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
};

/// Binary operations require two operands, one before the operator and one after the operator.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Binary {
    op: BinaryOp,
    lhs: Box<Expression>,
    rhs: Box<Expression>,
}

impl Binary {
    /// Creates a `BinOp` AST Expression.
    #[must_use]
    pub fn new(op: BinaryOp, lhs: Expression, rhs: Expression) -> Self {
        Self {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Gets the binary operation of the Expression.
    #[inline]
    #[must_use]
    pub const fn op(&self) -> BinaryOp {
        self.op
    }

    /// Gets the left hand side of the binary operation.
    #[inline]
    #[must_use]
    pub const fn lhs(&self) -> &Expression {
        &self.lhs
    }

    /// Gets the right hand side of the binary operation.
    #[inline]
    #[must_use]
    pub const fn rhs(&self) -> &Expression {
        &self.rhs
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

impl VisitWith for Binary {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_expression(&self.lhs));
        visitor.visit_expression(&self.rhs)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_expression_mut(&mut self.lhs));
        visitor.visit_expression_mut(&mut self.rhs)
    }
}
