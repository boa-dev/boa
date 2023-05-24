use crate::vm::{BindingOpcode, Opcode};

use super::{ByteCompiler, Literal};
use boa_ast::{declaration::ExportDeclaration, expression::Identifier, ModuleItem, ModuleItemList};
use boa_interner::Sym;

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
    pub fn compile_module_item(&mut self, item: &ModuleItem) {
        match item {
            ModuleItem::StatementListItem(stmt) => {
                self.compile_stmt_list_item(stmt, false, false);
            }
            ModuleItem::ImportDeclaration(_) => {
                // ModuleItem : ImportDeclaration

                // 1. Return empty.
            }
            ModuleItem::ExportDeclaration(export) => {
                #[allow(clippy::match_same_arms)]
                match export {
                    ExportDeclaration::ReExport { .. } | ExportDeclaration::List(_) => {
                        // ExportDeclaration :
                        //    export ExportFromClause FromClause ;
                        //    export NamedExports ;
                        //        1. Return empty.
                    }
                    ExportDeclaration::DefaultFunction(_)
                    | ExportDeclaration::DefaultGenerator(_)
                    | ExportDeclaration::DefaultAsyncFunction(_)
                    | ExportDeclaration::DefaultAsyncGenerator(_) => {
                        // Already instantiated in `initialize_environment`.
                    }
                    ExportDeclaration::VarStatement(var) => self.compile_var_decl(var),
                    ExportDeclaration::Declaration(decl) => self.compile_decl(decl, false),
                    ExportDeclaration::DefaultClassDeclaration(cl) => {
                        self.class(cl, cl.name().is_none());
                        if cl.name().is_none() {
                            self.emit_binding(
                                BindingOpcode::InitLet,
                                Identifier::from(Sym::DEFAULT_EXPORT),
                            );
                        }
                    }
                    ExportDeclaration::DefaultAssignmentExpression(expr) => {
                        let name = Identifier::from(Sym::DEFAULT_EXPORT);
                        self.create_mutable_binding(name, false);
                        self.compile_expr(expr, true);

                        if expr.is_anonymous_function_definition() {
                            let default = self
                                .interner()
                                .resolve_expect(Sym::DEFAULT)
                                .into_common(false);
                            self.emit_push_literal(Literal::String(default));
                            self.emit_opcode(Opcode::Swap);
                            self.emit_opcode(Opcode::SetFunctionName);
                            self.emit_u8(0);
                        }

                        self.emit_binding(BindingOpcode::InitLet, name);
                    }
                }
            }
        }
    }
}
