//! Statement list node.

use crate::syntax::ast::{expression::Identifier, statement::Statement, ContainsSymbol};
use boa_interner::{Interner, ToInternedString};

use rustc_hash::FxHashSet;

use super::{declaration::Binding, DeclarationList};

#[cfg(test)]
mod tests;

/// List of statements.
///
/// Similar to `Node::Block` but without the braces.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-StatementList
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatementList {
    statements: Box<[Statement]>,
    strict: bool,
}

impl StatementList {
    /// Gets the list of statements.
    #[inline]
    pub fn statements(&self) -> &[Statement] {
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
        for stmt in self.statements.iter() {
            // We rely on the node to add the correct indent.
            buf.push_str(&stmt.to_indented_string(interner, indentation));

            match stmt {
                Statement::DeclarationList(_)
                | Statement::Empty
                | Statement::Expression(_)
                | Statement::Continue(_)
                | Statement::Break(_)
                | Statement::Return(_)
                | Statement::Throw(_)
                | Statement::DoWhileLoop(_) => buf.push(';'),
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
                Statement::Function(decl) => {
                    if let Some(name) = decl.name() {
                        names.push((name, true));
                    }
                }
                Statement::Generator(decl) => {
                    if let Some(name) = decl.name() {
                        names.push((name, false));
                    }
                }
                Statement::AsyncFunction(decl) => {
                    if let Some(name) = decl.name() {
                        names.push((name, false));
                    }
                }
                Statement::AsyncGenerator(decl) => {
                    if let Some(name) = decl.name() {
                        names.push((name, false));
                    }
                }
                Statement::Class(decl) => {
                    if let Some(name) = decl.name() {
                        names.push((name, false));
                    }
                }
                Statement::DeclarationList(list) => match list {
                    super::DeclarationList::Const(declarations)
                    | super::DeclarationList::Let(declarations) => {
                        for decl in declarations.iter() {
                            match decl.binding() {
                                Binding::Identifier(ident) => {
                                    names.push((*ident, false));
                                }
                                Binding::Pattern(pattern) => {
                                    names.extend(
                                        pattern.idents().into_iter().map(|name| (name, false)),
                                    );
                                }
                            }
                        }
                    }
                    super::DeclarationList::Var(_) => {}
                },

                _ => {}
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
            match node {
                Statement::Class(decl) => {
                    if let Some(name) = decl.name() {
                        names.push(name);
                    }
                }
                Statement::DeclarationList(
                    DeclarationList::Const(declarations) | DeclarationList::Let(declarations),
                ) => {
                    for decl in &**declarations {
                        match decl.binding() {
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
        self.statements.iter().any(Statement::contains_arguments)
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

impl<T> From<T> for StatementList
where
    T: Into<Box<[Statement]>>,
{
    fn from(stm: T) -> Self {
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
