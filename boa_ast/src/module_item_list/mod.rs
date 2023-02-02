//! Module item list AST nodes.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-modules

use std::{convert::Infallible, ops::ControlFlow};

use boa_interner::Sym;
use rustc_hash::FxHashSet;

use crate::{
    declaration::{ExportDeclaration, ExportSpecifier, ImportDeclaration, ReExportKind},
    expression::Identifier,
    operations::BoundNamesVisitor,
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
    StatementListItem,
};

/// Module item list AST node.
///
/// It contains a list of
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ModuleItemList
#[derive(Debug, Clone)]
pub struct ModuleItemList {
    items: Box<[ModuleItem]>,
}

impl ModuleItemList {
    /// Gets the list of module items.
    #[inline]
    #[must_use]
    pub const fn items(&self) -> &[ModuleItem] {
        &self.items
    }

    /// Abstract operation [`ExportedNames`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-exportednames
    #[inline]
    #[must_use]
    pub fn exported_names(&self) -> Vec<Sym> {
        #[derive(Debug)]
        struct ExportedItemsVisitor<'vec>(&'vec mut Vec<Sym>);

        impl<'ast> Visitor<'ast> for ExportedItemsVisitor<'_> {
            type BreakTy = Infallible;

            fn visit_import_declaration(
                &mut self,
                _: &'ast ImportDeclaration,
            ) -> ControlFlow<Self::BreakTy> {
                ControlFlow::Continue(())
            }
            fn visit_statement_list_item(
                &mut self,
                _: &'ast StatementListItem,
            ) -> ControlFlow<Self::BreakTy> {
                ControlFlow::Continue(())
            }
            fn visit_export_specifier(
                &mut self,
                node: &'ast ExportSpecifier,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.push(node.alias());
                ControlFlow::Continue(())
            }
            fn visit_export_declaration(
                &mut self,
                node: &'ast ExportDeclaration,
            ) -> ControlFlow<Self::BreakTy> {
                match node {
                    ExportDeclaration::ReExport { kind, .. } => {
                        match kind {
                            ReExportKind::Namespaced { name: Some(name) } => self.0.push(*name),
                            ReExportKind::Namespaced { name: None } => {}
                            ReExportKind::Named { names } => {
                                for specifier in &**names {
                                    try_break!(self.visit_export_specifier(specifier));
                                }
                            }
                        }
                        ControlFlow::Continue(())
                    }
                    ExportDeclaration::List(list) => {
                        for specifier in &**list {
                            try_break!(self.visit_export_specifier(specifier));
                        }
                        ControlFlow::Continue(())
                    }
                    ExportDeclaration::VarStatement(var) => {
                        BoundNamesVisitor(self.0).visit_var_declaration(var)
                    }
                    ExportDeclaration::Declaration(decl) => {
                        BoundNamesVisitor(self.0).visit_declaration(decl)
                    }
                    ExportDeclaration::DefaultFunction(_)
                    | ExportDeclaration::DefaultGenerator(_)
                    | ExportDeclaration::DefaultAsyncFunction(_)
                    | ExportDeclaration::DefaultAsyncGenerator(_)
                    | ExportDeclaration::DefaultClassDeclaration(_)
                    | ExportDeclaration::DefaultAssignmentExpression(_) => {
                        self.0.push(Sym::DEFAULT);
                        ControlFlow::Continue(())
                    }
                }
            }
        }

        let mut names = Vec::new();

        ExportedItemsVisitor(&mut names).visit_module_item_list(self);

        names
    }

    /// Abstract operation [`ExportedBindings`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-exportedbindings
    #[inline]
    #[must_use]
    pub fn exported_bindings(&self) -> FxHashSet<Identifier> {
        #[derive(Debug)]
        struct ExportedBindingsVisitor<'vec>(&'vec mut FxHashSet<Identifier>);

        impl<'ast> Visitor<'ast> for ExportedBindingsVisitor<'_> {
            type BreakTy = Infallible;

            fn visit_import_declaration(
                &mut self,
                _: &'ast ImportDeclaration,
            ) -> ControlFlow<Self::BreakTy> {
                ControlFlow::Continue(())
            }
            fn visit_statement_list_item(
                &mut self,
                _: &'ast StatementListItem,
            ) -> ControlFlow<Self::BreakTy> {
                ControlFlow::Continue(())
            }
            fn visit_export_specifier(
                &mut self,
                node: &'ast ExportSpecifier,
            ) -> ControlFlow<Self::BreakTy> {
                self.0.insert(Identifier::new(node.private_name()));
                ControlFlow::Continue(())
            }
            fn visit_export_declaration(
                &mut self,
                node: &'ast ExportDeclaration,
            ) -> ControlFlow<Self::BreakTy> {
                let name = match node {
                    ExportDeclaration::ReExport { .. } => return ControlFlow::Continue(()),
                    ExportDeclaration::List(list) => {
                        for specifier in &**list {
                            try_break!(self.visit_export_specifier(specifier));
                        }
                        return ControlFlow::Continue(());
                    }
                    ExportDeclaration::DefaultAssignmentExpression(expr) => {
                        return BoundNamesVisitor(self.0).visit_expression(expr);
                    }
                    ExportDeclaration::VarStatement(var) => {
                        return BoundNamesVisitor(self.0).visit_var_declaration(var);
                    }
                    ExportDeclaration::Declaration(decl) => {
                        return BoundNamesVisitor(self.0).visit_declaration(decl);
                    }
                    ExportDeclaration::DefaultFunction(f) => f.name(),
                    ExportDeclaration::DefaultGenerator(g) => g.name(),
                    ExportDeclaration::DefaultAsyncFunction(af) => af.name(),
                    ExportDeclaration::DefaultAsyncGenerator(ag) => ag.name(),
                    ExportDeclaration::DefaultClassDeclaration(cl) => cl.name(),
                };

                self.0
                    .insert(name.unwrap_or_else(|| Identifier::new(Sym::DEFAULT_EXPORT)));

                ControlFlow::Continue(())
            }
        }

        let mut names = FxHashSet::default();

        ExportedBindingsVisitor(&mut names).visit_module_item_list(self);

        names
    }
}

impl<T> From<T> for ModuleItemList
where
    T: Into<Box<[ModuleItem]>>,
{
    #[inline]
    fn from(items: T) -> Self {
        Self {
            items: items.into(),
        }
    }
}

impl VisitWith for ModuleItemList {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        for item in &*self.items {
            try_break!(visitor.visit_module_item(item));
        }

        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        for item in &mut *self.items {
            try_break!(visitor.visit_module_item_mut(item));
        }

        ControlFlow::Continue(())
    }
}

/// Module item AST node.
///
/// This is an extension over a [`StatementList`](crate::StatementList), which can also include
/// multiple [`ImportDeclaration`] and [`ExportDeclaration`] nodes, along with
/// [`StatementListItem`] nodes.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ModuleItem
#[derive(Debug, Clone)]
pub enum ModuleItem {
    /// See [`ImportDeclaration`].
    ImportDeclaration(ImportDeclaration),
    /// See [`ExportDeclaration`].
    ExportDeclaration(ExportDeclaration),
    /// See [`StatementListItem`].
    StatementListItem(StatementListItem),
}

impl VisitWith for ModuleItem {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Self::ImportDeclaration(i) => visitor.visit_import_declaration(i),
            Self::ExportDeclaration(e) => visitor.visit_export_declaration(e),
            Self::StatementListItem(s) => visitor.visit_statement_list_item(s),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Self::ImportDeclaration(i) => visitor.visit_import_declaration_mut(i),
            Self::ExportDeclaration(e) => visitor.visit_export_declaration_mut(e),
            Self::StatementListItem(s) => visitor.visit_statement_list_item_mut(s),
        }
    }
}
