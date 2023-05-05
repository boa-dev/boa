use crate::{js_string, vm::Opcode};

use super::{ByteCompiler, Literal};
use boa_ast::{ModuleItem, ModuleItemList};

impl ByteCompiler<'_, '_> {
    /// Compiles a [`ModuleItemList`].
    #[inline]
    pub fn compile_module_item_list(&mut self, list: &ModuleItemList) {
        for node in list.items() {
            self.compile_module_item(node);
        }
    }

    /// Compiles a [`ModuleItem`].
    #[inline]
    #[allow(clippy::single_match_else)]
    pub fn compile_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::StatementListItem(stmt) => {
                self.compile_stmt_list_item(stmt, false);
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
}
