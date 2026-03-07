use super::Expression;
use crate::{
    Span, Spanned,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

/// A parenthesized expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-grouping-operator
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Grouping
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Parenthesized<'arena> {
    pub(crate) expression: Box<Expression<'arena>>,
    span: Span,
}

impl<'arena> Parenthesized<'arena> {
    /// Creates a parenthesized expression.
    #[inline]
    #[must_use]
    pub fn new(expression: Expression<'arena>, span: Span) -> Self {
        Self {
            expression: Box::new(expression),
            span,
        }
    }

    /// Gets the expression of this parenthesized expression.
    #[inline]
    #[must_use]
    pub const fn expression(&self) -> &Expression<'arena> {
        &self.expression
    }
}

impl Spanned for Parenthesized<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> From<Parenthesized<'arena>> for Expression<'arena> {
    fn from(p: Parenthesized<'arena>) -> Self {
        Self::Parenthesized(p)
    }
}

impl ToInternedString for Parenthesized<'_> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("({})", self.expression.to_interned_string(interner))
    }
}

impl<'arena> VisitWith<'arena> for Parenthesized<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_expression(&self.expression)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_expression_mut(&mut self.expression)
    }
}
