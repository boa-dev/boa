//! Functions and classes nodes, as defined by the [spec].
//!
//! [Functions][func] are mainly subprograms that can be called by external code to execute a sequence of
//! statements (the *body* of the function). Javascript functions fall in several categories:
//!
//! - [`Function`]s.
//! - [`ArrowFunction`]s.
//! - [`AsyncArrowFunction`]s.
//! - [`Generator`]s.
//! - [`AsyncFunction`]s.
//! - [`AsyncGenerator`]s.
//!
//! All of them can be declared in either [declaration][decl] form or [expression][expr] form,
//! except from `ArrowFunction`s and `AsyncArrowFunction`s, which can only be declared in expression form.
//!
//! This module also contains [`Class`]es, which are templates for creating objects. Classes
//! can also be declared in either declaration or expression form.
//!
//! [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-functions-and-classes
//! [func]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions
//! [decl]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
//! [expr]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function

mod arrow_function;
mod async_arrow_function;
mod async_function;
mod async_generator;
mod class;
mod generator;
mod parameters;

pub use arrow_function::ArrowFunction;
pub use async_arrow_function::AsyncArrowFunction;
pub use async_function::AsyncFunction;
pub use async_generator::AsyncGenerator;
pub use class::{Class, ClassElement};
use core::ops::ControlFlow;
pub use generator::Generator;
pub use parameters::{FormalParameter, FormalParameterList, FormalParameterListFlags};

use crate::try_break;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{block_to_string, join_nodes, StatementList};
use boa_interner::{Interner, ToIndentedString};

use super::expression::{Expression, Identifier};
use super::Declaration;

/// A function definition, as defined by the [spec].
///
/// By default, functions return `undefined`. To return any other value, the function must have
/// a return statement that specifies the value to return.
///
/// More information:
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-function-definitions
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    name: Option<Identifier>,
    parameters: FormalParameterList,
    body: StatementList,
    has_binding_identifier: bool,
}

impl Function {
    /// Creates a new function expression.
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
            has_binding_identifier: false,
        }
    }

    /// Creates a new function expression with an expression binding identifier.
    #[inline]
    #[must_use]
    pub fn new_with_binding_identifier(
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
    pub fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Gets the list of parameters of the function declaration.
    #[inline]
    #[must_use]
    pub fn parameters(&self) -> &FormalParameterList {
        &self.parameters
    }

    /// Gets the body of the function declaration.
    #[inline]
    #[must_use]
    pub fn body(&self) -> &StatementList {
        &self.body
    }

    /// Returns whether the function expression has a binding identifier.
    #[inline]
    #[must_use]
    pub fn has_binding_identifier(&self) -> bool {
        self.has_binding_identifier
    }
}

impl ToIndentedString for Function {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        let mut buf = "function".to_owned();
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

impl From<Function> for Expression {
    #[inline]
    fn from(expr: Function) -> Self {
        Self::Function(expr)
    }
}

impl From<Function> for Declaration {
    #[inline]
    fn from(f: Function) -> Self {
        Self::Function(f)
    }
}

impl VisitWith for Function {
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
