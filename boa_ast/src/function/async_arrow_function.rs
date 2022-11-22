use std::ops::ControlFlow;

use super::FormalParameterList;
use crate::try_break;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{
    expression::{Expression, Identifier},
    join_nodes, StatementList,
};
use boa_interner::{Interner, ToIndentedString};

/// An async arrow function expression, as defined by the [spec].
///
/// An [async arrow function][mdn] expression is a syntactically compact alternative to a regular function
/// expression. Arrow function expressions are ill suited as methods, and they cannot be used as
/// constructors. Arrow functions cannot be used as constructors and will throw an error when
/// used with new.
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncArrowFunction
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncArrowFunction {
    name: Option<Identifier>,
    parameters: FormalParameterList,
    body: StatementList,
}

impl AsyncArrowFunction {
    /// Creates a new `AsyncArrowFunction` AST Expression.
    #[inline]
    #[must_use]
    pub const fn new(
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

    /// Gets the name of the function declaration.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Sets the name of the function declaration.
    //#[inline]
    //#[must_use]
    //pub fn set_name(&mut self, name: Option<Identifier>) {
    //    self.name = name;
    //}

    /// Gets the list of parameters of the arrow function.
    #[inline]
    #[must_use]
    pub const fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the arrow function.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &StatementList {
        &self.body
    }
}

impl ToIndentedString for AsyncArrowFunction {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = format!("async ({}", join_nodes(interner, self.parameters.as_ref()));
        if self.body().statements().is_empty() {
            buf.push_str(") => {}");
        } else {
            buf.push_str(&format!(
                ") => {{\n{}{}}}",
                self.body.to_indented_string(interner, indentation + 1),
                "    ".repeat(indentation)
            ));
        }
        buf
    }
}

impl From<AsyncArrowFunction> for Expression {
    fn from(decl: AsyncArrowFunction) -> Self {
        Self::AsyncArrowFunction(decl)
    }
}

impl VisitWith for AsyncArrowFunction {
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
