//! Async Generator Expression
use crate::syntax::ast::{
    block_to_string,
    expression::{Expression, Identifier},
    join_nodes, Declaration, StatementList,
};
use boa_interner::{Interner, ToInternedString};

use super::FormalParameterList;

/// The `async function*` keyword can be used to define a generator function inside an expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorExpression
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncGenerator {
    name: Option<Identifier>,
    parameters: FormalParameterList,
    body: StatementList,
}

impl AsyncGenerator {
    /// Creates a new async generator expression
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
    pub fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the async generator expression
    pub fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the async generator expression
    pub fn body(&self) -> &StatementList {
        &self.body
    }

    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
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

impl ToInternedString for AsyncGenerator {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<AsyncGenerator> for Expression {
    fn from(expr: AsyncGenerator) -> Self {
        Self::AsyncGenerator(expr)
    }
}

impl From<AsyncGenerator> for Declaration {
    fn from(f: AsyncGenerator) -> Self {
        Self::AsyncGenerator(f)
    }
}
