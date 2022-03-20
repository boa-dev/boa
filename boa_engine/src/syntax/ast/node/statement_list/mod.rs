//! Statement list node.

use crate::syntax::ast::node::{Declaration, Node};
use boa_gc::{unsafe_empty_trace, Finalize, Trace};
use boa_interner::{Interner, Sym, ToInternedString};
use std::{ops::Deref, rc::Rc};

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
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
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

    pub fn lexically_declared_names(&self, interner: &Interner) -> FxHashSet<Sym> {
        let mut set = FxHashSet::default();
        for stmt in self.items() {
            if let Node::LetDeclList(decl_list) | Node::ConstDeclList(decl_list) = stmt {
                for decl in decl_list.as_ref() {
                    // It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains any duplicate entries.
                    // https://tc39.es/ecma262/#sec-block-static-semantics-early-errors
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            if !set.insert(ident.sym()) {
                                unreachable!(
                                    "Redeclaration of {}",
                                    interner.resolve_expect(ident.sym())
                                );
                            }
                        }
                        Declaration::Pattern(p) => {
                            for ident in p.idents().iter().copied() {
                                if !set.insert(ident) {
                                    unreachable!(
                                        "Redeclaration of {}",
                                        interner.resolve_expect(ident)
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        set
    }

    pub fn function_declared_names(&self) -> FxHashSet<Sym> {
        let mut set = FxHashSet::default();
        for stmt in self.items() {
            if let Node::FunctionDecl(decl) = stmt {
                set.insert(decl.name());
            }
        }
        set
    }

    pub fn var_declared_names(&self) -> FxHashSet<Sym> {
        let mut set = FxHashSet::default();
        for stmt in self.items() {
            if let Node::VarDeclList(decl_list) = stmt {
                for decl in decl_list.as_ref() {
                    match decl {
                        Declaration::Identifier { ident, .. } => {
                            set.insert(ident.sym());
                        }
                        Declaration::Pattern(p) => set.extend(p.idents().into_iter()),
                    }
                }
            }
        }
        set
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

// List of statements wrapped with Rc. We need this for self mutating functions.
// Since we need to cheaply clone the function body and drop the borrow of the function object to
// mutably borrow the function object and call this cloned function body
#[derive(Clone, Debug, Finalize, PartialEq)]
pub struct RcStatementList(Rc<StatementList>);

impl Deref for RcStatementList {
    type Target = StatementList;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<StatementList> for RcStatementList {
    #[inline]
    fn from(statementlist: StatementList) -> Self {
        Self(Rc::from(statementlist))
    }
}

// SAFETY: This is safe for types not containing any `Trace` types.
unsafe impl Trace for RcStatementList {
    unsafe_empty_trace!();
}
