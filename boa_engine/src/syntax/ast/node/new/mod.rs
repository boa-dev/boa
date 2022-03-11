use crate::syntax::ast::node::{Call, Node};
use boa_gc::{Finalize, Trace};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct New {
    call: Call,
}

impl New {
    /// Gets the name of the function call.
    pub fn expr(&self) -> &Node {
        self.call.expr()
    }

    /// Retrieves the arguments passed to the function.
    pub fn args(&self) -> &[Node] {
        self.call.args()
    }

    /// Returns the inner call
    pub(crate) fn call(&self) -> &Call {
        &self.call
    }
}

impl From<Call> for New {
    fn from(call: Call) -> Self {
        Self { call }
    }
}

impl ToInternedString for New {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("new {}", self.call.to_interned_string(interner))
    }
}

impl From<New> for Node {
    fn from(new: New) -> Self {
        Self::New(new)
    }
}
