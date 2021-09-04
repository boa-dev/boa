use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
    Context, JsResult, JsValue,
};
use std::fmt;

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

impl Executable for Throw {
    #[inline]
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        Err(self.expr().run(context)?)
    }
}

impl fmt::Display for Throw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "throw {}", self.expr)
    }
}

impl From<Throw> for Node {
    fn from(trw: Throw) -> Node {
        Self::Throw(trw)
    }
}
