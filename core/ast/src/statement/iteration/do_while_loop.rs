use crate::{
    expression::Expression,
    statement::Statement,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

/// The `do...while` statement creates a loop that executes a specified statement until the
/// test condition evaluates to false.
///
/// The condition is evaluated after executing the statement, resulting in the specified
/// statement executing at least once.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-do-while-statement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct DoWhileLoop<'arena> {
    body: Box<Statement<'arena>>,
    condition: Expression<'arena>,
}

impl<'arena> DoWhileLoop<'arena> {
    /// Gets the body of the do-while loop.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &Statement<'arena> {
        &self.body
    }

    /// Gets the condition of the do-while loop.
    #[inline]
    #[must_use]
    pub const fn cond(&self) -> &Expression<'arena> {
        &self.condition
    }
    /// Creates a `DoWhileLoop` AST node.
    #[inline]
    #[must_use]
    pub fn new(body: Statement<'arena>, condition: Expression<'arena>) -> Self {
        Self {
            body: body.into(),
            condition,
        }
    }
}

impl<'arena> ToIndentedString for DoWhileLoop<'arena> {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "do {} while ({})",
            self.body().to_indented_string(interner, indentation),
            self.cond().to_interned_string(interner)
        )
    }
}

impl<'arena> From<DoWhileLoop<'arena>> for Statement<'arena> {
    fn from(do_while: DoWhileLoop<'arena>) -> Self {
        Self::DoWhileLoop(do_while)
    }
}

impl<'arena> VisitWith<'arena> for DoWhileLoop<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_statement(&self.body)?;
        visitor.visit_expression(&self.condition)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_statement_mut(&mut self.body)?;
        visitor.visit_expression_mut(&mut self.condition)
    }
}
