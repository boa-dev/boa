use std::ops::ControlFlow;

use super::{FormalParameterList, FunctionBody};
use crate::operations::{contains, ContainsSymbol};
use crate::scope::FunctionScopes;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{
    expression::{Expression, Identifier},
    join_nodes,
};
use crate::{LinearSpan, LinearSpanIgnoreEq};
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
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncArrowFunction {
    pub(crate) name: Option<Identifier>,
    pub(crate) parameters: FormalParameterList,
    pub(crate) body: FunctionBody,
    pub(crate) contains_direct_eval: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scopes: FunctionScopes,
    linear_span: LinearSpanIgnoreEq,
}

impl AsyncArrowFunction {
    /// Creates a new `AsyncArrowFunction` AST Expression.
    #[must_use]
    pub fn new_boxed(
        name: Option<Identifier>,
        parameters: FormalParameterList,
        body: FunctionBody,
        linear_span: LinearSpan,
    ) -> Box<Self> {
        let contains_direct_eval = contains(&parameters, ContainsSymbol::DirectEval)
            || contains(&body, ContainsSymbol::DirectEval);
        Box::new(Self {
            name,
            parameters,
            body,
            contains_direct_eval,
            scopes: FunctionScopes::default(),
            linear_span: linear_span.into(),
        })
    }

    /// Gets the name of the async arrow function.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Sets the name of the async arrow function.
    #[inline]
    pub fn set_name(&mut self, name: Option<Identifier>) {
        self.name = name;
    }

    /// Gets the list of parameters of the async arrow function.
    #[inline]
    #[must_use]
    pub const fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the async arrow function.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &FunctionBody {
        &self.body
    }

    /// Returns the scopes of the async arrow function.
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

    /// Returns `true` if the function declaration contains a direct call to `eval`.
    #[inline]
    #[must_use]
    pub const fn contains_direct_eval(&self) -> bool {
        self.contains_direct_eval
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

impl From<Box<AsyncArrowFunction>> for Expression {
    fn from(decl: Box<AsyncArrowFunction>) -> Self {
        Self::AsyncArrowFunction(decl)
    }
}

impl VisitWith for AsyncArrowFunction {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(ident) = &self.name {
            visitor.visit_identifier(ident)?;
        }
        visitor.visit_formal_parameter_list(&self.parameters)?;
        visitor.visit_function_body(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(ident) = &mut self.name {
            visitor.visit_identifier_mut(ident)?;
        }
        visitor.visit_formal_parameter_list_mut(&mut self.parameters)?;
        visitor.visit_function_body_mut(&mut self.body)
    }
}
