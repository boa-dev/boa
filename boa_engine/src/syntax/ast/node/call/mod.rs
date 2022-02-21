use crate::syntax::ast::node::{join_nodes, Node};
use boa_gc::{Finalize, Trace};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Calling the function actually performs the specified actions with the indicated parameters.
///
/// Defining a function does not execute it. Defining it simply names the function and
/// specifies what to do when the function is called. Functions must be in scope when they are
/// called, but the function declaration can be hoisted. The scope of a function is the
/// function in which it is declared (or the entire program, if it is declared at the top
/// level).
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-CallExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Functions#Calling_functions
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Call {
    expr: Box<Node>,
    args: Box<[Node]>,
}

impl Call {
    /// Creates a new `Call` AST node.
    pub fn new<E, A>(expr: E, args: A) -> Self
    where
        E: Into<Node>,
        A: Into<Box<[Node]>>,
    {
        Self {
            expr: Box::new(expr.into()),
            args: args.into(),
        }
    }

    /// Gets the name of the function call.
    pub fn expr(&self) -> &Node {
        &self.expr
    }

    /// Retrieves the arguments passed to the function.
    pub fn args(&self) -> &[Node] {
        &self.args
    }
}

impl ToInternedString for Call {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{}({})",
            self.expr.to_interned_string(interner),
            join_nodes(interner, &self.args)
        )
    }
}

impl From<Call> for Node {
    fn from(call: Call) -> Self {
        Self::Call(call)
    }
}
