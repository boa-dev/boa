use crate::{
    expression::Expression,
    scope::Scope,
    statement::Statement,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

/// The `with` statement extends the scope chain for a statement.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/with
/// [spec]: https://tc39.es/ecma262/#prod-WithStatement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct With<'arena> {
    pub(crate) expression: Expression<'arena>,
    pub(crate) statement: Box<Statement<'arena>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scope: Scope,
}

impl<'arena> With<'arena> {
    /// Creates a `With` AST node.
    #[must_use]
    pub fn new(expression: Expression<'arena>, statement: Statement<'arena>) -> Self {
        Self {
            expression,
            statement: Box::new(statement),
            scope: Scope::default(),
        }
    }

    /// Gets the expression value of this `With` statement.
    #[must_use]
    pub const fn expression(&self) -> &Expression<'arena> {
        &self.expression
    }

    /// Gets the statement value of this `With` statement.
    #[must_use]
    pub const fn statement(&self) -> &Statement<'arena> {
        &self.statement
    }

    /// Returns the scope of the `With` statement.
    #[must_use]
    pub const fn scope(&self) -> &Scope {
        &self.scope
    }
}

impl<'arena> From<With<'arena>> for Statement<'arena> {
    fn from(with: With<'arena>) -> Self {
        Self::With(with)
    }
}

impl ToIndentedString for With<'_> {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "with ({}) {}",
            self.expression().to_interned_string(interner),
            self.statement().to_indented_string(interner, indentation)
        )
    }
}

impl<'arena> VisitWith<'arena> for With<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_expression(&self.expression)?;
        visitor.visit_statement(&self.statement)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_expression_mut(&mut self.expression)?;
        visitor.visit_statement_mut(&mut self.statement)
    }
}
