use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::syntax::ast::ContainsSymbol;

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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "deser", serde(transparent))]
#[derive(Clone, Debug, PartialEq)]
pub struct Spread {
    target: Box<Expression>,
}

impl Spread {
    /// Gets the target expression to be expanded by the spread operator.
    #[inline]
    pub fn target(&self) -> &Expression {
        &self.target
    }

    /// Creates a `Spread` AST Expression.
    #[inline]
    pub fn new(target: Expression) -> Self {
        Self {
            target: Box::new(target),
        }
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.target.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.target.contains(symbol)
    }
}

impl ToInternedString for Spread {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("...{}", self.target().to_interned_string(interner))
    }
}

impl From<Spread> for Expression {
    #[inline]
    fn from(spread: Spread) -> Self {
        Self::Spread(spread)
    }
}

impl VisitWith for Spread {
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

#[cfg(test)]
mod tests {
    use crate::exec;

    #[test]
    fn spread_with_new() {
        let scenario = r#"
    function F(m) {
        this.m = m;
    }
    function f(...args) {
        return new F(...args);
    }
    let a = f('message');
    a.m;
    "#;
        assert_eq!(&exec(scenario), r#""message""#);
    }

    #[test]
    fn spread_with_call() {
        let scenario = r#"
    function f(m) {
        return m;
    }
    function g(...args) {
        return f(...args);
    }
    let a = g('message');
    a;
    "#;
        assert_eq!(&exec(scenario), r#""message""#);
    }

    #[test]
    fn fmt() {
        crate::syntax::ast::test_formatting(
            r#"
        function f(m) {
            return m;
        }
        function g(...args) {
            return f(...args);
        }
        let a = g("message");
        a;
        "#,
        );
    }
}
