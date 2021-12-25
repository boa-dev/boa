use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::{join_nodes, FormalParameter, Node, StatementList},
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The `function` declaration (function statement) defines a function with the specified
/// parameters.
///
/// A function created with a function declaration is a `Function` object and has all the
/// properties, methods and behavior of `Function`.
///
/// A function can also be created using an expression (see [function expression][func_expr]).
///
/// By default, functions return `undefined`. To return any other value, the function must have
/// a return statement that specifies the value to return.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-terms-and-definitions-function
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
/// [func_expr]: ../enum.Node.html#variant.FunctionExpr
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct FunctionDecl {
    name: Box<str>,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl FunctionDecl {
    /// Creates a new function declaration.
    pub(in crate::syntax) fn new<N, P, B>(name: N, parameters: P, body: B) -> Self
    where
        N: Into<Box<str>>,
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
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the list of parameters of the function declaration.
    pub fn parameters(&self) -> &[FormalParameter] {
        &self.parameters
    }

    /// Gets the body of the function declaration.
    pub fn body(&self) -> &StatementList {
        &self.body
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        write!(f, "function {}(", self.name)?;
        join_nodes(f, &self.parameters)?;
        if self.body().items().is_empty() {
            f.write_str(") {}")
        } else {
            f.write_str(") {\n")?;
            self.body.display(f, indentation + 1)?;
            write!(f, "{}}}", "    ".repeat(indentation))
        }
    }
}

impl From<FunctionDecl> for Node {
    fn from(decl: FunctionDecl) -> Self {
        Self::FunctionDecl(decl)
    }
}

impl fmt::Display for FunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}
