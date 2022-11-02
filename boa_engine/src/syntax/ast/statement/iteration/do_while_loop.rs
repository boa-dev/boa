use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::syntax::ast::{expression::Expression, statement::Statement, ContainsSymbol};
use crate::try_break;
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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct DoWhileLoop {
    body: Box<Statement>,
    condition: Expression,
}

impl DoWhileLoop {
    /// Gets the body of the do-while loop.
    #[inline]
    pub fn body(&self) -> &Statement {
        &self.body
    }

    /// Gets the condition of the do-while loop.
    #[inline]
    pub fn cond(&self) -> &Expression {
        &self.condition
    }
    /// Creates a `DoWhileLoop` AST node.
    #[inline]
    pub fn new(body: Statement, condition: Expression) -> Self {
        Self {
            body: body.into(),
            condition,
        }
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.body.contains_arguments() || self.condition.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.body.contains(symbol) || self.condition.contains(symbol)
    }
}

impl ToIndentedString for DoWhileLoop {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "do {} while ({})",
            self.body().to_indented_string(interner, indentation),
            self.cond().to_interned_string(interner)
        )
    }
}

impl From<DoWhileLoop> for Statement {
    fn from(do_while: DoWhileLoop) -> Self {
        Self::DoWhileLoop(do_while)
    }
}

impl VisitWith for DoWhileLoop {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_statement(&self.body));
        visitor.visit_expression(&self.condition)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_statement_mut(&mut self.body));
        visitor.visit_expression_mut(&mut self.condition)
    }
}
