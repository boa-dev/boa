//! Iteration nodes

pub use self::{
    break_node::Break, continue_node::Continue, do_while_loop::DoWhileLoop, for_in_loop::ForInLoop,
    for_loop::ForLoop, for_of_loop::ForOfLoop, while_loop::WhileLoop,
};
use crate::syntax::ast::node::{
    declaration::Declaration, identifier::Identifier, DeclarationPattern,
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum IterableLoopInitializer {
    Identifier(Identifier),
    Var(Declaration),
    Let(Declaration),
    Const(Declaration),
    DeclarationPattern(DeclarationPattern),
}

impl IterableLoopInitializer {
    /// Return the bound names of a for loop initializer.
    ///
    /// The returned list may contain duplicates.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-boundnames
    pub(crate) fn bound_names(&self) -> Vec<Sym> {
        match self {
            IterableLoopInitializer::Let(decl) | IterableLoopInitializer::Const(decl) => match decl
            {
                Declaration::Identifier { ident, .. } => vec![ident.sym()],
                Declaration::Pattern(pattern) => pattern.idents(),
            },
            _ => Vec::new(),
        }
    }
}

impl ToInternedString for IterableLoopInitializer {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            IterableLoopInitializer::Identifier(identifier) => {
                identifier.to_interned_string(interner)
            }
            IterableLoopInitializer::Var(declaration) => {
                format!("var {}", declaration.to_interned_string(interner))
            }
            IterableLoopInitializer::Let(declaration) => {
                format!("let {}", declaration.to_interned_string(interner))
            }
            IterableLoopInitializer::Const(declaration) => {
                format!("const {}", declaration.to_interned_string(interner))
            }
            IterableLoopInitializer::DeclarationPattern(declaration) => {
                declaration.to_interned_string(interner)
            }
        }
    }
}

pub mod break_node;
pub mod continue_node;
pub mod do_while_loop;
pub mod for_in_loop;
pub mod for_loop;
pub mod for_of_loop;
pub mod while_loop;
