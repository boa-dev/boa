//! Local identifier Expression.

use crate::{
    string::ToStringEscaped,
    syntax::{ast::Position, parser::ParseError},
};
use boa_interner::{Interner, Sym, ToInternedString};

use super::Expression;

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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "deser", serde(transparent))]
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
    pub fn new(ident: Sym) -> Self {
        Self { ident }
    }

    /// Retrieves the identifier's string symbol in the interner.
    #[inline]
    pub fn sym(self) -> Sym {
        self.ident
    }

    /// Returns an error if `arguments` or `eval` are used as identifier in strict mode.
    pub(crate) fn check_strict_arguments_or_eval(
        self,
        position: Position,
    ) -> Result<(), ParseError> {
        match self.ident {
            Sym::ARGUMENTS => Err(ParseError::general(
                "unexpected identifier 'arguments' in strict mode",
                position,
            )),
            Sym::EVAL => Err(ParseError::general(
                "unexpected identifier 'eval' in strict mode",
                position,
            )),
            _ => Ok(()),
        }
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
