//! Iteration nodes

pub mod r#break;
pub mod r#continue;
pub mod do_while_loop;
pub mod for_in_loop;
pub mod for_loop;
pub mod for_of_loop;
pub mod while_loop;

use crate::syntax::ast::expression::Identifier;

pub use self::{
    do_while_loop::DoWhileLoop, for_in_loop::ForInLoop, for_loop::ForLoop, for_of_loop::ForOfLoop,
    r#break::Break, r#continue::Continue, while_loop::WhileLoop,
};
use boa_interner::{Interner, Sym, ToInternedString};

use super::{declaration::Binding, ContainsSymbol};

#[cfg(test)]
mod tests;

#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum IterableLoopInitializer {
    // TODO: This should also accept property accessors
    Identifier(Identifier),
    Var(Binding),
    Let(Binding),
    Const(Binding),
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
    pub(crate) fn bound_names(&self) -> Vec<Identifier> {
        match self {
            Self::Let(binding) | Self::Const(binding) => binding.idents(),
            _ => Vec::new(),
        }
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            Self::Identifier(ident) => *ident == Sym::ARGUMENTS,
            Self::Var(bind) | Self::Let(bind) | Self::Const(bind) => bind.contains_arguments(),
        }
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            Self::Var(declaration) | Self::Let(declaration) | Self::Const(declaration) => {
                declaration.contains(symbol)
            }
            Self::Identifier(_) => false,
        }
    }
}

impl ToInternedString for IterableLoopInitializer {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let (binding, pre) = match self {
            Self::Identifier(ident) => return ident.to_interned_string(interner),
            Self::Var(binding) => (binding, "var"),
            Self::Let(binding) => (binding, "let"),
            Self::Const(binding) => (binding, "const"),
        };

        format!("{pre} {}", binding.to_interned_string(interner))
    }
}
