use crate::{
    expression::Expression,
    statement::Statement,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

/// The `return` statement ends function execution and specifies a value to be returned to the
/// function caller.
///
/// Syntax: `return [expression];`
///
/// `expression`:
///  > The expression whose value is to be returned. If omitted, `undefined` is returned instead.
///
/// When a `return` statement is used in a function body, the execution of the function is
/// stopped. If specified, a given value is returned to the function caller.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ReturnStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Return<'arena> {
    target: Option<Expression<'arena>>,
}

impl<'arena> Return<'arena> {
    /// Gets the target expression value of this `Return` statement.
    #[must_use]
    pub const fn target(&self) -> Option<&Expression<'arena>> {
        self.target.as_ref()
    }

    /// Creates a `Return` AST node.
    #[must_use]
    pub const fn new(expression: Option<Expression<'arena>>) -> Self {
        Self { target: expression }
    }
}

impl<'arena> From<Return<'arena>> for Statement<'arena> {
    fn from(return_smt: Return<'arena>) -> Self {
        Self::Return(return_smt)
    }
}

impl<'arena> ToInternedString for Return<'arena> {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.target().map_or_else(
            || "return".to_owned(),
            |ex| format!("return {}", ex.to_interned_string(interner)),
        )
    }
}

impl<'arena> VisitWith<'arena> for Return<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        if let Some(expr) = &self.target {
            visitor.visit_expression(expr)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        if let Some(expr) = &mut self.target {
            visitor.visit_expression_mut(expr)
        } else {
            ControlFlow::Continue(())
        }
    }
}
