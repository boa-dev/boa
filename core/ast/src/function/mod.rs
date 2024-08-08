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

pub use arrow_function::ArrowFunction;
pub use async_arrow_function::AsyncArrowFunction;
pub use async_function::{AsyncFunctionDeclaration, AsyncFunctionExpression};
pub use async_generator::{AsyncGeneratorDeclaration, AsyncGeneratorExpression};
pub use class::{
    ClassDeclaration, ClassElement, ClassElementName, ClassExpression, ClassMethodDefinition,
    PrivateName,
};
pub use generator::{GeneratorDeclaration, GeneratorExpression};
pub use ordinary_function::{FunctionDeclaration, FunctionExpression};
pub use parameters::{FormalParameter, FormalParameterList, FormalParameterListFlags};

use crate::Script;

/// A Function body.
///
/// Since [`Script`] and `FunctionBody` has the same semantics, this is currently
/// only an alias of the former.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionBody
pub type FunctionBody = Script;
