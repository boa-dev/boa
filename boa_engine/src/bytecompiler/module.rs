use crate::{js_string, vm::Opcode};

use super::{ByteCompiler, Literal};
use boa_ast::{ModuleItem, ModuleItemList};

impl ByteCompiler<'_, '_> {
    /// Compiles a [`ModuleItemList`].
    #[inline]
    pub fn compile_module_item_list(&mut self, list: &ModuleItemList, configurable_globals: bool) {
        for node in list.items() {
            self.compile_module_item(node, configurable_globals);
        }
    }

    /// Compiles a [`ModuleItem`].
    #[inline]
    #[allow(clippy::single_match_else)]
    pub fn compile_module_item(&mut self, item: &ModuleItem, configurable_globals: bool) {
        match item {
            ModuleItem::StatementListItem(stmt) => {
                self.compile_stmt_list_item(stmt, false, configurable_globals);
            }
            _ => {
                // TODO: Remove after implementing modules.
                let msg = self.get_or_insert_literal(Literal::String(js_string!(
                    "modules are unimplemented"
                )));
                self.emit(Opcode::ThrowNewTypeError, &[msg]);
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
    pub(crate) fn create_decls_from_module_item(
        &mut self,
        item: &ModuleItem,
        configurable_globals: bool,
    ) -> bool {
        match item {
            ModuleItem::StatementListItem(stmt) => {
                self.create_decls_from_stmt_list_item(stmt, configurable_globals)
            }
            // TODO: Implement modules
            _ => false,
        }
    }
}
