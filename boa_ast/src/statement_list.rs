//! Statement list node.

use super::{declaration::Binding, Declaration};
use crate::{
    expression::Identifier,
    statement::Statement,
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, ToIndentedString};
use core::ops::ControlFlow;
use rustc_hash::FxHashSet;
use std::cmp::Ordering;

/// An item inside a [`StatementList`] Parse Node, as defined by the [spec].
///
/// Items in a `StatementList` can be either [`Declaration`]s (functions, classes, let/const declarations)
/// or [`Statement`]s (if, while, var statement).
///
/// [spec]: https://tc39.es/ecma262/#prod-StatementListItem
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Gets the var declared names of this `StatementListItem`.
    #[inline]
    pub fn var_declared_names(&self, vars: &mut FxHashSet<Identifier>) {
        match self {
            StatementListItem::Statement(stmt) => stmt.var_declared_names(vars),
            StatementListItem::Declaration(_) => {}
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

    /// Returns the var declared names of a `StatementList`.
    #[inline]
    pub fn var_declared_names(&self, vars: &mut FxHashSet<Identifier>) {
        for stmt in &*self.statements {
            stmt.var_declared_names(vars);
        }
    }

    /// Returns the lexically declared names of a `StatementList`.
    ///
    /// The returned list may contain duplicates.
    ///
    /// If a declared name originates from a function declaration it is flagged as `true` in the returned list.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-lexicallydeclarednames
    #[must_use]
    pub fn lexically_declared_names(&self) -> Vec<(Identifier, bool)> {
        let mut names = Vec::new();

        for node in self.statements() {
            match node {
                StatementListItem::Statement(_) => {}
                StatementListItem::Declaration(decl) => {
                    names.extend(decl.lexically_declared_names());
                }
            }
        }

        names
    }

    /// Return the top level lexically declared names of a `StatementList`.
    ///
    /// The returned list may contain duplicates.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-toplevellexicallydeclarednames
    #[must_use]
    pub fn lexically_declared_names_top_level(&self) -> Vec<Identifier> {
        let mut names = Vec::new();

        for node in self.statements() {
            if let StatementListItem::Declaration(decl) = node {
                match decl {
                    Declaration::Class(decl) => {
                        if let Some(name) = decl.name() {
                            names.push(name);
                        }
                    }
                    Declaration::Lexical(list) => {
                        for variable in list.variable_list().as_ref() {
                            match variable.binding() {
                                Binding::Identifier(ident) => {
                                    names.push(*ident);
                                }
                                Binding::Pattern(pattern) => {
                                    names.extend(pattern.idents());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        names
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
