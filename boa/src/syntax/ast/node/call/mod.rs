use crate::{
    builtins::iterable,
    exec::Executable,
    exec::InterpreterState,
    gc::{Finalize, Trace},
    syntax::ast::node::{join_nodes, Node},
    BoaProfiler, Context, JsResult, JsValue,
};
use std::fmt;

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

impl Executable for Call {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("Call", "exec");
        let (this, func) = match self.expr() {
            Node::GetConstField(ref get_const_field) => {
                let mut obj = get_const_field.obj().run(context)?;
                if !obj.is_object() {
                    obj = JsValue::Object(obj.to_object(context)?);
                }
                (
                    obj.clone(),
                    obj.get_field(get_const_field.field(), context)?,
                )
            }
            Node::GetField(ref get_field) => {
                let mut obj = get_field.obj().run(context)?;
                if !obj.is_object() {
                    obj = JsValue::Object(obj.to_object(context)?);
                }
                let field = get_field.field().run(context)?;
                (
                    obj.clone(),
                    obj.get_field(field.to_property_key(context)?, context)?,
                )
            }
            _ => (
                // 'this' binding should come from the function's self-contained environment
                context.global_object().into(),
                self.expr().run(context)?,
            ),
        };
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

        // execute the function call itself
        let fnct_result = context.call(&func, &this, &v_args);

        // unset the early return flag
        context
            .executor()
            .set_current_state(InterpreterState::Executing);

        fnct_result
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
