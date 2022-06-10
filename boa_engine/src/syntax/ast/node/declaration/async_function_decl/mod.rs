//! Async Function Declaration.

use crate::syntax::ast::node::{join_nodes, FormalParameterList, Node, StatementList};
use boa_interner::{Interner, Sym, ToInternedString};

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
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncFunctionDecl {
    name: Sym,
    parameters: FormalParameterList,
    body: StatementList,
}

impl AsyncFunctionDecl {
    /// Creates a new async function declaration.
    pub(in crate::syntax) fn new<P, B>(name: Sym, parameters: P, body: B) -> Self
    where
        P: Into<FormalParameterList>,
        B: Into<StatementList>,
    {
        Self {
            name,
            parameters: parameters.into(),
            body: body.into(),
        }
    }

    /// Gets the name of the async function declaration.
    pub fn name(&self) -> Sym {
        self.name
    }

    /// Gets the list of parameters of the async function declaration.
    pub fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the async function declaration.
    pub fn body(&self) -> &[Node] {
        self.body.items()
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indentation: usize,
    ) -> String {
        let mut buf = format!(
            "async function {}({}",
            interner.resolve_expect(self.name),
            join_nodes(interner, &self.parameters.parameters)
        );
        if self.body().is_empty() {
            buf.push_str(") {}");
        } else {
            buf.push_str(&format!(
                ") {{\n{}{}}}",
                self.body.to_indented_string(interner, indentation + 1),
                "    ".repeat(indentation)
            ));
        }
        buf
    }
}

impl From<AsyncFunctionDecl> for Node {
    fn from(decl: AsyncFunctionDecl) -> Self {
        Self::AsyncFunctionDecl(decl)
    }
}

impl ToInternedString for AsyncFunctionDecl {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}
