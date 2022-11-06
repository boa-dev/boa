//! Iteration nodes

mod r#break;
mod r#continue;
mod do_while_loop;
mod for_in_loop;
mod for_loop;
mod for_of_loop;
mod while_loop;

use crate::{
    declaration::Binding,
    expression::{access::PropertyAccess, Identifier},
    pattern::Pattern,
};
use core::ops::ControlFlow;

pub use self::{
    do_while_loop::DoWhileLoop,
    for_in_loop::ForInLoop,
    for_loop::{ForLoop, ForLoopInitializer},
    for_of_loop::ForOfLoop,
    r#break::Break,
    r#continue::Continue,
    while_loop::WhileLoop,
};
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use boa_interner::{Interner, ToInternedString};

/// A `for-in`, `for-of` and `for-await-of` loop initializer.
///
/// The [spec] specifies only single bindings for the listed types of loops, which makes us
/// unable to use plain `LexicalDeclaration`s or `VarStatement`s as initializers, since those
/// can have more than one binding.
///
/// [spec]: https://tc39.es/ecma262/#prod-ForInOfStatement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
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

impl VisitWith for IterableLoopInitializer {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            IterableLoopInitializer::Identifier(id) => visitor.visit_identifier(id),
            IterableLoopInitializer::Access(pa) => visitor.visit_property_access(pa),
            IterableLoopInitializer::Var(b)
            | IterableLoopInitializer::Let(b)
            | IterableLoopInitializer::Const(b) => visitor.visit_binding(b),
            IterableLoopInitializer::Pattern(p) => visitor.visit_pattern(p),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            IterableLoopInitializer::Identifier(id) => visitor.visit_identifier_mut(id),
            IterableLoopInitializer::Access(pa) => visitor.visit_property_access_mut(pa),
            IterableLoopInitializer::Var(b)
            | IterableLoopInitializer::Let(b)
            | IterableLoopInitializer::Const(b) => visitor.visit_binding_mut(b),
            IterableLoopInitializer::Pattern(p) => visitor.visit_pattern_mut(p),
        }
    }
}
