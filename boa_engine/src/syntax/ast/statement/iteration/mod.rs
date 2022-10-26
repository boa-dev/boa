//! Iteration nodes

mod r#break;
mod r#continue;
mod do_while_loop;
mod for_in_loop;
mod for_loop;
mod for_of_loop;
mod while_loop;

use crate::syntax::ast::{
    declaration::Binding,
    expression::{access::PropertyAccess, Identifier},
    pattern::Pattern,
};

pub use self::{
    do_while_loop::DoWhileLoop,
    for_in_loop::ForInLoop,
    for_loop::{ForLoop, ForLoopInitializer},
    for_of_loop::ForOfLoop,
    r#break::Break,
    r#continue::Continue,
    while_loop::WhileLoop,
};
use boa_interner::{Interner, Sym, ToInternedString};

use super::ContainsSymbol;

#[cfg(test)]
mod tests;

/// A `for-in`, `for-of` and `for-await-of` loop initializer.
///
/// The [spec] specifies only single bindings for the listed types of loops, which makes us
/// unable to use plain `LexicalDeclaration`s or `VarStatement`s as initializers, since those
/// can have more than one binding.
///
/// [spec]: https://tc39.es/ecma262/#prod-ForInOfStatement
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum IterableLoopInitializer {
    /// An already declared variable.
    Identifier(Identifier),
    /// A property access.
    Access(PropertyAccess),
    /// A new var declaration.
    Var(Binding),
    /// A new let declaration.
    Let(Binding),
    /// A new const declaration.
    Const(Binding),
    /// A pattern with already declared variables.
    Pattern(Pattern),
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
            Self::Access(access) => access.contains_arguments(),
            Self::Var(bind) | Self::Let(bind) | Self::Const(bind) => bind.contains_arguments(),
            Self::Pattern(pattern) => pattern.contains_arguments(),
        }
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            Self::Var(declaration) | Self::Let(declaration) | Self::Const(declaration) => {
                declaration.contains(symbol)
            }
            Self::Pattern(pattern) => pattern.contains(symbol),
            Self::Access(access) => access.contains(symbol),
            Self::Identifier(_) => false,
        }
    }
}

impl ToInternedString for IterableLoopInitializer {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let (binding, pre) = match self {
            Self::Identifier(ident) => return ident.to_interned_string(interner),
            Self::Pattern(pattern) => return pattern.to_interned_string(interner),
            Self::Access(access) => return access.to_interned_string(interner),
            Self::Var(binding) => (binding, "var"),
            Self::Let(binding) => (binding, "let"),
            Self::Const(binding) => (binding, "const"),
        };

        format!("{pre} {}", binding.to_interned_string(interner))
    }
}
