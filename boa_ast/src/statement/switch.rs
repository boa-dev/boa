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
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    condition: Expression,
    body: StatementList,
}

impl Case {
    /// Creates a `Case` AST node.
    #[inline]
    #[must_use]
    pub fn new(condition: Expression, body: StatementList) -> Self {
        Self { condition, body }
    }

    /// Gets the condition of the case.
    #[inline]
    #[must_use]
    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    /// Gets the statement listin the body of the case.
    #[inline]
    #[must_use]
    pub fn body(&self) -> &StatementList {
        &self.body
    }
}

impl VisitWith for Case {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_expression(&self.condition));
        visitor.visit_statement_list(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_expression_mut(&mut self.condition));
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
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Switch {
    val: Expression,
    cases: Box<[Case]>,
    default: Option<StatementList>,
}

impl Switch {
    /// Creates a `Switch` AST node.
    #[inline]
    #[must_use]
    pub fn new(val: Expression, cases: Box<[Case]>, default: Option<StatementList>) -> Self {
        Self {
            val,
            cases,
            default,
        }
    }

    /// Gets the value to switch.
    #[inline]
    #[must_use]
    pub fn val(&self) -> &Expression {
        &self.val
    }

    /// Gets the list of cases for the switch statement.
    #[inline]
    #[must_use]
    pub fn cases(&self) -> &[Case] {
        &self.cases
    }

    /// Gets the default statement list, if any.
    #[inline]
    #[must_use]
    pub fn default(&self) -> Option<&StatementList> {
        self.default.as_ref()
    }
}

impl ToIndentedString for Switch {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let indent = "    ".repeat(indentation);
        let mut buf = format!("switch ({}) {{\n", self.val().to_interned_string(interner));
        for e in self.cases().iter() {
            buf.push_str(&format!(
                "{}    case {}:\n{}",
                indent,
                e.condition().to_interned_string(interner),
                e.body().to_indented_string(interner, indentation + 2)
            ));
        }

        if let Some(ref default) = self.default {
            buf.push_str(&format!(
                "{indent}    default:\n{}",
                default.to_indented_string(interner, indentation + 2)
            ));
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
        for case in self.cases.iter() {
            try_break!(visitor.visit_case(case));
        }
        if let Some(sl) = &self.default {
            try_break!(visitor.visit_statement_list(sl));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_expression_mut(&mut self.val));
        for case in self.cases.iter_mut() {
            try_break!(visitor.visit_case_mut(case));
        }
        if let Some(sl) = &mut self.default {
            try_break!(visitor.visit_statement_list_mut(sl));
        }
        ControlFlow::Continue(())
    }
}
