//! Local identifier Expression.

use crate::{
    visitor::{VisitWith, Visitor, VisitorMut},
    ToStringEscaped,
};
use boa_interner::{Interner, Sym, ToInternedString};
use core::ops::ControlFlow;

use super::Expression;

/// List of reserved keywords exclusive to strict mode.
pub const RESERVED_IDENTIFIERS_STRICT: [Sym; 9] = [
    Sym::IMPLEMENTS,
    Sym::INTERFACE,
    Sym::LET,
    Sym::PACKAGE,
    Sym::PRIVATE,
    Sym::PROTECTED,
    Sym::PUBLIC,
    Sym::STATIC,
    Sym::YIELD,
];

/// An `identifier` is a sequence of characters in the code that identifies a variable,
/// function, or property.
///
/// In JavaScript, identifiers are case-sensitive and can contain Unicode letters, $, _, and
/// digits (0-9), but may not start with a digit.
///
/// An identifier differs from a string in that a string is data, while an identifier is part
/// of the code. In JavaScript, there is no way to convert identifiers to strings, but
/// sometimes it is possible to parse strings into identifiers.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-Identifier
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Identifier
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Identifier {
    ident: Sym,
}

impl PartialEq<Sym> for Identifier {
    #[inline]
    fn eq(&self, other: &Sym) -> bool {
        self.ident == *other
    }
}

impl PartialEq<Identifier> for Sym {
    #[inline]
    fn eq(&self, other: &Identifier) -> bool {
        *self == other.ident
    }
}

impl Identifier {
    /// Creates a new identifier AST Expression.
    #[inline]
    #[must_use]
    pub fn new(ident: Sym) -> Self {
        Self { ident }
    }

    /// Retrieves the identifier's string symbol in the interner.
    #[inline]
    #[must_use]
    pub fn sym(self) -> Sym {
        self.ident
    }
}

impl ToInternedString for Identifier {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        interner.resolve_expect(self.ident).join(
            String::from,
            ToStringEscaped::to_string_escaped,
            true,
        )
    }
}

impl From<Sym> for Identifier {
    #[inline]
    fn from(sym: Sym) -> Self {
        Self { ident: sym }
    }
}

impl From<Identifier> for Expression {
    #[inline]
    fn from(local: Identifier) -> Self {
        Self::Identifier(local)
    }
}

impl VisitWith for Identifier {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_sym(&self.ident)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_sym_mut(&mut self.ident)
    }
}
