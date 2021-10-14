//! Async Generator Declaration

use crate::{
    exec::Executable,
    syntax::ast::node::{join_nodes, FormalParameter, Node, StatementList},
    BoaProfiler, Context, JsResult, JsValue,
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The 'async function*' defines an async generator function
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorMethod

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct AsyncGeneratorDecl {
    name: Box<str>,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl AsyncGeneratorDecl {
    /// Creates a new async generator declaration.
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

    /// Gets the name of the async function declaration.
    pub fn name(&self) -> &str {
        &self.name
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
        write!(f, "async function* {}(", self.name())?;
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

impl Executable for AsyncGeneratorDecl {
    fn run(&self, _: &mut Context) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("AsyncGeneratorDecl", "exec");
        //TODO: Implement AsyncGeneratorDecl
        Ok(JsValue::undefined())
    }
}

impl From<AsyncGeneratorDecl> for Node {
    fn from(decl: AsyncGeneratorDecl) -> Self {
        Self::AsyncGeneratorDecl(decl)
    }
}

impl fmt::Display for AsyncGeneratorDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}
