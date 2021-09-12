use crate::{
    builtins::iterable,
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::node::{Call, Node},
    value::JsValue,
    BoaProfiler, Context, JsResult,
};
use std::fmt;

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
}

impl Executable for New {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("New", "exec");

        let func_object = self.expr().run(context)?;
        let mut v_args = Vec::with_capacity(self.args().len());
        for arg in self.args() {
            if let Node::Spread(ref x) = arg {
                let val = x.run(context)?;
                let iterator_record = iterable::get_iterator(&val, context)?;
                loop {
                    let next = iterator_record.next(context)?;
                    if next.done {
                        break;
                    }
                    let next_value = next.value;
                    v_args.push(next_value);
                }
                break; // after spread we don't accept any new arguments
            } else {
                v_args.push(arg.run(context)?);
            }
        }

        match func_object {
            JsValue::Object(ref object) => {
                object.construct(&v_args, &object.clone().into(), context)
            }
            _ => context
                .throw_type_error(format!("{} is not a constructor", self.expr().to_string(),)),
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
