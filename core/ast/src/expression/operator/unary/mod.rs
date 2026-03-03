//! Unary expression nodes.
//!
//! A unary expression comprises any operation applied to a single expression. Some examples include:
//!
//! - The [`delete`][del] operator.
//! - The [bitwise NOT][not] operator (`~`).
//!
//! The full list of valid unary operators is defined in [`UnaryOp`].
//!
//! [del]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/delete
//! [not]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_NOT
mod op;

use crate::{
    Span, Spanned,
    expression::Expression,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

pub use op::*;

/// A unary expression is an operation with only one operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary_operators
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Unary<'arena> {
    op: UnaryOp,
    target: Box<Expression<'arena>>,
    span: Span,
}

impl<'arena> Unary<'arena> {
    /// Creates a new `UnaryOp` AST Expression.
    #[inline]
    #[must_use]
    pub fn new(op: UnaryOp, target: Expression<'arena>, span: Span) -> Self {
        Self {
            op,
            target: Box::new(target),
            span,
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
    pub fn target(&self) -> &Expression<'arena> {
        self.target.as_ref()
    }

    /// Gets the target of this unary operator.
    #[inline]
    #[must_use]
    pub fn target_mut(&mut self) -> &mut Expression<'arena> {
        self.target.as_mut()
    }
}

impl<'arena> Spanned for Unary<'arena> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> ToInternedString for Unary<'arena> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("{} {}", self.op, self.target.to_interned_string(interner))
    }
}

impl<'arena> From<Unary<'arena>> for Expression<'arena> {
    #[inline]
    fn from(op: Unary<'arena>) -> Self {
        Self::Unary(op)
    }
}

impl<'arena> VisitWith<'arena> for Unary<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_expression(&self.target)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_expression_mut(&mut self.target)
    }
}
