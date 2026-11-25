use super::{ByteCompiler, ToJsString};
use crate::vm::opcode::BindingOpcode;
use boa_ast::{ModuleItem, ModuleItemList, declaration::ExportDeclaration};
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
                match export.as_ref() {
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
                        // 2. Let className be the sole element of BoundNames of ClassDeclaration.
                        // 3. If className is "*default*", then
                        if cl.name().sym() == Sym::DEFAULT_EXPORT {
                            let class = self.register_allocator.alloc();

                            // 1. Let value be ? BindingClassDeclarationEvaluation of ClassDeclaration.
                            self.compile_class(cl.as_ref().into(), Some(&class));

                            // a. Let env be the running execution context's LexicalEnvironment.
                            // b. Perform ? InitializeBoundName("*default*", value, env).
                            let default_export = self
                                .interner()
                                .resolve_expect(Sym::DEFAULT_EXPORT)
                                .into_common(false);
                            self.emit_binding(BindingOpcode::InitLexical, default_export, &class);

                            self.register_allocator.dealloc(class);
                        } else {
                            // 1. Let value be ? BindingClassDeclarationEvaluation of ClassDeclaration.
                            self.compile_class(cl.as_ref().into(), None);
                        }
                    }
                    ExportDeclaration::DefaultAssignmentExpression(expr) => {
                        // 1. If IsAnonymousFunctionDefinition(AssignmentExpression) is true, then
                        //     a. Let value be ? NamedEvaluation of AssignmentExpression with argument "default".
                        //    The parser has already named it "default".
                        // 2. Else,
                        //     a. Let rhs be ? Evaluation of AssignmentExpression.
                        //     b. Let value be ? GetValue(rhs).
                        let function = self.register_allocator.alloc();
                        self.compile_expr(expr, &function);

                        // 3. Let env be the running execution context's LexicalEnvironment.
                        // 4. Perform ? InitializeBoundName("*default*", value, env).
                        let name = Sym::DEFAULT_EXPORT.to_js_string(self.interner());
                        self.emit_binding(BindingOpcode::InitLexical, name, &function);
                        self.register_allocator.dealloc(function);
                    }
                }
            }
        }
    }
}
