//! Await expression Expression.

use crate::syntax::ast::ContainsSymbol;

use super::Expression;
use boa_interner::{Interner, ToIndentedString, ToInternedString};

/// An await expression is used within an async function to pause execution and wait for a
/// promise to resolve.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AwaitExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Await {
    expr: Box<Expression>,
}

impl Await {
    /// Return the expression that should be awaited.
    #[inline]
    pub(crate) fn expr(&self) -> &Expression {
        &self.expr
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.expr.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.expr.contains(symbol)
    }
}

impl<T> From<T> for Await
where
    T: Into<Box<Expression>>,
{
    #[inline]
    fn from(e: T) -> Self {
        Self { expr: e.into() }
    }
}

impl ToInternedString for Await {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("await {}", self.expr.to_indented_string(interner, 0))
    }
}

impl From<Await> for Expression {
    #[inline]
    fn from(awaitexpr: Await) -> Self {
        Self::Await(awaitexpr)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fmt() {
        // TODO: `let a = await fn()` is invalid syntax as of writing. It should be tested here once implemented.
        crate::syntax::ast::test_formatting(
            r#"
            async function f() {
                await function_call();
            }
            "#,
        );
    }
}
