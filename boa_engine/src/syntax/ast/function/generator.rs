use crate::syntax::ast::{
    block_to_string,
    expression::{Expression, Identifier},
    join_nodes, Declaration, StatementList,
};

use boa_interner::{Interner, ToInternedString};

use super::FormalParameterList;

/// The `function*` keyword can be used to define a generator function inside an expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-GeneratorExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function*
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Generator {
    name: Option<Identifier>,
    parameters: FormalParameterList,
    body: StatementList,
}

impl Generator {
    /// Creates a new generator expression
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

    /// Gets the name of the generator declaration.
    pub fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the generator declaration.
    pub fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the generator declaration.
    pub fn body(&self) -> &StatementList {
        &self.body
    }

    /// Converts the generator expresion Expression to a string with indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "function*".to_owned();
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

impl ToInternedString for Generator {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<Generator> for Expression {
    fn from(expr: Generator) -> Self {
        Self::Generator(expr)
    }
}

impl From<Generator> for Declaration {
    fn from(f: Generator) -> Self {
        Self::Generator(f)
    }
}
