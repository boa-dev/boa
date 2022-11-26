//! Export declaration AST nodes.
//!
//! This module contains `export` declaration AST nodes.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-exports
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/export

use super::FromClause;
use crate::{
    function::{AsyncFunction, AsyncGenerator, Class, Function, Generator},
    Declaration, Expression, Statement,
};
use boa_interner::Sym;

/// An export declaration AST node.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ExportDeclaration
#[derive(Debug, Clone)]
pub enum ExportDeclaration {
    /// Re-export all exports.
    ReExportAll {
        /// Alias for the module export.
        alias: Option<Sym>,
        /// From clause.
        from: FromClause,
    },
    /// List of exports.
    List {
        /// List of exports.
        list: Box<[ExportSpecifier]>,
        /// From clause.
        from: Option<FromClause>,
    },
    /// Variable statement export.
    VarStatement(Statement),
    /// Declaration export.
    Declaration(Declaration),
    /// Default function export.
    DefaultFunction(Function),
    /// Default generator export.
    DefaultGenerator(Generator),
    /// Default async function export.
    DefaultAsyncFunction(AsyncFunction),
    /// Default async generator export.
    DefaultAsyncGenerator(AsyncGenerator),
    /// Default class declaration export.
    DefaultClassDeclaration(Class),
    /// Default assignment expression export.
    DefaultAssignmentExpression(Expression),
}

/// Export specifier
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ExportSpecifier
#[derive(Debug, Clone, Copy)]
pub struct ExportSpecifier {
    export_name: Sym,
    alias: Option<Sym>,
}

impl ExportSpecifier {
    /// Creates a new [`ExportSpecifier`].
    #[inline]
    #[must_use]
    pub const fn new(export_name: Sym, alias: Option<Sym>) -> Self {
        Self { export_name, alias }
    }

    /// Gets the original export name.
    #[inline]
    #[must_use]
    pub const fn export_name(self) -> Sym {
        self.export_name
    }

    /// Gets an optional export alias for the export.
    #[inline]
    #[must_use]
    pub const fn alias(self) -> Option<Sym> {
        self.alias
    }
}
