use crate::syntax::ast::node::{join_nodes, Node};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The `super` keyword is used to access and call functions on an object's parent.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-SuperCall
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/super
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SuperCall {
    args: Box<[Node]>,
}

impl SuperCall {
    /// Creates a new `SuperCall` AST node.
    pub(crate) fn new<A>(args: A) -> Self
    where
        A: Into<Box<[Node]>>,
    {
        Self { args: args.into() }
    }

    /// Retrieves the arguments of the super call.
    pub(crate) fn args(&self) -> &[Node] {
        &self.args
    }
}

impl ToInternedString for SuperCall {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("super({})", join_nodes(interner, &self.args))
    }
}

impl From<SuperCall> for Node {
    fn from(call: SuperCall) -> Self {
        Self::SuperCall(call)
    }
}
