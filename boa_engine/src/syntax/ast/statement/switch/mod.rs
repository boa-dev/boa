//! Switch node.
//!
use crate::syntax::ast::{expression::Expression, statement::Statement};
use boa_interner::{Interner, ToInternedString};

use crate::syntax::ast::statement::StatementList;

use super::ContainsSymbol;

#[cfg(test)]
mod tests;

#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    condition: Expression,
    body: StatementList,
}

impl Case {
    /// Creates a `Case` AST node.
    pub fn new(condition: Expression, body: StatementList) -> Self {
        Self { condition, body }
    }

    /// Gets the condition of the case.
    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    /// Gets the statement listin the body of the case.
    pub fn body(&self) -> &StatementList {
        &self.body
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.condition.contains_arguments() || self.body.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.condition.contains(symbol)
            || self
                .body
                .statements()
                .iter()
                .any(|stmt| stmt.contains(symbol))
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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Switch {
    val: Expression,
    cases: Box<[Case]>,
    default: Option<StatementList>,
}

impl Switch {
    /// Creates a `Switch` AST node.
    pub fn new(val: Expression, cases: Box<[Case]>, default: Option<StatementList>) -> Self {
        Self {
            val,
            cases,
            default,
        }
    }

    /// Gets the value to switch.
    pub fn val(&self) -> &Expression {
        &self.val
    }

    /// Gets the list of cases for the switch statement.
    pub fn cases(&self) -> &[Case] {
        &self.cases
    }

    /// Gets the default statement list, if any.
    pub fn default(&self) -> Option<&StatementList> {
        self.default.as_ref()
    }

    /// Implements the display formatting with indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
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

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.val.contains_arguments()
            || self.cases.iter().any(Case::contains_arguments)
            || matches!(self.default, Some(ref stmts) if stmts.contains_arguments())
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.val.contains(symbol) || self.cases.iter().any(|case| case.contains(symbol))
    }
}

impl ToInternedString for Switch {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<Switch> for Statement {
    fn from(switch: Switch) -> Self {
        Self::Switch(switch)
    }
}
