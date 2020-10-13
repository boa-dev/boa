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

/// Async arrow function.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]:
/// [mdn]: https://tc39.es/ecma262/#prod-AsyncArrowFunction
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct AsyncArrowFunctionDecl {
    params: Box<[FormalParameter]>,
    body: StatementList,
}

impl AsyncArrowFunctionDecl {
    /// Creates a new `AsyncArrowFunctionDecl` AST node.
    pub(in crate::syntax) fn new<P, B>(params: P, body: B) -> Self
    where
        P: Into<Box<[FormalParameter]>>,
        B: Into<StatementList>,
    {
        Self {
            params: params.into(),
            body: body.into(),
        }
    }

    /// Gets the list of parameters of the async arrow function.
    pub(crate) fn params(&self) -> &[FormalParameter] {
        &self.params
    }

    /// Gets the body of the async arrow function.
    pub(crate) fn body(&self) -> &[Node] {
        &self.body.statements()
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        write!(f, "async (")?;
        join_nodes(f, &self.params)?;
        f.write_str(") => ")?;
        self.body.display(f, indentation)
    }
}

impl Executable for AsyncArrowFunctionDecl {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        Ok(interpreter.create_function(
            self.params().to_vec(),
            self.body().to_vec(),
            FunctionFlags::CALLABLE
                | FunctionFlags::CONSTRUCTABLE
                | FunctionFlags::LEXICAL_THIS_MODE,
        ))
    }
}

impl fmt::Display for AsyncArrowFunctionDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<AsyncArrowFunctionDecl> for Node {
    fn from(decl: AsyncArrowFunctionDecl) -> Self {
        Self::AsyncArrowFunctionDecl(decl)
    }
}
