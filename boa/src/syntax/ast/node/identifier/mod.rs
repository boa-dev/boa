//! Local identifier node.

use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
    BoaProfiler, Context, JsResult, JsValue,
};
use std::fmt;

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
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Identifier {
    ident: Box<str>,
}

impl Executable for Identifier {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("Identifier", "exec");
        context.get_binding_value(self.as_ref())
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.ident, f)
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.ident
    }
}

impl<T> From<T> for Identifier
where
    T: Into<Box<str>>,
{
    fn from(stm: T) -> Self {
        Self { ident: stm.into() }
    }
}

impl From<Identifier> for Node {
    fn from(local: Identifier) -> Self {
        Self::Identifier(local)
    }
}
