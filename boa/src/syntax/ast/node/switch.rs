//! Switch node.
//!
use super::Node;
use gc::{Finalize, Trace};
use std::fmt;

use crate::syntax::ast::node::StatementList;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Case {
    condition: Node,
    body: StatementList,
}

impl Case {
    pub fn condition(&self) -> &Node {
        &self.condition
    }

    pub fn body(&self) -> &StatementList {
        &self.body
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Switch {
    val: Box<Node>,
    cases: Box<[Case]>,
    default: Option<Box<Node>>,
}

impl Switch {
    pub fn val(&self) -> &Node {
        &self.val
    }

    pub fn cases(&self) -> &[Case] {
        &self.cases
    }

    pub fn default(&self) -> Option<&Node> {
        self.default.as_ref().map(Box::as_ref)
    }

    /// Creates a `Switch` AST node.
    pub fn new<V, C>(val: V, cases: C, default: Option<V>) -> Self
    where
        V: Into<Node>,
        C: Into<Box<[Case]>>,
    {
        Self {
            val: Box::new(val.into()),
            cases: cases.into(),
            default: default.map(V::into).map(Box::new),
        }
    }

    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        writeln!(f, "switch ({}) {{", self.val())?;
        for e in self.cases().iter() {
            writeln!(f, "{}case {}:", indent, e.condition())?;
            e.body().display(f, indent)?;
        }

        if self.default().is_some() {
            writeln!(f, "{}default:", indent)?;
            self.default().as_ref().unwrap().display(f, indent + 1)?;
        }
        writeln!(f, "{}}}", indent)
    }
}

impl fmt::Display for Switch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<Switch> for Node {
    fn from(switch: Switch) -> Self {
        Self::Switch(switch)
    }
}
