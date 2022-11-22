//! Unary expression nodes.
//!
//! A Binary expression comprises any operation applied to a single expression. Some examples include:
//!
//! - [Increment and decrement operations][inc] (`++`, `--`).
//! - The [`delete`][del] operator.
//! - The [bitwise NOT][not] operator (`~`).
//!
//! The full list of valid unary operators is defined in [`UnaryOp`].
//!
//! [inc]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#increment_and_decrement
//! [del]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/delete
//! [not]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_NOT
mod op;

use core::ops::ControlFlow;
pub use op::*;

use boa_interner::{Interner, ToInternedString};

use crate::expression::Expression;
use crate::visitor::{VisitWith, Visitor, VisitorMut};

/// A unary expression is an operation with only one operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary_operators
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Unary {
    op: UnaryOp,
    target: Box<Expression>,
}

impl Unary {
    /// Creates a new `UnaryOp` AST Expression.
    #[must_use]
    pub fn new(op: UnaryOp, target: Expression) -> Self {
        Self {
            op,
            target: Box::new(target),
        }
    }

    /// Gets the unary operation of the Expression.
    #[inline]
    #[must_use]
    pub const fn op(&self) -> UnaryOp {
        self.op
    }

    /// Gets the target of this unary operator.
    #[inline]
    #[must_use]
    pub fn target(&self) -> &Expression {
        self.target.as_ref()
    }
}

impl ToInternedString for Unary {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        let space = match self.op {
            UnaryOp::TypeOf | UnaryOp::Delete | UnaryOp::Void => " ",
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
    #[inline]
    fn from(op: Unary) -> Self {
        Self::Unary(op)
    }
}

impl VisitWith for Unary {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_expression(&self.target)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_expression_mut(&mut self.target)
    }
}
