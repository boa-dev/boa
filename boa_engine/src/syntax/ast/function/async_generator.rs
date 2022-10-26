//! Async Generator Expression
use crate::syntax::ast::{
    block_to_string,
    expression::{Expression, Identifier},
    join_nodes, Declaration, StatementList,
};
use boa_interner::{Interner, ToIndentedString};

use super::FormalParameterList;

/// An async generator definition, as defined by the [spec].
///
/// An [async generator][mdn] combines async functions with generators, making it possible to use
/// `await` and `yield` expressions within the definition of the function.
///
/// [spec]: https://tc39.es/ecma262/#sec-async-generator-function-definitions
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function*
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncGenerator {
    name: Option<Identifier>,
    parameters: FormalParameterList,
    body: StatementList,
}

impl AsyncGenerator {
    /// Creates a new async generator expression
    #[inline]
    pub(in crate::syntax) fn new(
        name: Option<Identifier>,
        parameters: FormalParameterList,
        body: StatementList,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
        }
    }

    /// Gets the name of the async generator expression
    #[inline]
    pub fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the async generator expression
    #[inline]
    pub fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the async generator expression
    #[inline]
    pub fn body(&self) -> &StatementList {
        &self.body
    }
}

impl ToIndentedString for AsyncGenerator {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "async function*".to_owned();
        if let Some(name) = self.name {
            buf.push_str(&format!(" {}", interner.resolve_expect(name.sym())));
        }
        buf.push_str(&format!(
            "({}) {}",
            join_nodes(interner, &self.parameters.parameters),
            block_to_string(&self.body, interner, indentation)
        ));

        buf
    }
}

impl From<AsyncGenerator> for Expression {
    #[inline]
    fn from(expr: AsyncGenerator) -> Self {
        Self::AsyncGenerator(expr)
    }
}

impl From<AsyncGenerator> for Declaration {
    #[inline]
    fn from(f: AsyncGenerator) -> Self {
        Self::AsyncGenerator(f)
    }
}
