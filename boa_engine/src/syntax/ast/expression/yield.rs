use boa_interner::{Interner, ToInternedString};
use std::ops::ControlFlow;

use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::syntax::ast::ContainsSymbol;

use super::Expression;

/// The `yield` keyword is used to pause and resume a generator function
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-YieldExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/yield
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Yield {
    target: Option<Box<Expression>>,
    delegate: bool,
}

impl Yield {
    /// Gets the target expression of this `Yield` statement.
    #[inline]
    pub fn target(&self) -> Option<&Expression> {
        self.target.as_ref().map(Box::as_ref)
    }

    /// Returns `true` if this `Yield` statement delegates to another generator or iterable object.
    #[inline]
    pub fn delegate(&self) -> bool {
        self.delegate
    }

    /// Creates a `Yield` AST Expression.
    #[inline]
    pub fn new(expr: Option<Expression>, delegate: bool) -> Self {
        Self {
            target: expr.map(Box::new),
            delegate,
        }
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        matches!(self.target, Some(ref expr) if expr.contains_arguments())
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        matches!(self.target, Some(ref expr) if expr.contains(symbol))
    }
}

impl From<Yield> for Expression {
    #[inline]
    fn from(r#yield: Yield) -> Self {
        Self::Yield(r#yield)
    }
}

impl ToInternedString for Yield {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        let y = if self.delegate { "yield*" } else { "yield" };
        if let Some(ex) = self.target() {
            format!("{y} {}", ex.to_interned_string(interner))
        } else {
            y.to_owned()
        }
    }
}

impl VisitWith for Yield {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(expr) = &self.target {
            visitor.visit_expression(&**expr)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(expr) = &mut self.target {
            visitor.visit_expression_mut(&mut **expr)
        } else {
            ControlFlow::Continue(())
        }
    }
}
