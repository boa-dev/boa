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

use std::ops::ControlFlow;

use super::{ModuleSpecifier, VarDeclaration};
use crate::{
    function::{AsyncFunction, AsyncGenerator, Class, Function, Generator},
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
    Declaration, Expression,
};
use boa_interner::Sym;

/// The kind of re-export in an [`ExportDeclaration`].
#[derive(Debug, Clone)]
pub enum ReExportKind {
    /// Namespaced Re-export (`export * as name from "module-name"`).
    Namespaced {
        /// Reexported name for the imported module.
        name: Option<Sym>,
    },
    /// Re-export list (`export { export1, export2 as alias2 } from "module-name"`).
    Named {
        /// List of the required re-exports of the re-exported module.
        names: Box<[ExportSpecifier]>,
    },
}

impl VisitWith for ReExportKind {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Self::Namespaced { name: Some(name) } => visitor.visit_sym(name),
            Self::Namespaced { name: None } => ControlFlow::Continue(()),
            Self::Named { names } => {
                for name in &**names {
                    try_break!(visitor.visit_export_specifier(name));
                }
                ControlFlow::Continue(())
            }
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Self::Namespaced { name: Some(name) } => visitor.visit_sym_mut(name),
            Self::Namespaced { name: None } => ControlFlow::Continue(()),
            Self::Named { names } => {
                for name in &mut **names {
                    try_break!(visitor.visit_export_specifier_mut(name));
                }
                ControlFlow::Continue(())
            }
        }
    }
}

/// An export declaration AST node.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ExportDeclaration
#[derive(Debug, Clone)]
pub enum ExportDeclaration {
    /// Re-export.
    ReExport {
        /// The kind of reexport declared.
        kind: ReExportKind,
        /// Reexported module specifier.
        specifier: ModuleSpecifier,
    },
    /// List of exports.
    List(Box<[ExportSpecifier]>),
    /// Variable statement export.
    VarStatement(VarDeclaration),
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

impl VisitWith for ExportDeclaration {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Self::ReExport { specifier, kind } => {
                try_break!(visitor.visit_module_specifier(specifier));
                visitor.visit_re_export_kind(kind)
            }
            Self::List(list) => {
                for item in &**list {
                    try_break!(visitor.visit_export_specifier(item));
                }
                ControlFlow::Continue(())
            }
            Self::VarStatement(var) => visitor.visit_var_declaration(var),
            Self::Declaration(decl) => visitor.visit_declaration(decl),
            Self::DefaultFunction(f) => visitor.visit_function(f),
            Self::DefaultGenerator(g) => visitor.visit_generator(g),
            Self::DefaultAsyncFunction(af) => visitor.visit_async_function(af),
            Self::DefaultAsyncGenerator(ag) => visitor.visit_async_generator(ag),
            Self::DefaultClassDeclaration(c) => visitor.visit_class(c),
            Self::DefaultAssignmentExpression(expr) => visitor.visit_expression(expr),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Self::ReExport { specifier, kind } => {
                try_break!(visitor.visit_module_specifier_mut(specifier));
                visitor.visit_re_export_kind_mut(kind)
            }
            Self::List(list) => {
                for item in &mut **list {
                    try_break!(visitor.visit_export_specifier_mut(item));
                }
                ControlFlow::Continue(())
            }
            Self::VarStatement(var) => visitor.visit_var_declaration_mut(var),
            Self::Declaration(decl) => visitor.visit_declaration_mut(decl),
            Self::DefaultFunction(f) => visitor.visit_function_mut(f),
            Self::DefaultGenerator(g) => visitor.visit_generator_mut(g),
            Self::DefaultAsyncFunction(af) => visitor.visit_async_function_mut(af),
            Self::DefaultAsyncGenerator(ag) => visitor.visit_async_generator_mut(ag),
            Self::DefaultClassDeclaration(c) => visitor.visit_class_mut(c),
            Self::DefaultAssignmentExpression(expr) => visitor.visit_expression_mut(expr),
        }
    }
}

/// Export specifier
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ExportSpecifier
#[derive(Debug, Clone, Copy)]
pub struct ExportSpecifier {
    alias: Sym,
    private_name: Sym,
}

impl ExportSpecifier {
    /// Creates a new [`ExportSpecifier`].
    #[inline]
    #[must_use]
    pub const fn new(alias: Sym, private_name: Sym) -> Self {
        Self {
            alias,
            private_name,
        }
    }

    /// Gets the original alias.
    #[inline]
    #[must_use]
    pub const fn alias(self) -> Sym {
        self.alias
    }

    /// Gets the private name of the export inside the module.
    #[inline]
    #[must_use]
    pub const fn private_name(self) -> Sym {
        self.private_name
    }
}

impl VisitWith for ExportSpecifier {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_sym(&self.alias));
        visitor.visit_sym(&self.private_name)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_sym_mut(&mut self.alias));
        visitor.visit_sym_mut(&mut self.private_name)
    }
}
