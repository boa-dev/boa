//! Async Generator Expression

use super::{FormalParameterList, FunctionBody};
use crate::operations::{ContainsSymbol, contains};
use crate::scope::{FunctionScopes, Scope};
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{
    Declaration, Spanned, block_to_string,
    expression::{Expression, Identifier},
    join_nodes,
};
use crate::{LinearSpan, LinearSpanIgnoreEq, Span};
use boa_interner::{Interner, ToIndentedString};
use core::{fmt::Write as _, ops::ControlFlow};

/// An async generator declaration.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorDeclaration
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function*
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncGeneratorDeclaration<'arena> {
    name: Identifier,
    pub(crate) parameters: FormalParameterList<'arena>,
    pub(crate) body: FunctionBody<'arena>,
    pub(crate) contains_direct_eval: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scopes: FunctionScopes,
    linear_span: LinearSpanIgnoreEq,
}

impl<'arena> AsyncGeneratorDeclaration<'arena> {
    /// Creates a new async generator declaration.
    #[inline]
    #[must_use]
    pub fn new(
        name: Identifier,
        parameters: FormalParameterList<'arena>,
        body: FunctionBody<'arena>,
        linear_span: LinearSpan,
    ) -> Self {
        let contains_direct_eval = contains(&parameters, ContainsSymbol::DirectEval)
            || contains(&body, ContainsSymbol::DirectEval);
        Self {
            name,
            parameters,
            body,
            contains_direct_eval,
            scopes: FunctionScopes::default(),
            linear_span: linear_span.into(),
        }
    }

    /// Gets the name of the async generator declaration.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Identifier {
        self.name
    }

    /// Gets the list of parameters of the async generator declaration.
    #[inline]
    #[must_use]
    pub const fn parameters(&self) -> &FormalParameterList<'arena> {
        &self.parameters
    }

    /// Gets the body of the async generator declaration.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &FunctionBody<'arena> {
        &self.body
    }

    /// Gets the scopes of the async generator declaration.
    #[inline]
    #[must_use]
    pub const fn scopes(&self) -> &FunctionScopes {
        &self.scopes
    }

    /// Gets linear span of the function declaration.
    #[inline]
    #[must_use]
    pub const fn linear_span(&self) -> LinearSpan {
        self.linear_span.0
    }

    /// Returns `true` if the async generator declaration contains a direct call to `eval`.
    #[inline]
    #[must_use]
    pub const fn contains_direct_eval(&self) -> bool {
        self.contains_direct_eval
    }
}

impl ToIndentedString for AsyncGeneratorDeclaration<'_> {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "async function* {}({}) {}",
            interner.resolve_expect(self.name.sym()),
            join_nodes(interner, self.parameters.as_ref()),
            block_to_string(&self.body.statements, interner, indentation)
        )
    }
}

impl<'arena> VisitWith<'arena> for AsyncGeneratorDeclaration<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_identifier(&self.name)?;
        visitor.visit_formal_parameter_list(&self.parameters)?;
        visitor.visit_function_body(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_identifier_mut(&mut self.name)?;
        visitor.visit_formal_parameter_list_mut(&mut self.parameters)?;
        visitor.visit_function_body_mut(&mut self.body)
    }
}

impl<'arena> From<AsyncGeneratorDeclaration<'arena>> for Declaration<'arena> {
    #[inline]
    fn from(f: AsyncGeneratorDeclaration<'arena>) -> Self {
        Self::AsyncGeneratorDeclaration(f)
    }
}

/// An async generator expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function*
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncGeneratorExpression<'arena> {
    pub(crate) name: Option<Identifier>,
    pub(crate) parameters: FormalParameterList<'arena>,
    pub(crate) body: FunctionBody<'arena>,
    pub(crate) has_binding_identifier: bool,
    pub(crate) contains_direct_eval: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) name_scope: Option<Scope>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scopes: FunctionScopes,
    linear_span: LinearSpanIgnoreEq,

    span: Span,
}

impl<'arena> AsyncGeneratorExpression<'arena> {
    /// Creates a new async generator expression.
    #[inline]
    #[must_use]
    pub fn new(
        name: Option<Identifier>,
        parameters: FormalParameterList<'arena>,
        body: FunctionBody<'arena>,
        linear_span: LinearSpan,
        has_binding_identifier: bool,
        span: Span,
    ) -> Self {
        let contains_direct_eval = contains(&parameters, ContainsSymbol::DirectEval)
            || contains(&body, ContainsSymbol::DirectEval);
        Self {
            name,
            parameters,
            body,
            has_binding_identifier,
            name_scope: None,
            contains_direct_eval,
            scopes: FunctionScopes::default(),
            linear_span: linear_span.into(),
            span,
        }
    }

    /// Gets the name of the async generator expression.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the async generator expression.
    #[inline]
    #[must_use]
    pub const fn parameters(&self) -> &FormalParameterList<'arena> {
        &self.parameters
    }

    /// Gets the body of the async generator expression.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &FunctionBody<'arena> {
        &self.body
    }

    /// Returns whether the async generator expression has a binding identifier.
    #[inline]
    #[must_use]
    pub const fn has_binding_identifier(&self) -> bool {
        self.has_binding_identifier
    }

    /// Gets the name scope of the async generator expression.
    #[inline]
    #[must_use]
    pub const fn name_scope(&self) -> Option<&Scope> {
        self.name_scope.as_ref()
    }

    /// Gets the scopes of the async generator expression.
    #[inline]
    #[must_use]
    pub const fn scopes(&self) -> &FunctionScopes {
        &self.scopes
    }

    /// Gets linear span of the function declaration.
    #[inline]
    #[must_use]
    pub const fn linear_span(&self) -> LinearSpan {
        self.linear_span.0
    }

    /// Returns `true` if the async generator expression contains a direct call to `eval`.
    #[inline]
    #[must_use]
    pub const fn contains_direct_eval(&self) -> bool {
        self.contains_direct_eval
    }
}

impl Spanned for AsyncGeneratorExpression<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl ToIndentedString for AsyncGeneratorExpression<'_> {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "async function*".to_owned();
        if self.has_binding_identifier
            && let Some(name) = self.name
        {
            let _ = write!(buf, " {}", interner.resolve_expect(name.sym()));
        }
        let _ = write!(
            buf,
            "({}) {}",
            join_nodes(interner, self.parameters.as_ref()),
            block_to_string(&self.body.statements, interner, indentation)
        );

        buf
    }
}

impl<'arena> From<AsyncGeneratorExpression<'arena>> for Expression<'arena> {
    #[inline]
    fn from(expr: AsyncGeneratorExpression<'arena>) -> Self {
        Self::AsyncGeneratorExpression(expr)
    }
}

impl<'arena> VisitWith<'arena> for AsyncGeneratorExpression<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        if let Some(ident) = &self.name {
            visitor.visit_identifier(ident)?;
        }
        visitor.visit_formal_parameter_list(&self.parameters)?;
        visitor.visit_function_body(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        if let Some(ident) = &mut self.name {
            visitor.visit_identifier_mut(ident)?;
        }
        visitor.visit_formal_parameter_list_mut(&mut self.parameters)?;
        visitor.visit_function_body_mut(&mut self.body)
    }
}
