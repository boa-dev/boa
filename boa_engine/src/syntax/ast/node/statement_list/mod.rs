//! Statement list node.

use crate::syntax::ast::node::{Declaration, Node};
use boa_interner::{Interner, Sym, ToInternedString};

use rustc_hash::FxHashSet;
#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct StatementList {
    items: Box<[Node]>,
    strict: bool,
}

impl StatementList {
    /// Gets the list of items.
    #[inline]
    pub fn items(&self) -> &[Node] {
        &self.items
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
    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indentation: usize,
    ) -> String {
        let mut buf = String::new();
        // Print statements
        for node in self.items.iter() {
            // We rely on the node to add the correct indent.
            buf.push_str(&node.to_indented_string(interner, indentation));

            match node {
                Node::Block(_) | Node::If(_) | Node::Switch(_) | Node::WhileLoop(_) => {}
                _ => buf.push(';'),
            }

            buf.push('\n');
        }
        buf
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
    pub(crate) fn lexically_declared_names(&self) -> Vec<(Sym, bool)> {
        let mut names = Vec::new();

        for node in self.items() {
            match node {
                Node::FunctionDecl(decl) => {
                    names.push((decl.name(), true));
                }
                Node::GeneratorDecl(decl) => {
                    names.push((decl.name(), false));
                }
                Node::AsyncFunctionDecl(decl) => {
                    names.push((decl.name(), false));
                }
                Node::AsyncGeneratorDecl(decl) => {
                    names.push((decl.name(), false));
                }
                Node::ClassDecl(decl) => {
                    names.push((decl.name(), false));
                }
                Node::LetDeclList(decl_list) | Node::ConstDeclList(decl_list) => match decl_list {
                    super::DeclarationList::Const(declarations)
                    | super::DeclarationList::Let(declarations) => {
                        for decl in declarations.iter() {
                            match decl {
                                Declaration::Identifier { ident, .. } => {
                                    names.push((ident.sym(), false));
                                }
                                Declaration::Pattern(pattern) => {
                                    names.extend(
                                        pattern.idents().into_iter().map(|name| (name, false)),
                                    );
                                }
                            }
                        }
                    }
                    super::DeclarationList::Var(_) => unreachable!(),
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
    pub(crate) fn lexically_declared_names_top_level(&self) -> Vec<Sym> {
        let mut names = Vec::new();

        for node in self.items() {
            match node {
                Node::ClassDecl(decl) => {
                    names.push(decl.name());
                }
                Node::LetDeclList(decl_list) | Node::ConstDeclList(decl_list) => match decl_list {
                    super::DeclarationList::Const(declarations)
                    | super::DeclarationList::Let(declarations) => {
                        for decl in declarations.iter() {
                            match decl {
                                Declaration::Identifier { ident, .. } => {
                                    names.push(ident.sym());
                                }
                                Declaration::Pattern(pattern) => {
                                    names.extend(pattern.idents());
                                }
                            }
                        }
                    }
                    super::DeclarationList::Var(_) => unreachable!(),
                },
                _ => {}
            }
        }

        names
    }

    /// Return the variable declared names of a `StatementList`.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-vardeclarednames
    pub(crate) fn var_declared_names_new(&self, vars: &mut FxHashSet<Sym>) {
        for node in self.items() {
            node.var_declared_names(vars);
        }
    }
}

impl<T> From<T> for StatementList
where
    T: Into<Box<[Node]>>,
{
    fn from(stm: T) -> Self {
        Self {
            items: stm.into(),
            strict: false,
        }
    }
}

impl ToInternedString for StatementList {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}
