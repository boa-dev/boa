use crate::{
    exec::{Executable, InterpreterState},
    syntax::ast::node::Node,
    Context, Result, Value,
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `continue` statement terminates execution of the statements in the current iteration of
/// the current or labeled loop, and continues execution of the loop with the next iteration.
///
/// The continue statement can include an optional label that allows the program to jump to the
/// next iteration of a labeled loop statement instead of the current loop. In this case, the
/// continue statement needs to be nested within this labeled statement.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ContinueStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Continue {
    label: Option<Box<str>>,
}

impl Continue {
    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Box::as_ref)
    }

    /// Creates a `Continue` AST node.
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

impl Executable for Continue {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        interpreter
            .executor()
            .set_current_state(InterpreterState::Continue(self.label().map(Box::from)));

        Ok(Value::undefined())
    }
}

impl fmt::Display for Continue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "continue{}",
            if let Some(label) = self.label() {
                format!(" {}", label)
            } else {
                String::new()
            }
        )
    }
}

impl From<Continue> for Node {
    fn from(cont: Continue) -> Node {
        Self::Continue(cont)
    }
}
