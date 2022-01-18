//! Local identifier node.

use crate::{
    gc::{empty_trace, Finalize, Trace},
    syntax::ast::node::Node,
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "deser", serde(transparent))]
#[derive(Debug, Clone, Copy, Finalize, PartialEq)]
pub struct Identifier {
    ident: Sym,
}

impl Identifier {
    /// Creates a new identifier AST node.
    pub fn new(ident: Sym) -> Self {
        Self { ident }
    }

    /// Retrieves the identifier's string symbol in the interner.
    pub fn sym(self) -> Sym {
        self.ident
    }
}

impl ToInternedString for Identifier {
    fn to_interned_string(&self, interner: &Interner) -> String {
        interner.resolve_expect(self.ident).to_owned()
    }
}

unsafe impl Trace for Identifier {
    empty_trace!();
}

impl From<Sym> for Identifier {
    fn from(sym: Sym) -> Self {
        Self { ident: sym }
    }
}

impl From<Identifier> for Node {
    fn from(local: Identifier) -> Self {
        Self::Identifier(local)
    }
}
