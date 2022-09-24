use boa_interner::{Interner, Sym, ToInternedString};

use crate::syntax::ast::Statement;
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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Break {
    label: Option<Sym>,
}

impl Break {
    /// Creates a `Break` AST node.
    pub fn new(label: Option<Sym>) -> Self {
        Self { label }
    }

    /// Gets the label of the break statement, if any.
    pub fn label(&self) -> Option<Sym> {
        self.label
    }
}

impl ToInternedString for Break {
    fn to_interned_string(&self, interner: &Interner) -> String {
        if let Some(label) = self.label {
            format!("break {}", interner.resolve_expect(label))
        } else {
            "break".to_owned()
        }
    }
}

impl From<Break> for Statement {
    fn from(break_smt: Break) -> Self {
        Self::Break(break_smt)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fmt() {
        // Blocks do not store their label, so we cannot test with
        // the outer block having a label.
        //
        // TODO: Once block labels are implemented, this test should
        // include them:
        //
        // ```
        // outer: {
        //     while (true) {
        //         break outer;
        //     }
        //     skipped_call();
        // }
        // ```
        crate::syntax::ast::test_formatting(
            r#"
        {
            while (true) {
                break outer;
            }
            skipped_call();
        }
        while (true) {
            break;
        }
        "#,
        );
    }
}
