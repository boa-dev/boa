//! Statement list node.

use super::Declaration;
use crate::{
    statement::Statement,
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, ToIndentedString};
use core::ops::ControlFlow;

use std::cmp::Ordering;

/// An item inside a [`StatementList`] Parse Node, as defined by the [spec].
///
/// Items in a `StatementList` can be either [`Declaration`]s (functions, classes, let/const declarations)
/// or [`Statement`]s (if, while, var statement).
///
/// [spec]: https://tc39.es/ecma262/#prod-StatementListItem
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum StatementListItem {
    /// See [`Statement`].
    Statement(Statement),
    /// See [`Declaration`].
    Declaration(Declaration),
}

impl StatementListItem {
    /// Returns a node ordering based on the hoistability of each statement.
    #[must_use]
    pub fn hoistable_order(a: &Self, b: &Self) -> Ordering {
        match (a, b) {
            (
                Self::Declaration(Declaration::Function(_)),
                Self::Declaration(Declaration::Function(_)),
            ) => Ordering::Equal,
            (_, Self::Declaration(Declaration::Function(_))) => Ordering::Greater,
            (Self::Declaration(Declaration::Function(_)), _) => Ordering::Less,

            (_, _) => Ordering::Equal,
        }
    }
}

impl ToIndentedString for StatementListItem {
    /// Creates a string of the value of the node with the given indentation. For example, an
    /// indent level of 2 would produce this:
    ///
    /// ```js
    ///         function hello() {
    ///             console.log("hello");
    ///         }
    ///         hello();
    ///         a = 2;
    /// ```
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "    ".repeat(indentation);

        match self {
            StatementListItem::Statement(stmt) => {
                buf.push_str(&stmt.to_no_indent_string(interner, indentation));
            }
            StatementListItem::Declaration(decl) => {
                buf.push_str(&decl.to_indented_string(interner, indentation));
            }
        }

        buf
    }
}

impl From<Statement> for StatementListItem {
    #[inline]
    fn from(stmt: Statement) -> Self {
        StatementListItem::Statement(stmt)
    }
}

impl From<Declaration> for StatementListItem {
    #[inline]
    fn from(decl: Declaration) -> Self {
        StatementListItem::Declaration(decl)
    }
}

impl VisitWith for StatementListItem {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            StatementListItem::Statement(statement) => visitor.visit_statement(statement),
            StatementListItem::Declaration(declaration) => visitor.visit_declaration(declaration),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            StatementListItem::Statement(statement) => visitor.visit_statement_mut(statement),
            StatementListItem::Declaration(declaration) => {
                visitor.visit_declaration_mut(declaration)
            }
        }
    }
}

/// List of statements.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-StatementList
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatementList {
    statements: Box<[StatementListItem]>,
    strict: bool,
}

impl StatementList {
    /// Gets the list of statements.
    #[inline]
    #[must_use]
    pub fn statements(&self) -> &[StatementListItem] {
        &self.statements
    }

    /// Get the strict mode.
    #[inline]
    #[must_use]
    pub fn strict(&self) -> bool {
        self.strict
    }

    /// Set the strict mode.
    #[inline]
    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }
}

impl From<Box<[StatementListItem]>> for StatementList {
    #[inline]
    fn from(stm: Box<[StatementListItem]>) -> Self {
        Self {
            statements: stm,
            strict: false,
        }
    }
}

impl From<Vec<StatementListItem>> for StatementList {
    #[inline]
    fn from(stm: Vec<StatementListItem>) -> Self {
        Self {
            statements: stm.into(),
            strict: false,
        }
    }
}

impl ToIndentedString for StatementList {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = String::new();
        // Print statements
        for item in self.statements.iter() {
            // We rely on the node to add the correct indent.
            buf.push_str(&item.to_indented_string(interner, indentation));

            buf.push('\n');
        }
        buf
    }
}

impl VisitWith for StatementList {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        for statement in self.statements.iter() {
            try_break!(visitor.visit_statement_list_item(statement));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        for statement in self.statements.iter_mut() {
            try_break!(visitor.visit_statement_list_item_mut(statement));
        }
        ControlFlow::Continue(())
    }
}

#[cfg(feature = "fuzz")]
impl<'a> arbitrary::Arbitrary<'a> for StatementList {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            statements: u.arbitrary()?,
            strict: false, // disable strictness; this is *not* in source data
        })
    }
}
