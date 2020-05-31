//! Expression nodes.

use super::{join_nodes, Node};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.expr)?;
        join_nodes(f, &self.args)?;
        f.write_str(")")
    }
}

impl From<Call> for Node {
    fn from(call: Call) -> Self {
        Self::Call(call)
    }
}

/// The `new` operator lets developers create an instance of a user-defined object type or of
/// one of the built-in object types that has a constructor function.
///
/// The new keyword does the following things:
///  - Creates a blank, plain JavaScript object;
///  - Links (sets the constructor of) this object to another object;
///  - Passes the newly created object from Step 1 as the this context;
///  - Returns this if the function doesn't return its own object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-NewExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct New {
    call: Call,
}

impl New {
    /// Gets the name of the function call.
    pub fn expr(&self) -> &Node {
        &self.call.expr()
    }

    /// Retrieves the arguments passed to the function.
    pub fn args(&self) -> &[Node] {
        &self.call.args()
    }
}

impl From<Call> for New {
    fn from(call: Call) -> Self {
        Self { call }
    }
}

impl fmt::Display for New {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "new {}", self.call)
    }
}

impl From<New> for Node {
    fn from(new: New) -> Self {
        Self::New(new)
    }
}
