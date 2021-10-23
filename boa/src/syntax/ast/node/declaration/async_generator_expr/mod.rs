//! Async Generator Expression

use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::node::{join_nodes, FormalParameter, Node, StatementList},
    Context, JsResult, JsValue,
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The `async function*` keyword can be used to define a generator function inside an expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorExpression
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct AsyncGeneratorExpr {
    name: Option<Box<str>>,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl AsyncGeneratorExpr {
    /// Creates a new async generator expression
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

    /// Gets the name of the async generator expression
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(Box::as_ref)
    }

    /// Gets the list of parameters of the async generator expression
    pub fn parameters(&self) -> &[FormalParameter] {
        &self.parameters
    }

    /// Gets the body of the async generator expression
    pub fn body(&self) -> &StatementList {
        &self.body
    }

    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        f.write_str("async function*")?;
        if let Some(ref name) = self.name {
            write!(f, " {}", name)?;
        }
        f.write_str("(")?;
        join_nodes(f, &self.parameters)?;
        f.write_str(") ")?;
        self.display_block(f, indentation)
    }

    pub(in crate::syntax::ast::node) fn display_block(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        if self.body().items().is_empty() {
            f.write_str("{}")
        } else {
            f.write_str("{\n")?;
            self.body.display(f, indentation + 1)?;
            write!(f, "{}}}", "    ".repeat(indentation))
        }
    }
}

impl Executable for AsyncGeneratorExpr {
    fn run(&self, _context: &mut Context) -> JsResult<JsValue> {
        //TODO: Implement AsyncGeneratorFunction
        Ok(JsValue::undefined())
    }
}

impl fmt::Display for AsyncGeneratorExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<AsyncGeneratorExpr> for Node {
    fn from(expr: AsyncGeneratorExpr) -> Self {
        Self::AsyncGeneratorExpr(expr)
    }
}
