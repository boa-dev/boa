//! Statement list node.

use std::cmp::Ordering;

use crate::syntax::ast::{expression::Identifier, statement::Statement, ContainsSymbol};
use boa_interner::{Interner, ToInternedString};

use rustc_hash::FxHashSet;

use super::{declaration::Binding, Declaration};

#[cfg(test)]
mod tests;

#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum StatementListItem {
    Statement(Statement),
    Declaration(Declaration),
}

impl StatementListItem {
    /// Returns a node ordering based on the hoistability of each statement.
    pub(crate) fn hoistable_order(a: &Self, b: &Self) -> Ordering {
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

impl From<Statement> for StatementListItem {
    fn from(stmt: Statement) -> Self {
        StatementListItem::Statement(stmt)
    }
}

impl From<Declaration> for StatementListItem {
    fn from(decl: Declaration) -> Self {
        StatementListItem::Declaration(decl)
    }
}

impl StatementListItem {
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

    pub(crate) fn var_declared_names(&self, vars: &mut FxHashSet<Identifier>) {
        match self {
            StatementListItem::Statement(stmt) => stmt.var_declared_names(vars),
            StatementListItem::Declaration(_) => {}
        }
    }

    /// Returns true if the node contains a identifier reference named 'arguments'.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            StatementListItem::Statement(stmt) => stmt.contains_arguments(),
            StatementListItem::Declaration(decl) => decl.contains_arguments(),
        }
    }

    /// Returns `true` if the node contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            StatementListItem::Statement(stmt) => stmt.contains(symbol),
            StatementListItem::Declaration(decl) => decl.contains(symbol),
        }
    }
}

/// List of statements.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-StatementList
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatementList {
    statements: Box<[StatementListItem]>,
    strict: bool,
}

impl StatementList {
    /// Gets the list of statements.
    #[inline]
    pub fn statements(&self) -> &[StatementListItem] {
        &self.statements
    }

    /// Get the strict mode.
    #[inline]
    pub fn strict(&self) -> bool {
        self.strict
    }

    /// Set the strict mode.
    #[inline]
    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    /// Implements the display formatting with indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = String::new();
        // Print statements
        for item in self.statements.iter() {
            // We rely on the node to add the correct indent.
            buf.push_str(&item.to_indented_string(interner, indentation));

            match item {
                StatementListItem::Statement(
                    Statement::Var(_)
                    | Statement::Expression(_)
                    | Statement::Continue(_)
                    | Statement::Break(_)
                    | Statement::Return(_)
                    | Statement::Throw(_)
                    | Statement::DoWhileLoop(_),
                )
                | StatementListItem::Declaration(Declaration::Lexical(_)) => buf.push(';'),
                _ => {}
            }

            buf.push('\n');
        }
        buf
    }

    pub(crate) fn var_declared_names(&self, vars: &mut FxHashSet<Identifier>) {
        for stmt in &*self.statements {
            stmt.var_declared_names(vars);
        }
    }

    /// Return the lexically declared names of a `StatementList`.
    ///
    /// The returned list may contain duplicates.
    ///
    /// If a declared name originates from a function declaration it is flagged as `true` in the returned list.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-lexicallydeclarednames
    pub(crate) fn lexically_declared_names(&self) -> Vec<(Identifier, bool)> {
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
    pub(crate) fn lexically_declared_names_top_level(&self) -> Vec<Identifier> {
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

    /// Returns true if the node contains a identifier reference named 'arguments'.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.statements
            .iter()
            .any(StatementListItem::contains_arguments)
    }

    /// Returns `true` if the node contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.statements.iter().any(|stmt| stmt.contains(symbol))
    }
}

impl From<Box<[StatementListItem]>> for StatementList {
    fn from(stm: Box<[StatementListItem]>) -> Self {
        Self {
            statements: stm,
            strict: false,
        }
    }
}

impl From<Vec<StatementListItem>> for StatementList {
    fn from(stm: Vec<StatementListItem>) -> Self {
        Self {
            statements: stm.into(),
            strict: false,
        }
    }
}

impl ToInternedString for StatementList {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}
