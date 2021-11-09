use crate::syntax::ast::node::{join_nodes, FormalParameter, Node, StatementList};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The `function*` keyword can be used to define a generator function inside an expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-GeneratorExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function*
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct GeneratorExpr {
    name: Option<Box<str>>,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl GeneratorExpr {
    /// Creates a new generator expression
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

    /// Gets the name of the generator declaration.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(Box::as_ref)
    }

    /// Gets the list of parameters of the generator declaration.
    pub fn parameters(&self) -> &[FormalParameter] {
        &self.parameters
    }

    /// Gets the body of the generator declaration.
    pub fn body(&self) -> &StatementList {
        &self.body
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        f.write_str("function*")?;
        if let Some(ref name) = self.name {
            write!(f, " {}", name)?;
        }
        f.write_str("(")?;
        join_nodes(f, &self.parameters)?;
        f.write_str(") ")?;
        self.display_block(f, indentation)
    }

    /// Displays the generator's body. This includes the curly braces at the start and end.
    /// This will not indent the first brace, but will indent the last brace.
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

impl fmt::Display for GeneratorExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<GeneratorExpr> for Node {
    fn from(expr: GeneratorExpr) -> Self {
        Self::GeneratorExpr(expr)
    }
}
