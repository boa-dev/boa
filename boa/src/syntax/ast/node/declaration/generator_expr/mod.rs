use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::{join_nodes, FormalParameter, Node, StatementList},
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

use super::block_to_string;

/// The `function*` keyword can be used to define a generator function inside an expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-GeneratorExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function*
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct GeneratorExpr {
    name: Option<Sym>,
    parameters: Box<[FormalParameter]>,
    body: StatementList,
}

impl GeneratorExpr {
    /// Creates a new generator expression
    pub(in crate::syntax) fn new<N, P, B>(name: N, parameters: P, body: B) -> Self
    where
        N: Into<Option<Sym>>,
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
    pub fn name(&self) -> Option<Sym> {
        self.name
    }

    /// Gets the list of parameters of the generator declaration.
    pub fn parameters(&self) -> &[FormalParameter] {
        &self.parameters
    }

    /// Gets the body of the generator declaration.
    pub fn body(&self) -> &StatementList {
        &self.body
    }

    /// Converts the generator expresion node to a string with indentation.
    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indentation: usize,
    ) -> String {
        let mut buf = "function*".to_owned();
        if let Some(name) = self.name {
            buf.push_str(&format!(
                " {}",
                interner.resolve(name).expect("string disappeared")
            ));
        }
        buf.push_str(&format!(
            "({}) {}",
            join_nodes(interner, &self.parameters),
            block_to_string(&self.body, interner, indentation)
        ));

        buf
    }
}

impl ToInternedString for GeneratorExpr {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<GeneratorExpr> for Node {
    fn from(expr: GeneratorExpr) -> Self {
        Self::GeneratorExpr(expr)
    }
}
