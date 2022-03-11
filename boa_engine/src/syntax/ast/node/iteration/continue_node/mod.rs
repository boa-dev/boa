use crate::syntax::ast::node::Node;
use boa_gc::{Finalize, Trace};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Continue {
    label: Option<Sym>,
}

impl Continue {
    /// Creates a `Continue` AST node.
    pub fn new<L>(label: L) -> Self
    where
        L: Into<Option<Sym>>,
    {
        Self {
            label: label.into(),
        }
    }

    pub fn label(&self) -> Option<Sym> {
        self.label
    }
}

impl ToInternedString for Continue {
    fn to_interned_string(&self, interner: &Interner) -> String {
        if let Some(label) = self.label {
            format!("continue {}", interner.resolve_expect(label))
        } else {
            "continue".to_owned()
        }
    }
}

impl From<Continue> for Node {
    fn from(cont: Continue) -> Self {
        Self::Continue(cont)
    }
}
