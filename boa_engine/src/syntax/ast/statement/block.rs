//! Block AST node.

use crate::syntax::ast::{expression::Identifier, ContainsSymbol, StatementList};

use super::Statement;
use boa_interner::{Interner, ToInternedString};

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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Block {
    #[cfg_attr(feature = "deser", serde(flatten))]
    statements: StatementList,
}

impl Block {
    /// Gets the list of statements and declarations in this block.
    pub(crate) fn statement_list(&self) -> &StatementList {
        &self.statements
    }

    /// Get the lexically declared names of the block.
    pub(crate) fn lexically_declared_names(&self) -> Vec<(Identifier, bool)> {
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

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.statements.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.statements.contains(symbol)
    }
}

impl<T> From<T> for Block
where
    T: Into<StatementList>,
{
    fn from(list: T) -> Self {
        Self {
            statements: list.into(),
        }
    }
}

impl ToInternedString for Block {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<Block> for Statement {
    fn from(block: Block) -> Self {
        Self::Block(block)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fmt() {
        crate::syntax::ast::test_formatting(
            r#"
        {
            let a = function_call();
            console.log("hello");
        }
        another_statement();
        "#,
        );
        // TODO: Once block labels are implemtned, this should be tested:
        // super::super::test_formatting(
        //     r#"
        //     block_name: {
        //         let a = function_call();
        //         console.log("hello");
        //     }
        //     another_statement();
        //     "#,
        // );
    }
}
