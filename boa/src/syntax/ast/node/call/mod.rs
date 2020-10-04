use crate::{
    exec::Executable,
    exec::InterpreterState,
    syntax::ast::node::{join_nodes, Node},
    value::{Type, Value},
    BoaProfiler, Context, Result,
};
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

impl Executable for Call {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("Call", "exec");
        let (this, func) = match self.expr() {
            Node::GetConstField(ref get_const_field) => {
                let mut obj = get_const_field.obj().run(interpreter)?;
                if obj.get_type() != Type::Object {
                    obj = Value::Object(obj.to_object(interpreter)?);
                }
                (obj.clone(), obj.get_field(get_const_field.field()))
            }
            Node::GetField(ref get_field) => {
                let obj = get_field.obj().run(interpreter)?;
                let field = get_field.field().run(interpreter)?;
                (
                    obj.clone(),
                    obj.get_field(field.to_property_key(interpreter)?),
                )
            }
            _ => (
                interpreter.realm().global_obj.clone(),
                self.expr().run(interpreter)?,
            ), // 'this' binding should come from the function's self-contained environment
        };
        let mut v_args = Vec::with_capacity(self.args().len());
        for arg in self.args() {
            if let Node::Spread(ref x) = arg {
                let val = x.run(interpreter)?;
                let mut vals = interpreter.extract_array_properties(&val).unwrap();
                v_args.append(&mut vals);
                break; // after spread we don't accept any new arguments
            }
            v_args.push(arg.run(interpreter)?);
        }

        // execute the function call itself
        let fnct_result = interpreter.call(&func, &this, &v_args);

        // unset the early return flag
        interpreter
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
