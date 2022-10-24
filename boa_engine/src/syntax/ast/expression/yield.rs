use boa_interner::{Interner, ToInternedString};

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
    expr: Option<Box<Expression>>,
    delegate: bool,
}

impl Yield {
    /// Gets the expression of this `Yield` statement.
    #[inline]
    pub fn expr(&self) -> Option<&Expression> {
        self.expr.as_ref().map(Box::as_ref)
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
            expr: expr.map(Box::new),
            delegate,
        }
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        matches!(self.expr, Some(ref expr) if expr.contains_arguments())
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        matches!(self.expr, Some(ref expr) if expr.contains(symbol))
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
        if let Some(ex) = self.expr() {
            format!("{y} {}", ex.to_interned_string(interner))
        } else {
            y.to_owned()
        }
    }
}
