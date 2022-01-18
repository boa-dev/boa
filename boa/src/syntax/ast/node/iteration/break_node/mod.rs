use crate::{
    gc::{empty_trace, Finalize, Trace},
    syntax::ast::Node,
};
use boa_interner::{Interner, Sym, ToInternedString};

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
#[derive(Debug, Clone, Copy, Finalize, PartialEq)]
pub struct Break {
    label: Option<Sym>,
}

impl Break {
    /// Creates a `Break` AST node.
    pub fn new<L>(label: L) -> Self
    where
        L: Into<Option<Sym>>,
    {
        Self {
            label: label.into(),
        }
    }

    /// Gets the label of the break statement, if any.
    pub fn label(&self) -> Option<Sym> {
        self.label
    }
}

unsafe impl Trace for Break {
    empty_trace!();
}

impl ToInternedString for Break {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "break{}",
            if let Some(label) = self.label {
                format!(" {}", interner.resolve_expect(label))
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
