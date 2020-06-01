use super::Node;
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `break` statement terminates the current loop, switch, or label statement and transfers
/// program control to the statement following the terminated statement.
///
/// The break statement includes an optional label that allows the program to break out of a
/// labeled statement. The break statement needs to be nested within the referenced label. The
/// labeled statement can be any block statement; it does not have to be preceded by a loop
/// statement.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-BreakStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Break {
    label: Option<Box<str>>,
}

impl Break {
    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Box::as_ref)
    }

    /// Creates a `Break` AST node.
    pub fn new<OL, L>(label: OL) -> Self
    where
        L: Into<Box<str>>,
        OL: Into<Option<L>>,
    {
        Self {
            label: label.into().map(L::into),
        }
    }
}

impl fmt::Display for Break {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "break{}",
            if self.label().is_some() {
                format!(" {}", self.label().as_ref().unwrap())
            } else {
                String::new()
            }
        )
    }
}

impl From<Break> for Node {
    fn from(break_smt: Break) -> Node {
        Self::Break(break_smt)
    }
}
