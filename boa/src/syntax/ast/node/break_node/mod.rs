use super::Node;
use crate::{
    exec::Executable,
    exec::InterpreterState,
    gc::{Finalize, Trace},
    Context, JsResult, JsValue,
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Break {
    label: Option<Box<str>>,
}

impl Break {
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

    /// Gets the label of the break statement, if any.
    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Box::as_ref)
    }
}

impl Executable for Break {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        context
            .executor()
            .set_current_state(InterpreterState::Break(self.label().map(Box::from)));

        Ok(JsValue::undefined())
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
