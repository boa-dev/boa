//! Block AST node.

use super::Statement;
use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::syntax::ast::{expression::Identifier, ContainsSymbol, StatementList};
use boa_interner::{Interner, ToIndentedString};
use std::ops::ControlFlow;

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
    #[inline]
    pub(crate) fn statement_list(&self) -> &StatementList {
        &self.statements
    }

    /// Get the lexically declared names of the block.
    #[inline]
    pub(crate) fn lexically_declared_names(&self) -> Vec<(Identifier, bool)> {
        self.statements.lexically_declared_names()
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
    #[inline]
    fn from(list: T) -> Self {
        Self {
            statements: list.into(),
        }
    }
}

impl ToIndentedString for Block {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "{{\n{}{}}}",
            self.statements
                .to_indented_string(interner, indentation + 1),
            "    ".repeat(indentation)
        )
    }
}

impl From<Block> for Statement {
    #[inline]
    fn from(block: Block) -> Self {
        Self::Block(block)
    }
}

impl VisitWith for Block {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_statement_list(&self.statements)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_statement_list_mut(&mut self.statements)
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
