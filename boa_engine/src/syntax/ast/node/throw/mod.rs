use crate::syntax::ast::node::Node;
use boa_gc::{Finalize, Trace};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// The `throw` statement throws a user-defined exception.
///
/// Syntax: `throw expression;`
///
/// Execution of the current function will stop (the statements after throw won't be executed),
/// and control will be passed to the first catch block in the call stack. If no catch block
/// exists among caller functions, the program will terminate.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ThrowStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/throw
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Throw {
    expr: Box<Node>,
}

impl Throw {
    pub fn expr(&self) -> &Node {
        &self.expr
    }

    /// Creates a `Throw` AST node.
    pub fn new<V>(val: V) -> Self
    where
        V: Into<Node>,
    {
        Self {
            expr: Box::new(val.into()),
        }
    }
}

impl ToInternedString for Throw {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("throw {}", self.expr.to_interned_string(interner))
    }
}

impl From<Throw> for Node {
    fn from(trw: Throw) -> Self {
        Self::Throw(trw)
    }
}
