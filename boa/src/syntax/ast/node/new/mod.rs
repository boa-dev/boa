use crate::{
    exec::Executable,
    syntax::ast::node::{Call, Node},
    value::Value,
    BoaProfiler, Context, Result,
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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

impl Executable for New {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("New", "exec");

        let func_object = self.expr().run(interpreter)?;
        let mut v_args = Vec::with_capacity(self.args().len());
        for arg in self.args() {
            v_args.push(arg.run(interpreter)?);
        }

        match func_object {
            Value::Object(ref object) => object.construct(&v_args, interpreter),
            _ => Ok(Value::undefined()),
        }
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
