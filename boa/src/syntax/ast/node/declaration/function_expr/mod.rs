use crate::{
    builtins::function::FunctionFlags,
    exec::Executable,
    syntax::ast::node::{join_nodes, FormalParameter, Node, StatementList},
    Context, Result, Value,
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `function` expression defines a function with the specified parameters.
///
/// A function created with a function expression is a `Function` object and has all the
/// properties, methods and behavior of `Function`.
///
/// A function can also be created using a declaration (see function expression).
///
/// By default, functions return `undefined`. To return any other value, the function must have
/// a return statement that specifies the value to return.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-function
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct FunctionExpr {
    name: Option<Box<str>>,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl FunctionExpr {
    /// Creates a new function expression
    pub(in crate::syntax) fn new<N, P, B>(name: N, parameters: P, body: B) -> Self
    where
        N: Into<Option<Box<str>>>,
        P: Into<Box<[FormalParameter]>>,
        B: Into<StatementList>,
    {
        Self {
            name: name.into(),
            parameters: parameters.into(),
            body: body.into(),
        }
    }

    /// Gets the name of the function declaration.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(Box::as_ref)
    }

    /// Gets the list of parameters of the function declaration.
    pub fn parameters(&self) -> &[FormalParameter] {
        &self.parameters
    }

    /// Gets the body of the function declaration.
    pub fn body(&self) -> &[Node] {
        self.body.statements()
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        f.write_str("function")?;
        if let Some(ref name) = self.name {
            write!(f, " {}", name)?;
        }
        f.write_str("(")?;
        join_nodes(f, &self.parameters)?;
        f.write_str(") {{")?;

        self.body.display(f, indentation + 1)?;

        writeln!(f, "}}")
    }
}

impl Executable for FunctionExpr {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let val = interpreter.create_function(
            self.parameters().to_vec(),
            self.body().to_vec(),
            FunctionFlags::CALLABLE | FunctionFlags::CONSTRUCTABLE,
        );

        if let Some(name) = self.name() {
            val.set_field("name", Value::from(name));
        }

        Ok(val)
    }
}

impl fmt::Display for FunctionExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<FunctionExpr> for Node {
    fn from(expr: FunctionExpr) -> Self {
        Self::FunctionExpr(expr)
    }
}
