use crate::{
    block_to_string,
    expression::{Expression, Identifier},
    join_nodes, Declaration, StatementList,
};
use core::ops::ControlFlow;

use crate::try_break;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use boa_interner::{Interner, ToIndentedString};

use super::FormalParameterList;

/// A generator definition, as defined by the [spec].
///
/// [Generators][mdn] are "resumable functions", which can be suspended during execution and
/// resumed at any later time. The main feature of a generator are `yield` expressions, which
/// specifies the value returned when a generator is suspended, and the point from which
/// the execution will resume.
///
/// [spec]: https://tc39.es/ecma262/#sec-generator-function-definitions
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function*
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Generator {
    name: Option<Identifier>,
    parameters: FormalParameterList,
    body: StatementList,
}

impl Generator {
    /// Creates a new generator expression
    #[inline]
    #[must_use]
    pub fn new(
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
    #[inline]
    #[must_use]
    pub fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the generator declaration.
    #[inline]
    #[must_use]
    pub fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the generator declaration.
    #[inline]
    #[must_use]
    pub fn body(&self) -> &StatementList {
        &self.body
    }
}

impl ToIndentedString for Generator {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "function*".to_owned();
        if let Some(name) = self.name {
            buf.push_str(&format!(" {}", interner.resolve_expect(name.sym())));
        }
        buf.push_str(&format!(
            "({}) {}",
            join_nodes(interner, self.parameters.as_ref()),
            block_to_string(&self.body, interner, indentation)
        ));

        buf
    }
}

impl From<Generator> for Expression {
    #[inline]
    fn from(expr: Generator) -> Self {
        Self::Generator(expr)
    }
}

impl From<Generator> for Declaration {
    #[inline]
    fn from(f: Generator) -> Self {
        Self::Generator(f)
    }
}

impl VisitWith for Generator {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(ident) = &self.name {
            try_break!(visitor.visit_identifier(ident));
        }
        try_break!(visitor.visit_formal_parameter_list(&self.parameters));
        visitor.visit_statement_list(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(ident) = &mut self.name {
            try_break!(visitor.visit_identifier_mut(ident));
        }
        try_break!(visitor.visit_formal_parameter_list_mut(&mut self.parameters));
        visitor.visit_statement_list_mut(&mut self.body)
    }
}
