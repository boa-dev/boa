//! Switch node.
//!
use crate::{
    expression::Expression,
    statement::Statement,
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
    StatementList,
};
use boa_interner::{Interner, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

/// A case clause inside a [`Switch`] statement, as defined by the [spec].
///
/// Even though every [`Case`] body is a [`StatementList`], it doesn't create a new lexical
/// environment. This means any variable declared in a `Case` will be considered as part of the
/// lexical environment of the parent [`Switch`] block.
///
/// [spec]: https://tc39.es/ecma262/#prod-CaseClause
/// [truthy]: https://developer.mozilla.org/en-US/docs/Glossary/Truthy
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    condition: Option<Expression>,
    body: StatementList,
}

impl Case {
    /// Creates a regular `Case` AST node.
    #[inline]
    #[must_use]
    pub const fn new(condition: Expression, body: StatementList) -> Self {
        Self {
            condition: Some(condition),
            body,
        }
    }

    /// Creates a default `Case` AST node.
    #[inline]
    #[must_use]
    pub const fn default(body: StatementList) -> Self {
        Self {
            condition: None,
            body,
        }
    }

    /// Gets the condition of the case.
    ///
    /// If it's a regular case returns [`Some`] with the [`Expression`],
    /// otherwise [`None`] is returned if it's the default case.
    #[inline]
    #[must_use]
    pub const fn condition(&self) -> Option<&Expression> {
        self.condition.as_ref()
    }

    /// Gets the statement listin the body of the case.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &StatementList {
        &self.body
    }

    /// Check if the case is the `default` case.
    #[inline]
    #[must_use]
    pub const fn is_default(&self) -> bool {
        self.condition.is_none()
    }
}

impl VisitWith for Case {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(condition) = &self.condition {
            try_break!(visitor.visit_expression(condition));
        }

        visitor.visit_statement_list(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(condition) = &mut self.condition {
            try_break!(visitor.visit_expression_mut(condition));
        }
        visitor.visit_statement_list_mut(&mut self.body)
    }
}

/// The `switch` statement evaluates an expression, matching the expression's value to a case
/// clause, and executes statements associated with that case, as well as statements in cases
/// that follow the matching case.
///
/// A `switch` statement first evaluates its expression. It then looks for the first case
/// clause whose expression evaluates to the same value as the result of the input expression
/// (using the strict comparison, `===`) and transfers control to that clause, executing the
/// associated statements. (If multiple cases match the provided value, the first case that
/// matches is selected, even if the cases are not equal to each other.)
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-SwitchStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Switch {
    val: Expression,
    cases: Box<[Case]>,
}

impl Switch {
    /// Creates a `Switch` AST node.
    #[inline]
    #[must_use]
    pub fn new(val: Expression, cases: Box<[Case]>) -> Self {
        Self { val, cases }
    }

    /// Gets the value to switch.
    #[inline]
    #[must_use]
    pub const fn val(&self) -> &Expression {
        &self.val
    }

    /// Gets the list of cases for the switch statement.
    #[inline]
    #[must_use]
    pub const fn cases(&self) -> &[Case] {
        &self.cases
    }

    /// Gets the default statement list, if any.
    #[inline]
    #[must_use]
    pub fn default(&self) -> Option<&StatementList> {
        for case in self.cases.as_ref() {
            if case.is_default() {
                return Some(case.body());
            }
        }
        None
    }
}

impl ToIndentedString for Switch {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let indent = "    ".repeat(indentation);
        let mut buf = format!("switch ({}) {{\n", self.val().to_interned_string(interner));
        for e in &*self.cases {
            if let Some(condition) = e.condition() {
                buf.push_str(&format!(
                    "{indent}    case {}:\n{}",
                    condition.to_interned_string(interner),
                    e.body().to_indented_string(interner, indentation + 2)
                ));
            } else {
                buf.push_str(&format!(
                    "{indent}    default:\n{}",
                    e.body().to_indented_string(interner, indentation + 2)
                ));
            }
        }

        buf.push_str(&format!("{indent}}}"));

        buf
    }
}

impl From<Switch> for Statement {
    #[inline]
    fn from(switch: Switch) -> Self {
        Self::Switch(switch)
    }
}

impl VisitWith for Switch {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_expression(&self.val));
        for case in &*self.cases {
            try_break!(visitor.visit_case(case));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_expression_mut(&mut self.val));
        for case in &mut *self.cases {
            try_break!(visitor.visit_case_mut(case));
        }
        ControlFlow::Continue(())
    }
}
