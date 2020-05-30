use super::Node;
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `return` statement ends function execution and specifies a value to be returned to the
/// function caller.
///
/// Syntax: `return [expression];`
///
/// `expression`:
///  > The expression whose value is to be returned. If omitted, `undefined` is returned
///  > instead.
///
/// When a `return` statement is used in a function body, the execution of the function is
/// stopped. If specified, a given value is returned to the function caller.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ReturnStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Return {
    value: Option<Box<Node>>,
}

impl Return {
    pub fn value(&self) -> &Option<Box<Node>> {
        &self.value
    }

    /// Creates a `Return` AST node.
    pub fn new<E, OE>(expr: OE) -> Self
    where
        E: Into<Node>,
        OE: Into<Option<E>>,
    {
        Self {
            value: expr.into().map(E::into).map(Box::new)
        }
    }
}

impl From<Return> for Node {
    fn from(return_smt: Return) -> Node {
        Node::Return(return_smt)
    }
}