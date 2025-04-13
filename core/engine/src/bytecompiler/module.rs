use super::{ByteCompiler, Literal, ToJsString};
use crate::vm::opcode::BindingOpcode;
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
                    ExportDeclaration::DefaultClassDeclaration(cl) => {
                        self.compile_class(cl.into(), None);
                    }
                    ExportDeclaration::DefaultAssignmentExpression(expr) => {
                        let function = self.register_allocator.alloc();
                        self.compile_expr(expr, &function);

                        if expr.is_anonymous_function_definition() {
                            let default = self
                                .interner()
                                .resolve_expect(Sym::DEFAULT)
                                .into_common(false);
                            let key = self.register_allocator.alloc();
                            self.emit_push_literal(Literal::String(default), &key);
                            self.bytecode.emit_set_function_name(
                                function.variable(),
                                key.variable(),
                                0u32.into(),
                            );
                            self.register_allocator.dealloc(key);
                        }

                        let name = Sym::DEFAULT_EXPORT.to_js_string(self.interner());
                        self.emit_binding(BindingOpcode::InitLexical, name, &function);
                        self.register_allocator.dealloc(function);
                    }
                }
            }
        }
    }
}
