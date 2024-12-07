use super::{FormalParameterList, FunctionBody};
use crate::{
    block_to_string,
    expression::{Expression, Identifier},
    join_nodes,
    operations::{contains, ContainsSymbol},
    scope::{FunctionScopes, Scope},
    scope_analyzer::{analyze_binding_escapes, collect_bindings},
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
    Declaration,
};
use boa_interner::{Interner, ToIndentedString};
use core::ops::ControlFlow;

/// A function declaration.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionDeclaration
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDeclaration {
    name: Identifier,
    pub(crate) parameters: FormalParameterList,
    pub(crate) body: FunctionBody,
    pub(crate) contains_direct_eval: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scopes: FunctionScopes,
}

impl FunctionDeclaration {
    /// Creates a new function declaration.
    #[inline]
    #[must_use]
    pub fn new(name: Identifier, parameters: FormalParameterList, body: FunctionBody) -> Self {
        let contains_direct_eval = contains(&parameters, ContainsSymbol::DirectEval)
            || contains(&body, ContainsSymbol::DirectEval);
        Self {
            name,
            parameters,
            body,
            contains_direct_eval,
            scopes: FunctionScopes::default(),
        }
    }

    /// Gets the name of the function declaration.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Identifier {
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
    pub const fn body(&self) -> &FunctionBody {
        &self.body
    }

    /// Gets the scopes of the function declaration.
    #[inline]
    #[must_use]
    pub const fn scopes(&self) -> &FunctionScopes {
        &self.scopes
    }

    /// Returns `true` if the function declaration contains a direct call to `eval`.
    #[inline]
    #[must_use]
    pub const fn contains_direct_eval(&self) -> bool {
        self.contains_direct_eval
    }
}

impl ToIndentedString for FunctionDeclaration {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "function {}({}) {}",
            interner.resolve_expect(self.name.sym()),
            join_nodes(interner, self.parameters.as_ref()),
            block_to_string(&self.body.statements, interner, indentation)
        )
    }
}

impl VisitWith for FunctionDeclaration {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_identifier(&self.name));
        try_break!(visitor.visit_formal_parameter_list(&self.parameters));
        visitor.visit_function_body(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_identifier_mut(&mut self.name));
        try_break!(visitor.visit_formal_parameter_list_mut(&mut self.parameters));
        visitor.visit_function_body_mut(&mut self.body)
    }
}

impl From<FunctionDeclaration> for Declaration {
    #[inline]
    fn from(f: FunctionDeclaration) -> Self {
        Self::FunctionDeclaration(f)
    }
}

/// A function expression.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionExpression {
    pub(crate) name: Option<Identifier>,
    pub(crate) parameters: FormalParameterList,
    pub(crate) body: FunctionBody,
    pub(crate) has_binding_identifier: bool,
    pub(crate) contains_direct_eval: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) name_scope: Option<Scope>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scopes: FunctionScopes,
}

impl FunctionExpression {
    /// Creates a new function expression.
    #[inline]
    #[must_use]
    pub fn new(
        name: Option<Identifier>,
        parameters: FormalParameterList,
        body: FunctionBody,
        has_binding_identifier: bool,
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
        }
    }

    /// Gets the name of the function expression.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the function expression.
    #[inline]
    #[must_use]
    pub const fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the function expression.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &FunctionBody {
        &self.body
    }

    /// Returns whether the function expression has a binding identifier.
    #[inline]
    #[must_use]
    pub const fn has_binding_identifier(&self) -> bool {
        self.has_binding_identifier
    }

    /// Gets the name scope of the function expression.
    #[inline]
    #[must_use]
    pub const fn name_scope(&self) -> Option<&Scope> {
        self.name_scope.as_ref()
    }

    /// Gets the scopes of the function expression.
    #[inline]
    #[must_use]
    pub const fn scopes(&self) -> &FunctionScopes {
        &self.scopes
    }

    /// Returns `true` if the function expression contains a direct call to `eval`.
    #[inline]
    #[must_use]
    pub const fn contains_direct_eval(&self) -> bool {
        self.contains_direct_eval
    }

    /// Analyze the scope of the function expression.
    pub fn analyze_scope(&mut self, strict: bool, scope: &Scope, interner: &Interner) -> bool {
        if !collect_bindings(self, strict, false, scope, interner) {
            return false;
        }
        analyze_binding_escapes(self, false, scope.clone(), interner)
    }
}

impl ToIndentedString for FunctionExpression {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "function".to_owned();
        if self.has_binding_identifier {
            if let Some(name) = self.name {
                buf.push_str(&format!(" {}", interner.resolve_expect(name.sym())));
            }
        }
        buf.push_str(&format!(
            "({}) {}",
            join_nodes(interner, self.parameters.as_ref()),
            block_to_string(&self.body.statements, interner, indentation)
        ));

        buf
    }
}

impl From<FunctionExpression> for Expression {
    #[inline]
    fn from(expr: FunctionExpression) -> Self {
        Self::FunctionExpression(expr)
    }
}

impl VisitWith for FunctionExpression {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(ident) = &self.name {
            try_break!(visitor.visit_identifier(ident));
        }
        try_break!(visitor.visit_formal_parameter_list(&self.parameters));
        visitor.visit_function_body(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(ident) = &mut self.name {
            try_break!(visitor.visit_identifier_mut(ident));
        }
        try_break!(visitor.visit_formal_parameter_list_mut(&mut self.parameters));
        visitor.visit_function_body_mut(&mut self.body)
    }
}
