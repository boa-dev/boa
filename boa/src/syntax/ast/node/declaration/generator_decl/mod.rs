use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::{join_nodes, FormalParameter, Node, StatementList},
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The `function*` declaration (`function` keyword followed by an asterisk) defines a generator function,
/// which returns a `Generator` object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-GeneratorDeclaration
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function*
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct GeneratorDecl {
    name: Box<str>,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl GeneratorDecl {
    /// Creates a new generator declaration.
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

    /// Gets the name of the generator declaration.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the list of parameters of the generator declaration.
    pub fn parameters(&self) -> &[FormalParameter] {
        &self.parameters
    }

    /// Gets the body of the generator declaration.
    pub fn body(&self) -> &[Node] {
        self.body.items()
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        write!(f, "function* {}(", self.name)?;
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

impl From<GeneratorDecl> for Node {
    fn from(decl: GeneratorDecl) -> Self {
        Self::GeneratorDecl(decl)
    }
}

impl fmt::Display for GeneratorDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}
