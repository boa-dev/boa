//! Async Function Declaration.

use crate::{
    exec::Executable,
    syntax::ast::node::{join_nodes, FormalParameter, Node, StatementList},
    BoaProfiler, Context, JsResult, JsValue,
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// An async function is used to specify an action (or series of actions) to perform asynchronously.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-async-function-prototype-properties
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct AsyncFunctionDecl {
    name: Option<Box<str>>,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl AsyncFunctionDecl {
    /// Creates a new async function declaration.
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

    /// Gets the name of the async function declaration.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Gets the list of parameters of the async function declaration.
    pub fn parameters(&self) -> &[FormalParameter] {
        &self.parameters
    }

    /// Gets the body of the async function declaration.
    pub fn body(&self) -> &[Node] {
        self.body.items()
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "async function {}(", name)?,
            None => write!(f, "async function (")?,
        }
        join_nodes(f, &self.parameters)?;
        if self.body().is_empty() {
            f.write_str(") {}")
        } else {
            f.write_str(") {\n")?;
            self.body.display(f, indentation + 1)?;
            write!(f, "{}}}", "    ".repeat(indentation))
        }
    }
}

impl Executable for AsyncFunctionDecl {
    fn run(&self, _: &mut Context) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("AsyncFunctionDecl", "exec");
        // TODO: Implement AsyncFunctionDecl
        Ok(JsValue::undefined())
    }
}

impl From<AsyncFunctionDecl> for Node {
    fn from(decl: AsyncFunctionDecl) -> Self {
        Self::AsyncFunctionDecl(decl)
    }
}

impl fmt::Display for AsyncFunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}
