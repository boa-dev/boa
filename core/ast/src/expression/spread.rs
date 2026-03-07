use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

use crate::{
    Span, Spanned,
    visitor::{VisitWith, Visitor, VisitorMut},
};

use super::Expression;

/// The `spread` operator allows an iterable such as an array expression or string to be
/// expanded.
///
/// Syntax: `...x`
///
/// It expands array expressions or strings in places where zero or more arguments (for
/// function calls) or elements (for array literals)
/// are expected, or an object expression to be expanded in places where zero or more key-value
/// pairs (for object literals) are expected.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-SpreadElement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Spread_syntax
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Spread<'arena> {
    target: Box<Expression<'arena>>,
    span: Span,
}

impl<'arena> Spread<'arena> {
    /// Creates a [`Spread`] AST Expression.
    #[inline]
    #[must_use]
    pub fn new(target: Expression<'arena>, span: Span) -> Self {
        Self {
            target: Box::new(target),
            span,
        }
    }

    /// Gets the target expression to be expanded by the spread operator.
    #[inline]
    #[must_use]
    pub const fn target(&self) -> &Expression<'arena> {
        &self.target
    }
}

impl Spanned for Spread<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl ToInternedString for Spread<'_> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("...{}", self.target().to_interned_string(interner))
    }
}

impl<'arena> From<Spread<'arena>> for Expression<'arena> {
    #[inline]
    fn from(spread: Spread<'arena>) -> Self {
        Self::Spread(spread)
    }
}

impl<'arena> VisitWith<'arena> for Spread<'arena> {
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
