//! This module contains Function and Class AST nodes.
//!
//! ECMAScript defines multiple types of functions and classes.
//! They are split into different AST nodes to reduce ambiguity and to make the AST more readable.
//!
//! - Functions:
//!   - [`FunctionDeclaration`]
//!   - [`FunctionExpression`]
//! - Async functions:
//!   - [`AsyncFunctionDeclaration`]
//!   - [`AsyncFunctionExpression`]
//! - Generators
//!   - [`GeneratorDeclaration`]
//!   - [`GeneratorExpression`]
//! - Async Generators
//!   - [`AsyncGeneratorDeclaration`]
//!   - [`AsyncGeneratorExpression`]
//! - Arrow Functions
//!   - [`ArrowFunction`]
//! - Async Arrow Functions
//!   - [`AsyncArrowFunction`]
//! - Classes
//!   - [`ClassDeclaration`]
//!   - [`ClassExpression`]

mod arrow_function;
mod async_arrow_function;
mod async_function;
mod async_generator;
mod class;
mod generator;
mod ordinary_function;
mod parameters;

use std::ops::ControlFlow;

pub use arrow_function::ArrowFunction;
pub use async_arrow_function::AsyncArrowFunction;
pub use async_function::{AsyncFunctionDeclaration, AsyncFunctionExpression};
pub use async_generator::{AsyncGeneratorDeclaration, AsyncGeneratorExpression};
use boa_interner::{Interner, ToIndentedString};
pub use class::{
    ClassDeclaration, ClassElement, ClassElementName, ClassExpression, ClassFieldDefinition,
    ClassMethodDefinition, PrivateFieldDefinition, PrivateName, StaticBlockBody,
};
pub use generator::{GeneratorDeclaration, GeneratorExpression};
pub use ordinary_function::{FunctionDeclaration, FunctionExpression};
pub use parameters::{FormalParameter, FormalParameterList, FormalParameterListFlags};

use crate::{
    LinearPosition, Span, Spanned, StatementList, StatementListItem,
    visitor::{VisitWith, Visitor, VisitorMut},
};

/// A Function body.
///
/// Since `Script` and `FunctionBody` have the same semantics, this is currently
/// only an alias of the former.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionBody
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionBody<'arena> {
    pub(crate) statements: StatementList<'arena>,
    span: Span,
}

impl<'arena> FunctionBody<'arena> {
    /// Creates a new `FunctionBody` AST node.
    #[inline]
    #[must_use]
    pub fn new(statements: StatementList<'arena>, span: Span) -> Self {
        Self { statements, span }
    }

    /// Gets the list of statements.
    #[inline]
    #[must_use]
    pub const fn statements(&self) -> &[StatementListItem<'arena>] {
        self.statements.statements()
    }

    /// Gets the statement list.
    #[inline]
    #[must_use]
    pub const fn statement_list(&self) -> &StatementList<'arena> {
        &self.statements
    }

    /// Get the strict mode.
    #[inline]
    #[must_use]
    pub const fn strict(&self) -> bool {
        self.statements.strict()
    }

    /// Get end of linear position in source code.
    #[inline]
    #[must_use]
    pub const fn linear_pos_end(&self) -> LinearPosition {
        self.statements.linear_pos_end()
    }
}

impl<'arena> Spanned for FunctionBody<'arena> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> ToIndentedString for FunctionBody<'arena> {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        self.statements.to_indented_string(interner, indentation)
    }
}

impl<'arena> VisitWith<'arena> for FunctionBody<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        for statement in &*self.statements {
            visitor.visit_statement_list_item(statement)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        for statement in &mut *self.statements.statements {
            visitor.visit_statement_list_item_mut(statement)?;
        }
        ControlFlow::Continue(())
    }
}
