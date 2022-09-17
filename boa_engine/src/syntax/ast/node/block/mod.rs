//! Block AST node.

use super::{Node, StatementList};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// A `block` statement (or compound statement in other languages) is used to group zero or
/// more statements.
///
/// The block statement is often called compound statement in other languages.
/// It allows you to use multiple statements where JavaScript expects only one statement.
/// Combining statements into blocks is a common practice in JavaScript. The opposite behavior
/// is possible using an empty statement, where you provide no statement, although one is
/// required.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-BlockStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    #[cfg_attr(feature = "deser", serde(flatten))]
    statements: StatementList,
    label: Option<Sym>,
}

impl Block {
    /// Gets the list of statements and declarations in this block.
    pub(crate) fn items(&self) -> &[Node] {
        self.statements.items()
    }

    /// Get the lexically declared names of the block.
    pub(crate) fn lexically_declared_names(&self) -> Vec<(Sym, bool)> {
        self.statements.lexically_declared_names()
    }

    /// Implements the display formatting with indentation.
    pub(super) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "{{\n{}{}}}",
            self.statements
                .to_indented_string(interner, indentation + 1),
            "    ".repeat(indentation)
        )
    }

    pub fn label(&self) -> Option<Sym> {
        self.label
    }

    pub fn set_label(&mut self, label: Sym) {
        self.label = Some(label);
    }
}

impl<T> From<T> for Block
where
    T: Into<StatementList>,
{
    fn from(list: T) -> Self {
        Self {
            statements: list.into(),
            label: None,
        }
    }
}

impl ToInternedString for Block {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<Block> for Node {
    fn from(block: Block) -> Self {
        Self::Block(block)
    }
}
