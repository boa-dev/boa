//! Switch node.
//!
use super::{join_nodes, Node};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Switch {
    val: Box<Node>,
    cases: Box<[(Node, Box<[Node]>)]>,
    default: Option<Box<Node>>,
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
impl Switch {
    pub fn val(&self) -> &Node {
        &self.val
    }

    pub fn cases(&self) -> &[(Node, Box<[Node]>)] {
        &self.cases
    }

    pub fn default(&self) -> &Option<Box<Node>> {
        &self.default
    }

    pub fn new(
        val: Box<Node>,
        cases: Box<[(Node, Box<[Node]>)]>,
        default: Option<Box<Node>>,
    ) -> Switch {
        Self {
            val,
            cases,
            default,
        }
    }

    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        writeln!(f, "switch ({}) {{", self.val())?;
        for e in self.cases().iter() {
            writeln!(f, "{}case {}:", indent, e.0)?;
            join_nodes(f, &e.1)?;
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
