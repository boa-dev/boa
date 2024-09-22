use crate::vm::{BindingOpcode, Opcode};

use super::{ByteCompiler, Literal, Operand, ToJsString};
use boa_ast::{declaration::ExportDeclaration, ModuleItem, ModuleItemList};
use boa_interner::Sym;

impl ByteCompiler<'_> {
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
                    ExportDeclaration::DefaultFunctionDeclaration(_)
                    | ExportDeclaration::DefaultGeneratorDeclaration(_)
                    | ExportDeclaration::DefaultAsyncFunctionDeclaration(_)
                    | ExportDeclaration::DefaultAsyncGeneratorDeclaration(_) => {
                        // Already instantiated in `initialize_environment`.
                    }
                    ExportDeclaration::VarStatement(var) => self.compile_var_decl(var),
                    ExportDeclaration::Declaration(decl) => self.compile_decl(decl, false),
                    ExportDeclaration::DefaultClassDeclaration(cl) => self.class(cl.into(), false),
                    ExportDeclaration::DefaultAssignmentExpression(expr) => {
                        let name = Sym::DEFAULT_EXPORT.to_js_string(self.interner());
                        self.compile_expr(expr, true);

                        if expr.is_anonymous_function_definition() {
                            let default = self
                                .interner()
                                .resolve_expect(Sym::DEFAULT)
                                .into_common(false);
                            self.emit_push_literal(Literal::String(default));
                            self.emit_opcode(Opcode::Swap);
                            self.emit(Opcode::SetFunctionName, &[Operand::U8(0)]);
                        }

                        self.emit_binding(BindingOpcode::InitLexical, name);
                    }
                }
            }
        }
    }
}
