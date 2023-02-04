use boa_ast::{ModuleItem, ModuleItemList};

use crate::JsResult;

use super::ByteCompiler;

impl ByteCompiler<'_, '_> {
    /// Compiles a [`ModuleItemList`].
    #[inline]
    pub fn compile_module_item_list(
        &mut self,
        list: &ModuleItemList,
        configurable_globals: bool,
    ) -> JsResult<()> {
        for node in list.items() {
            self.compile_module_item(node, configurable_globals)?;
        }
        Ok(())
    }

    /// Compiles a [`ModuleItem`].
    #[inline]
    #[allow(unused_variables, clippy::missing_panics_doc)] // Unimplemented
    pub fn compile_module_item(
        &mut self,
        item: &ModuleItem,
        configurable_globals: bool,
    ) -> JsResult<()> {
        match item {
            ModuleItem::ImportDeclaration(import) => todo!("import declaration compilation"),
            ModuleItem::ExportDeclaration(export) => todo!("export declaration compilation"),
            ModuleItem::StatementListItem(stmt) => {
                self.compile_stmt_list_item(stmt, false, configurable_globals)
            }
        }
    }

    /// Creates the declarations for a module.
    pub(crate) fn create_module_decls(
        &mut self,
        stmt_list: &ModuleItemList,
        configurable_globals: bool,
    ) {
        for node in stmt_list.items() {
            self.create_decls_from_module_item(node, configurable_globals);
        }
    }

    /// Creates the declarations from a [`ModuleItem`].
    #[inline]
    #[allow(unused_variables)] // Unimplemented
    pub(crate) fn create_decls_from_module_item(
        &mut self,
        item: &ModuleItem,
        configurable_globals: bool,
    ) -> bool {
        match item {
            ModuleItem::ImportDeclaration(import) => todo!("import declaration generation"),
            ModuleItem::ExportDeclaration(export) => todo!("export declaration generation"),
            ModuleItem::StatementListItem(stmt) => {
                self.create_decls_from_stmt_list_item(stmt, configurable_globals)
            }
        }
    }
}
