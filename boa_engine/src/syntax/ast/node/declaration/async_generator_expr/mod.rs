//! Async Generator Expression

use crate::syntax::ast::node::{join_nodes, FormalParameterList, Node, StatementList};
use boa_gc::{Finalize, Trace};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

use super::block_to_string;

/// The `async function*` keyword can be used to define a generator function inside an expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorExpression
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct AsyncGeneratorExpr {
    name: Option<Sym>,
    parameters: FormalParameterList,
    body: StatementList,
}

impl AsyncGeneratorExpr {
    /// Creates a new async generator expression
    pub(in crate::syntax) fn new<N, P, B>(name: N, parameters: P, body: B) -> Self
    where
        N: Into<Option<Sym>>,
        P: Into<FormalParameterList>,
        B: Into<StatementList>,
    {
        Self {
            name: name.into(),
            parameters: parameters.into(),
            body: body.into(),
        }
    }

    /// Gets the name of the async generator expression
    pub fn name(&self) -> Option<Sym> {
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

    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indentation: usize,
    ) -> String {
        let mut buf = "async function*".to_owned();
        if let Some(name) = self.name {
            buf.push_str(&format!(" {}", interner.resolve_expect(name)));
        }
        buf.push_str(&format!(
            "({}) {}",
            join_nodes(interner, &self.parameters.parameters),
            block_to_string(&self.body, interner, indentation)
        ));

        buf
    }
}

impl ToInternedString for AsyncGeneratorExpr {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<AsyncGeneratorExpr> for Node {
    fn from(expr: AsyncGeneratorExpr) -> Self {
        Self::AsyncGeneratorExpr(expr)
    }
}
