//! Async Function Expression.

use crate::try_break;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{
    expression::{Expression, Identifier},
    join_nodes, Declaration, StatementList,
};
use boa_interner::{Interner, ToIndentedString};
use core::ops::ControlFlow;

use super::FormalParameterList;

/// An async function definition, as defined by the [spec].
///
/// An [async function][mdn] is a function where await expressions are allowed within it.
/// The async and await keywords enable asynchronous programming on Javascript without the use
/// of promise chains.
///
/// [spec]: https://tc39.es/ecma262/#sec-async-function-definitions
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncFunction {
    name: Option<Identifier>,
    parameters: FormalParameterList,
    body: StatementList,
    has_binding_identifier: bool,
}

impl AsyncFunction {
    /// Creates a new function expression
    #[inline]
    #[must_use]
    pub const fn new(
        name: Option<Identifier>,
        parameters: FormalParameterList,
        body: StatementList,
        has_binding_identifier: bool,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
            has_binding_identifier,
        }
    }

    /// Gets the name of the function declaration.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the function declaration.
    #[inline]
    #[must_use]
    pub const fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the function declaration.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &StatementList {
        &self.body
    }

    /// Returns whether the function expression has a binding identifier.
    #[inline]
    #[must_use]
    pub const fn has_binding_identifier(&self) -> bool {
        self.has_binding_identifier
    }
}

impl ToIndentedString for AsyncFunction {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "async function".to_owned();
        if let Some(name) = self.name {
            buf.push_str(&format!(" {}", interner.resolve_expect(name.sym())));
        }
        buf.push_str(&format!(
            "({}",
            join_nodes(interner, self.parameters.as_ref())
        ));
        if self.body().statements().is_empty() {
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

impl From<AsyncFunction> for Expression {
    #[inline]
    fn from(expr: AsyncFunction) -> Self {
        Self::AsyncFunction(expr)
    }
}

impl From<AsyncFunction> for Declaration {
    #[inline]
    fn from(f: AsyncFunction) -> Self {
        Self::AsyncFunction(f)
    }
}

impl VisitWith for AsyncFunction {
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
