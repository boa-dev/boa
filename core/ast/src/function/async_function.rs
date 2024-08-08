//! Async Function Expression.

use super::{FormalParameterList, FunctionBody};
use crate::{
    block_to_string,
    expression::{Expression, Identifier},
    join_nodes, try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
    Declaration,
};
use boa_interner::{Interner, ToIndentedString};
use core::ops::ControlFlow;

/// An async function declaration.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncFunctionDeclaration
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncFunctionDeclaration {
    name: Identifier,
    parameters: FormalParameterList,
    body: FunctionBody,
}

impl AsyncFunctionDeclaration {
    /// Creates a new async function declaration.
    #[inline]
    #[must_use]
    pub const fn new(
        name: Identifier,
        parameters: FormalParameterList,
        body: FunctionBody,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
        }
    }

    /// Gets the name of the async function declaration.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Identifier {
        self.name
    }

    /// Gets the list of parameters of the async function declaration.
    #[inline]
    #[must_use]
    pub const fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the async function declaration.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &FunctionBody {
        &self.body
    }
}

impl ToIndentedString for AsyncFunctionDeclaration {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "async function {}({}) {}",
            interner.resolve_expect(self.name.sym()),
            join_nodes(interner, self.parameters.as_ref()),
            block_to_string(self.body.statements(), interner, indentation)
        )
    }
}

impl VisitWith for AsyncFunctionDeclaration {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_identifier(&self.name));
        try_break!(visitor.visit_formal_parameter_list(&self.parameters));
        visitor.visit_script(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_identifier_mut(&mut self.name));
        try_break!(visitor.visit_formal_parameter_list_mut(&mut self.parameters));
        visitor.visit_script_mut(&mut self.body)
    }
}

impl From<AsyncFunctionDeclaration> for Declaration {
    #[inline]
    fn from(f: AsyncFunctionDeclaration) -> Self {
        Self::AsyncFunctionDeclaration(f)
    }
}

/// An async function expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncFunctionExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncFunctionExpression {
    pub(crate) name: Option<Identifier>,
    parameters: FormalParameterList,
    body: FunctionBody,
    has_binding_identifier: bool,
}

impl AsyncFunctionExpression {
    /// Creates a new async function expression.
    #[inline]
    #[must_use]
    pub const fn new(
        name: Option<Identifier>,
        parameters: FormalParameterList,
        body: FunctionBody,
        has_binding_identifier: bool,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
            has_binding_identifier,
        }
    }

    /// Gets the name of the async function expression.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the async function expression.
    #[inline]
    #[must_use]
    pub const fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the async function expression.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &FunctionBody {
        &self.body
    }

    /// Returns whether the async function expression has a binding identifier.
    #[inline]
    #[must_use]
    pub const fn has_binding_identifier(&self) -> bool {
        self.has_binding_identifier
    }
}

impl ToIndentedString for AsyncFunctionExpression {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "async function".to_owned();
        if self.has_binding_identifier {
            if let Some(name) = self.name {
                buf.push_str(&format!(" {}", interner.resolve_expect(name.sym())));
            }
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

impl From<AsyncFunctionExpression> for Expression {
    #[inline]
    fn from(expr: AsyncFunctionExpression) -> Self {
        Self::AsyncFunctionExpression(expr)
    }
}

impl VisitWith for AsyncFunctionExpression {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(ident) = &self.name {
            try_break!(visitor.visit_identifier(ident));
        }
        try_break!(visitor.visit_formal_parameter_list(&self.parameters));
        visitor.visit_script(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(ident) = &mut self.name {
            try_break!(visitor.visit_identifier_mut(ident));
        }
        try_break!(visitor.visit_formal_parameter_list_mut(&mut self.parameters));
        visitor.visit_script_mut(&mut self.body)
    }
}
