//! Async Function Expression.

use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::syntax::ast::{
    expression::{Expression, Identifier},
    join_nodes, Declaration, StatementList,
};
use crate::try_break;
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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncFunction {
    name: Option<Identifier>,
    parameters: FormalParameterList,
    body: StatementList,
}

impl AsyncFunction {
    /// Creates a new function expression
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

    /// Gets the name of the function declaration.
    #[inline]
    pub fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the function declaration.
    #[inline]
    pub fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the function declaration.
    #[inline]
    pub fn body(&self) -> &StatementList {
        &self.body
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
            join_nodes(interner, &self.parameters.parameters)
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

#[cfg(test)]
mod tests {
    #[test]
    fn fmt() {
        crate::syntax::ast::test_formatting(
            r#"
            async function async_func(a, b) {
                console.log(a);
            }
            async function async_func_2(a, b) {}
            pass_async_func(async function(a, b) {
                console.log("in async callback", a);
            });
            pass_async_func(async function(a, b) {});
            "#,
        );
    }
}
