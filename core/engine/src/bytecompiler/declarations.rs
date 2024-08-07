use std::rc::Rc;

use crate::{
    bytecompiler::{ByteCompiler, FunctionCompiler, FunctionSpec, NodeKind},
    environments::CompileTimeEnvironment,
    vm::{BindingOpcode, Opcode},
    Context, JsNativeError, JsResult,
};
use boa_ast::{
    declaration::{Binding, LexicalDeclaration, VariableList},
    expression::Identifier,
    function::{FormalParameterList, FunctionBody},
    operations::{
        all_private_identifiers_valid, bound_names, lexically_declared_names,
        lexically_scoped_declarations, var_declared_names, var_scoped_declarations,
        LexicallyScopedDeclaration, VarScopedDeclaration,
    },
    visitor::NodeRef,
    Declaration, Script, StatementListItem,
};
use boa_interner::{JStrRef, Sym};

#[cfg(feature = "annex-b")]
use boa_ast::operations::annex_b_function_declarations_names;

use super::{Operand, ToJsString};

/// `GlobalDeclarationInstantiation ( script, env )`
///
/// This diverges from the specification by separating the context from the compilation process.
/// Many steps are skipped that are done during bytecode compilation.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-globaldeclarationinstantiation
#[cfg(not(feature = "annex-b"))]
#[allow(clippy::unnecessary_wraps)]
#[allow(clippy::ptr_arg)]
pub(crate) fn global_declaration_instantiation_context(
    _annex_b_function_names: &mut Vec<Identifier>,
    _script: &Script,
    _env: &Rc<CompileTimeEnvironment>,
    _context: &mut Context,
) -> JsResult<()> {
    Ok(())
}

/// `GlobalDeclarationInstantiation ( script, env )`
///
/// This diverges from the specification by separating the context from the compilation process.
/// Many steps are skipped that are done during bytecode compilation.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-globaldeclarationinstantiation
#[cfg(feature = "annex-b")]
pub(crate) fn global_declaration_instantiation_context(
    annex_b_function_names: &mut Vec<Identifier>,
    script: &Script,
    env: &Rc<CompileTimeEnvironment>,
    context: &mut Context,
) -> JsResult<()> {
    // SKIP: 1. Let lexNames be the LexicallyDeclaredNames of script.
    // SKIP: 2. Let varNames be the VarDeclaredNames of script.
    // SKIP: 3. For each element name of lexNames, do
    // SKIP: 4. For each element name of varNames, do

    // 5. Let varDeclarations be the VarScopedDeclarations of script.
    // Note: VarScopedDeclarations for a Script node is TopLevelVarScopedDeclarations.
    let var_declarations = var_scoped_declarations(script);

    // SKIP: 6. Let functionsToInitialize be a new empty List.

    // 7. Let declaredFunctionNames be a new empty List.
    let mut declared_function_names = Vec::new();

    // 8. For each element d of varDeclarations, in reverse List order, do
    for declaration in var_declarations.iter().rev() {
        // a. If d is not either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
        // a.i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
        // a.ii. NOTE: If there are multiple function declarations for the same name, the last declaration is used.
        let name = match declaration {
            VarScopedDeclaration::Function(f) => f.name(),
            VarScopedDeclaration::Generator(f) => f.name(),
            VarScopedDeclaration::AsyncFunction(f) => f.name(),
            VarScopedDeclaration::AsyncGenerator(f) => f.name(),
            VarScopedDeclaration::VariableDeclaration(_) => {
                continue;
            }
        };

        // a.iii. Let fn be the sole element of the BoundNames of d.
        let name = name.expect("function declaration must have a name");

        // a.iv. If declaredFunctionNames does not contain fn, then
        if !declared_function_names.contains(&name) {
            // SKIP: 1. Let fnDefinable be ? env.CanDeclareGlobalFunction(fn).
            // SKIP: 2. If fnDefinable is false, throw a TypeError exception.
            // 3. Append fn to declaredFunctionNames.
            declared_function_names.push(name);

            // SKIP: 4. Insert d as the first element of functionsToInitialize.
        }
    }

    // // 9. Let declaredVarNames be a new empty List.
    let mut declared_var_names = Vec::new();

    // 10. For each element d of varDeclarations, do
    //     a. If d is either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
    for declaration in var_declarations {
        let VarScopedDeclaration::VariableDeclaration(declaration) = declaration else {
            continue;
        };

        // i. For each String vn of the BoundNames of d, do
        for name in bound_names(&declaration) {
            // 1. If declaredFunctionNames does not contain vn, then
            if !declared_function_names.contains(&name) {
                // SKIP: a. Let vnDefinable be ? env.CanDeclareGlobalVar(vn).
                // SKIP: b. If vnDefinable is false, throw a TypeError exception.
                // c. If declaredVarNames does not contain vn, then
                if !declared_var_names.contains(&name) {
                    // i. Append vn to declaredVarNames.
                    declared_var_names.push(name);
                }
            }
        }
    }

    // 11. NOTE: No abnormal terminations occur after this algorithm step if the global object is an ordinary object.
    //     However, if the global object is a Proxy exotic object it may exhibit behaviours
    //     that cause abnormal terminations in some of the following steps.

    // 12. NOTE: Annex B.3.2.2 adds additional steps at this point.
    // 12. Perform the following steps:
    // a. Let strict be IsStrict of script.
    // b. If strict is false, then
    if !script.strict() {
        let lex_names = lexically_declared_names(script);

        // i. Let declaredFunctionOrVarNames be the list-concatenation of declaredFunctionNames and declaredVarNames.
        // ii. For each FunctionDeclaration f that is directly contained in the StatementList of a Block, CaseClause,
        //     or DefaultClause Contained within script, do
        for f in annex_b_function_declarations_names(script) {
            // 1. Let F be StringValue of the BindingIdentifier of f.
            // 2. If replacing the FunctionDeclaration f with a VariableStatement that has F as a BindingIdentifier
            //    would not produce any Early Errors for script, then
            if !lex_names.contains(&f) {
                let f_string = f.to_js_string(context.interner());

                // a. If env.HasLexicalDeclaration(F) is false, then
                if !env.has_lex_binding(&f_string) {
                    // i. Let fnDefinable be ? env.CanDeclareGlobalVar(F).
                    let fn_definable = context.can_declare_global_function(&f_string)?;

                    // ii. If fnDefinable is true, then
                    if fn_definable {
                        // i. NOTE: A var binding for F is only instantiated here if it is neither
                        //          a VarDeclaredName nor the name of another FunctionDeclaration.
                        // ii. If declaredFunctionOrVarNames does not contain F, then
                        if !declared_function_names.contains(&f) && !declared_var_names.contains(&f)
                        {
                            // i. Perform ? env.CreateGlobalVarBinding(F, false).
                            context.create_global_var_binding(f_string, false)?;

                            // ii. Append F to declaredFunctionOrVarNames.
                            declared_function_names.push(f);
                        }
                        // iii. When the FunctionDeclaration f is evaluated, perform the following
                        //      steps in place of the FunctionDeclaration Evaluation algorithm provided in 15.2.6:
                        //     i. Let genv be the running execution context's VariableEnvironment.
                        //     ii. Let benv be the running execution context's LexicalEnvironment.
                        //     iii. Let fobj be ! benv.GetBindingValue(F, false).
                        //     iv. Perform ? genv.SetMutableBinding(F, fobj, false).
                        //     v. Return unused.
                        annex_b_function_names.push(f);
                    }
                }
            }
        }
    }

    // SKIP: 13. Let lexDeclarations be the LexicallyScopedDeclarations of script.
    // SKIP: 14. Let privateEnv be null.
    // SKIP: 15. For each element d of lexDeclarations, do
    // SKIP: 16. For each Parse Node f of functionsToInitialize, do
    // SKIP: 17. For each String vn of declaredVarNames, do

    // 18. Return unused.
    Ok(())
}

/// `EvalDeclarationInstantiation ( body, varEnv, lexEnv, privateEnv, strict )`
///
/// This diverges from the specification by separating the context from the compilation process.
/// Many steps are skipped that are done during bytecode compilation.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-evaldeclarationinstantiation
pub(crate) fn eval_declaration_instantiation_context(
    #[allow(unused, clippy::ptr_arg)] annex_b_function_names: &mut Vec<Identifier>,
    body: &Script,
    #[allow(unused)] strict: bool,
    #[allow(unused)] var_env: &Rc<CompileTimeEnvironment>,
    #[allow(unused)] lex_env: &Rc<CompileTimeEnvironment>,
    context: &mut Context,
) -> JsResult<()> {
    // SKIP: 3. If strict is false, then

    // 4. Let privateIdentifiers be a new empty List.
    // 5. Let pointer be privateEnv.
    // 6. Repeat, while pointer is not null,
    //     a. For each Private Name binding of pointer.[[Names]], do
    //         i. If privateIdentifiers does not contain binding.[[Description]],
    //            append binding.[[Description]] to privateIdentifiers.
    //     b. Set pointer to pointer.[[OuterPrivateEnvironment]].
    let private_identifiers = context.vm.environments.private_name_descriptions();
    let private_identifiers = private_identifiers
        .into_iter()
        .map(|ident| {
            // TODO: Replace JStrRef with JsStr this would eliminate the to_vec call.
            let ident = ident.to_vec();
            context
                .interner()
                .get(JStrRef::Utf16(&ident))
                .expect("string should be in interner")
        })
        .collect();

    // 7. If AllPrivateIdentifiersValid of body with argument privateIdentifiers is false, throw a SyntaxError exception.
    if !all_private_identifiers_valid(body, private_identifiers) {
        return Err(JsNativeError::syntax()
            .with_message("invalid private identifier")
            .into());
    }

    // 2. Let varDeclarations be the VarScopedDeclarations of body.
    #[cfg(feature = "annex-b")]
    let var_declarations = var_scoped_declarations(body);

    // SKIP: 8. Let functionsToInitialize be a new empty List.

    // 9. Let declaredFunctionNames be a new empty List.
    #[cfg(feature = "annex-b")]
    let mut declared_function_names = Vec::new();

    // 10. For each element d of varDeclarations, in reverse List order, do
    #[cfg(feature = "annex-b")]
    for declaration in var_declarations.iter().rev() {
        // a. If d is not either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
        // a.i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
        // a.ii. NOTE: If there are multiple function declarations for the same name, the last declaration is used.
        let name = match &declaration {
            VarScopedDeclaration::Function(f) => f.name(),
            VarScopedDeclaration::Generator(f) => f.name(),
            VarScopedDeclaration::AsyncFunction(f) => f.name(),
            VarScopedDeclaration::AsyncGenerator(f) => f.name(),
            VarScopedDeclaration::VariableDeclaration(_) => {
                continue;
            }
        };

        // a.iii. Let fn be the sole element of the BoundNames of d.
        let name = name.expect("function declaration must have a name");

        // a.iv. If declaredFunctionNames does not contain fn, then
        if !declared_function_names.contains(&name) {
            // SKIP: 1. If varEnv is a Global Environment Record, then

            // 2. Append fn to declaredFunctionNames.
            declared_function_names.push(name);

            // SKIP: 3. Insert d as the first element of functionsToInitialize.
        }
    }

    // 11. NOTE: Annex B.3.2.3 adds additional steps at this point.
    // 11. If strict is false, then
    #[cfg(feature = "annex-b")]
    if !strict {
        let lexically_declared_names = lexically_declared_names(body);

        // a. Let declaredFunctionOrVarNames be the list-concatenation of declaredFunctionNames and declaredVarNames.
        // b. For each FunctionDeclaration f that is directly contained in the StatementList
        //    of a Block, CaseClause, or DefaultClause Contained within body, do
        for f in annex_b_function_declarations_names(body) {
            // i. Let F be StringValue of the BindingIdentifier of f.
            // ii. If replacing the FunctionDeclaration f with a VariableStatement that has F
            //     as a BindingIdentifier would not produce any Early Errors for body, then
            if !lexically_declared_names.contains(&f) {
                // 1. Let bindingExists be false.
                let mut binding_exists = false;

                // 2. Let thisEnv be lexEnv.
                let mut this_env = lex_env.clone();

                // 3. Assert: The following loop will terminate.
                // 4. Repeat, while thisEnv is not varEnv,
                while this_env.environment_index() != lex_env.environment_index() {
                    let f = f.to_js_string(context.interner());

                    // a. If thisEnv is not an Object Environment Record, then
                    // i. If ! thisEnv.HasBinding(F) is true, then
                    if this_env.has_binding(&f) {
                        // i. Let bindingExists be true.
                        binding_exists = true;
                        break;
                    }

                    // b. Set thisEnv to thisEnv.[[OuterEnv]].
                    if let Some(outer) = this_env.outer() {
                        this_env = outer;
                    } else {
                        break;
                    }
                }

                // 5. If bindingExists is false and varEnv is a Global Environment Record, then
                let fn_definable = if !binding_exists && var_env.is_global() {
                    let f = f.to_js_string(context.interner());

                    // a. If varEnv.HasLexicalDeclaration(F) is false, then
                    // b. Else,
                    if var_env.has_lex_binding(&f) {
                        // i. Let fnDefinable be false.
                        false
                    } else {
                        // i. Let fnDefinable be ? varEnv.CanDeclareGlobalVar(F).
                        context.can_declare_global_var(&f)?
                    }
                }
                // 6. Else,
                else {
                    // a. Let fnDefinable be true.
                    true
                };

                // 7. If bindingExists is false and fnDefinable is true, then
                if !binding_exists && fn_definable {
                    // a. If declaredFunctionOrVarNames does not contain F, then
                    if !declared_function_names.contains(&f) {
                        // i. If varEnv is a Global Environment Record, then
                        if var_env.is_global() {
                            let f = f.to_js_string(context.interner());

                            // i. Perform ? varEnv.CreateGlobalVarBinding(F, true).
                            context.create_global_var_binding(f, true)?;
                        }

                        // SKIP: ii. Else,
                        // SKIP: iii. Append F to declaredFunctionOrVarNames.
                    }

                    // b. When the FunctionDeclaration f is evaluated, perform the following steps
                    //    in place of the FunctionDeclaration Evaluation algorithm provided in 15.2.6:
                    //     i. Let genv be the running execution context's VariableEnvironment.
                    //     ii. Let benv be the running execution context's LexicalEnvironment.
                    //     iii. Let fobj be ! benv.GetBindingValue(F, false).
                    //     iv. Perform ? genv.SetMutableBinding(F, fobj, false).
                    //     v. Return unused.
                    annex_b_function_names.push(f);
                }
            }
        }
    }

    // SKIP: 12. Let declaredVarNames be a new empty List.
    // SKIP: 13. For each element d of varDeclarations, do
    // SKIP: 14. NOTE: No abnormal terminations occur after this algorithm step unless varEnv is a
    //           Global Environment Record and the global object is a Proxy exotic object.
    // SKIP: 15. Let lexDeclarations be the LexicallyScopedDeclarations of body.
    // SKIP: 16. For each element d of lexDeclarations, do
    // SKIP: 17. For each Parse Node f of functionsToInitialize, do
    // SKIP: 18. For each String vn of declaredVarNames, do

    // 19. Return unused.
    Ok(())
}

impl ByteCompiler<'_> {
    /// `GlobalDeclarationInstantiation ( script, env )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-globaldeclarationinstantiation
    pub(crate) fn global_declaration_instantiation(
        &mut self,
        script: &Script,
        env: &Rc<CompileTimeEnvironment>,
    ) {
        // 1. Let lexNames be the LexicallyDeclaredNames of script.
        let lex_names = lexically_declared_names(script);

        // 2. Let varNames be the VarDeclaredNames of script.
        let var_names = var_declared_names(script);

        // 3. For each element name of lexNames, do
        for name in lex_names {
            let name = name.to_js_string(self.interner());

            // Note: Our implementation differs from the spec here.
            // a. If env.HasVarDeclaration(name) is true, throw a SyntaxError exception.
            // b. If env.HasLexicalDeclaration(name) is true, throw a SyntaxError exception.
            if env.has_binding(&name) {
                self.emit_syntax_error("duplicate lexical declaration");
                return;
            }

            // c. Let hasRestrictedGlobal be ? env.HasRestrictedGlobalProperty(name).
            let index = self.get_or_insert_string(name);
            self.emit_with_varying_operand(Opcode::HasRestrictedGlobalProperty, index);

            // d. If hasRestrictedGlobal is true, throw a SyntaxError exception.
            let exit = self.jump_if_false();
            self.emit_syntax_error("cannot redefine non-configurable global property");
            self.patch_jump(exit);
        }

        // 4. For each element name of varNames, do
        for name in var_names {
            let name = name.to_js_string(self.interner());

            // a. If env.HasLexicalDeclaration(name) is true, throw a SyntaxError exception.
            if env.has_lex_binding(&name) {
                self.emit_syntax_error("duplicate lexical declaration");
                return;
            }
        }

        // 5. Let varDeclarations be the VarScopedDeclarations of script.
        // Note: VarScopedDeclarations for a Script node is TopLevelVarScopedDeclarations.
        let var_declarations = var_scoped_declarations(script);

        // 6. Let functionsToInitialize be a new empty List.
        let mut functions_to_initialize = Vec::new();

        // 7. Let declaredFunctionNames be a new empty List.
        let mut declared_function_names = Vec::new();

        // 8. For each element d of varDeclarations, in reverse List order, do
        for declaration in var_declarations.iter().rev() {
            // a. If d is not either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
            // a.i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
            // a.ii. NOTE: If there are multiple function declarations for the same name, the last declaration is used.
            let name = match declaration {
                VarScopedDeclaration::Function(f) => f.name(),
                VarScopedDeclaration::Generator(f) => f.name(),
                VarScopedDeclaration::AsyncFunction(f) => f.name(),
                VarScopedDeclaration::AsyncGenerator(f) => f.name(),
                VarScopedDeclaration::VariableDeclaration(_) => {
                    continue;
                }
            };

            // a.iii. Let fn be the sole element of the BoundNames of d.
            let name = name.expect("function declaration must have a name");

            // a.iv. If declaredFunctionNames does not contain fn, then
            if !declared_function_names.contains(&name) {
                // 1. Let fnDefinable be ? env.CanDeclareGlobalFunction(fn).
                let index = self.get_or_insert_name(name);
                self.emit_with_varying_operand(Opcode::CanDeclareGlobalFunction, index);

                // 2. If fnDefinable is false, throw a TypeError exception.
                let exit = self.jump_if_true();
                self.emit_type_error("cannot declare global function");
                self.patch_jump(exit);

                // 3. Append fn to declaredFunctionNames.
                declared_function_names.push(name);

                // 4. Insert d as the first element of functionsToInitialize.
                functions_to_initialize.push(declaration.clone());
            }
        }

        functions_to_initialize.reverse();

        // 9. Let declaredVarNames be a new empty List.
        let mut declared_var_names = Vec::new();

        // 10. For each element d of varDeclarations, do
        //     a. If d is either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
        for declaration in var_declarations {
            let VarScopedDeclaration::VariableDeclaration(declaration) = declaration else {
                continue;
            };

            // i. For each String vn of the BoundNames of d, do
            for name in bound_names(&declaration) {
                // 1. If declaredFunctionNames does not contain vn, then
                if !declared_function_names.contains(&name) {
                    // a. Let vnDefinable be ? env.CanDeclareGlobalVar(vn).
                    let index = self.get_or_insert_name(name);
                    self.emit_with_varying_operand(Opcode::CanDeclareGlobalVar, index);

                    // b. If vnDefinable is false, throw a TypeError exception.
                    let exit = self.jump_if_true();
                    self.emit_type_error("cannot declare global variable");
                    self.patch_jump(exit);

                    // c. If declaredVarNames does not contain vn, then
                    if !declared_var_names.contains(&name) {
                        // i. Append vn to declaredVarNames.
                        declared_var_names.push(name);
                    }
                }
            }
        }

        // NOTE: These steps depend on the global object are done before bytecode compilation.
        //
        // SKIP: 11. NOTE: No abnormal terminations occur after this algorithm step if the global object is an ordinary object.
        //     However, if the global object is a Proxy exotic object it may exhibit behaviours
        //     that cause abnormal terminations in some of the following steps.
        // SKIP: 12. NOTE: Annex B.3.2.2 adds additional steps at this point.
        // SKIP: 12. Perform the following steps:
        // SKIP: a. Let strict be IsStrict of script.
        // SKIP: b. If strict is false, then

        // 13. Let lexDeclarations be the LexicallyScopedDeclarations of script.
        // 14. Let privateEnv be null.
        // 15. For each element d of lexDeclarations, do
        for statement in &**script.statements() {
            // a. NOTE: Lexically declared names are only instantiated here but not initialized.
            // b. For each element dn of the BoundNames of d, do
            //     i. If IsConstantDeclaration of d is true, then
            //         1. Perform ? env.CreateImmutableBinding(dn, true).
            //     ii. Else,
            //         1. Perform ? env.CreateMutableBinding(dn, false).
            if let StatementListItem::Declaration(declaration) = statement {
                match declaration {
                    Declaration::Class(class) => {
                        for name in bound_names(class) {
                            let name = name.to_js_string(self.interner());
                            env.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                        for name in bound_names(declaration) {
                            let name = name.to_js_string(self.interner());
                            env.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                        for name in bound_names(declaration) {
                            let name = name.to_js_string(self.interner());
                            env.create_immutable_binding(name, true);
                        }
                    }
                    _ => {}
                }
            }
        }

        // 16. For each Parse Node f of functionsToInitialize, do
        for function in functions_to_initialize {
            // a. Let fn be the sole element of the BoundNames of f.
            let (name, generator, r#async, parameters, body) = match &function {
                VarScopedDeclaration::Function(f) => {
                    (f.name(), false, false, f.parameters(), f.body())
                }
                VarScopedDeclaration::Generator(f) => {
                    (f.name(), true, false, f.parameters(), f.body())
                }
                VarScopedDeclaration::AsyncFunction(f) => {
                    (f.name(), false, true, f.parameters(), f.body())
                }
                VarScopedDeclaration::AsyncGenerator(f) => {
                    (f.name(), true, true, f.parameters(), f.body())
                }
                VarScopedDeclaration::VariableDeclaration(_) => {
                    continue;
                }
            };
            let name = name.expect("function declaration must have a name");

            let code = FunctionCompiler::new()
                .name(name.sym().to_js_string(self.interner()))
                .generator(generator)
                .r#async(r#async)
                .strict(self.strict())
                .in_with(self.in_with)
                .binding_identifier(None)
                .compile(
                    parameters,
                    body,
                    self.variable_environment.clone(),
                    self.lexical_environment.clone(),
                    self.interner,
                );

            // Ensures global functions are printed when generating the global flowgraph.
            let function_index = self.push_function_to_constants(code);

            // b. Let fo be InstantiateFunctionObject of f with arguments env and privateEnv.
            let dst = self.register_allocator.alloc();
            self.emit_get_function(&dst, function_index);
            self.push_from_register(&dst);
            self.register_allocator.dealloc(dst);

            // c. Perform ? env.CreateGlobalFunctionBinding(fn, fo, false).
            let name_index = self.get_or_insert_name(name);
            self.emit(
                Opcode::CreateGlobalFunctionBinding,
                &[Operand::Bool(false), Operand::Varying(name_index)],
            );
        }

        // 17. For each String vn of declaredVarNames, do
        for var in declared_var_names {
            // a. Perform ? env.CreateGlobalVarBinding(vn, false).
            let index = self.get_or_insert_name(var);
            self.emit(
                Opcode::CreateGlobalVarBinding,
                &[Operand::Bool(false), Operand::Varying(index)],
            );
        }

        // 18. Return unused.
    }

    /// `BlockDeclarationInstantiation ( code, env )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-blockdeclarationinstantiation
    pub(crate) fn block_declaration_instantiation<'a, N>(
        &mut self,
        block: &'a N,
        env: &Rc<CompileTimeEnvironment>,
    ) where
        &'a N: Into<NodeRef<'a>>,
    {
        // 1. Let declarations be the LexicallyScopedDeclarations of code.
        let declarations = lexically_scoped_declarations(block);

        // 2. Let privateEnv be the running execution context's PrivateEnvironment.
        // Note: Private environments are currently handled differently.

        // 3. For each element d of declarations, do
        for d in &declarations {
            // i. If IsConstantDeclaration of d is true, then
            if let LexicallyScopedDeclaration::LexicalDeclaration(LexicalDeclaration::Const(d)) = d
            {
                // a. For each element dn of the BoundNames of d, do
                for dn in bound_names::<'_, VariableList>(d) {
                    // 1. Perform ! env.CreateImmutableBinding(dn, true).
                    let dn = dn.to_js_string(self.interner());
                    env.create_immutable_binding(dn, true);
                }
            }
            // ii. Else,
            else {
                // a. For each element dn of the BoundNames of d, do
                for dn in d.bound_names() {
                    let dn = dn.to_js_string(self.interner());

                    #[cfg(not(feature = "annex-b"))]
                    // 1. Perform ! env.CreateMutableBinding(dn, false). NOTE: This step is replaced in section B.3.2.6.
                    env.create_mutable_binding(dn, false);

                    #[cfg(feature = "annex-b")]
                    // 1. If ! env.HasBinding(dn) is false, then
                    if !env.has_binding(&dn) {
                        // a. Perform  ! env.CreateMutableBinding(dn, false).
                        env.create_mutable_binding(dn, false);
                    }
                }
            }
        }

        // Note: Not sure if the spec is wrong here or if our implementation just differs too much,
        //       but we need 3.a to be finished for all declarations before 3.b can be done.

        // b. If d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration, then
        //     i. Let fn be the sole element of the BoundNames of d.
        //     ii. Let fo be InstantiateFunctionObject of d with arguments env and privateEnv.
        //     iii. Perform ! env.InitializeBinding(fn, fo). NOTE: This step is replaced in section B.3.2.6.
        // TODO: Support B.3.2.6.
        for d in declarations {
            match d {
                LexicallyScopedDeclaration::Function(function) => {
                    self.function_with_binding(function.into(), NodeKind::Declaration, false);
                }
                LexicallyScopedDeclaration::Generator(function) => {
                    self.function_with_binding(function.into(), NodeKind::Declaration, false);
                }
                LexicallyScopedDeclaration::AsyncFunction(function) => {
                    self.function_with_binding(function.into(), NodeKind::Declaration, false);
                }
                LexicallyScopedDeclaration::AsyncGenerator(function) => {
                    self.function_with_binding(function.into(), NodeKind::Declaration, false);
                }
                _ => {}
            }
        }

        // 4. Return unused.
    }

    /// `EvalDeclarationInstantiation ( body, varEnv, lexEnv, privateEnv, strict )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-evaldeclarationinstantiation
    pub(crate) fn eval_declaration_instantiation(
        &mut self,
        body: &Script,
        strict: bool,
        var_env: &Rc<CompileTimeEnvironment>,
        lex_env: &Rc<CompileTimeEnvironment>,
    ) {
        // 2. Let varDeclarations be the VarScopedDeclarations of body.
        let var_declarations = var_scoped_declarations(body);

        // 3. If strict is false, then
        if !strict {
            // 1. Let varNames be the VarDeclaredNames of body.
            let var_names = var_declared_names(body);

            // a. If varEnv is a Global Environment Record, then
            if var_env.is_global() {
                // i. For each element name of varNames, do
                for name in &var_names {
                    let name = name.to_js_string(self.interner());

                    // 1. If varEnv.HasLexicalDeclaration(name) is true, throw a SyntaxError exception.
                    // 2. NOTE: eval will not create a global var declaration that would be shadowed by a global lexical declaration.
                    if var_env.has_lex_binding(&name) {
                        self.emit_syntax_error("duplicate lexical declaration");
                        return;
                    }
                }
            }

            // b. Let thisEnv be lexEnv.
            let mut this_env = lex_env.clone();

            // c. Assert: The following loop will terminate.
            // d. Repeat, while thisEnv is not varEnv,
            while this_env.environment_index() != var_env.environment_index() {
                // i. If thisEnv is not an Object Environment Record, then
                // 1. NOTE: The environment of with statements cannot contain any lexical
                //    declaration so it doesn't need to be checked for var/let hoisting conflicts.
                // 2. For each element name of varNames, do
                for name in &var_names {
                    let name = self.interner().resolve_expect(name.sym()).utf16().into();

                    // a. If ! thisEnv.HasBinding(name) is true, then
                    if this_env.has_binding(&name) {
                        // i. Throw a SyntaxError exception.
                        // ii. NOTE: Annex B.3.4 defines alternate semantics for the above step.
                        let msg = format!("variable declaration {} in eval function already exists as a lexical variable", name.to_std_string_escaped());
                        self.emit_syntax_error(&msg);
                        return;
                    }
                    // b. NOTE: A direct eval will not hoist var declaration over a like-named lexical declaration.
                }

                // ii. Set thisEnv to thisEnv.[[OuterEnv]].
                if let Some(outer) = this_env.outer() {
                    this_env = outer;
                } else {
                    break;
                }
            }
        }

        // NOTE: These steps depend on the current environment state are done before bytecode compilation,
        //       in `eval_declaration_instantiation_context`.
        //
        // SKIP: 4. Let privateIdentifiers be a new empty List.
        // SKIP: 5. Let pointer be privateEnv.
        // SKIP: 6. Repeat, while pointer is not null,
        //           a. For each Private Name binding of pointer.[[Names]], do
        //               i. If privateIdentifiers does not contain binding.[[Description]],
        //                  append binding.[[Description]] to privateIdentifiers.
        //           b. Set pointer to pointer.[[OuterPrivateEnvironment]].
        // SKIP: 7. If AllPrivateIdentifiersValid of body with argument privateIdentifiers is false, throw a SyntaxError exception.

        // 8. Let functionsToInitialize be a new empty List.
        let mut functions_to_initialize = Vec::new();

        // 9. Let declaredFunctionNames be a new empty List.
        let mut declared_function_names = Vec::new();

        // 10. For each element d of varDeclarations, in reverse List order, do
        for declaration in var_declarations.iter().rev() {
            // a. If d is not either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
            // a.i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
            // a.ii. NOTE: If there are multiple function declarations for the same name, the last declaration is used.
            let name = match &declaration {
                VarScopedDeclaration::Function(f) => f.name(),
                VarScopedDeclaration::Generator(f) => f.name(),
                VarScopedDeclaration::AsyncFunction(f) => f.name(),
                VarScopedDeclaration::AsyncGenerator(f) => f.name(),
                VarScopedDeclaration::VariableDeclaration(_) => {
                    continue;
                }
            };

            // a.iii. Let fn be the sole element of the BoundNames of d.
            let name = name.expect("function declaration must have a name");

            // a.iv. If declaredFunctionNames does not contain fn, then
            if !declared_function_names.contains(&name) {
                // 1. If varEnv is a Global Environment Record, then
                if var_env.is_global() {
                    let index = self.get_or_insert_name(name);

                    // a. Let fnDefinable be ? varEnv.CanDeclareGlobalFunction(fn).
                    self.emit_with_varying_operand(Opcode::CanDeclareGlobalFunction, index);

                    // b. If fnDefinable is false, throw a TypeError exception.
                    let exit = self.jump_if_true();
                    self.emit_type_error("cannot declare global function");
                    self.patch_jump(exit);
                }

                // 2. Append fn to declaredFunctionNames.
                declared_function_names.push(name);

                // 3. Insert d as the first element of functionsToInitialize.
                functions_to_initialize.push(declaration.clone());
            }
        }

        functions_to_initialize.reverse();

        // 11. NOTE: Annex B.3.2.3 adds additional steps at this point.
        // 11. If strict is false, then
        #[cfg(feature = "annex-b")]
        if !strict {
            // NOTE: This diviates from the specification, we split the first part of defining the annex-b names
            //       in `eval_declaration_instantiation_context`, because it depends on the context.
            if !var_env.is_global() {
                for name in self.annex_b_function_names.clone() {
                    let f = name.to_js_string(self.interner());
                    // i. Let bindingExists be ! varEnv.HasBinding(F).
                    // ii. If bindingExists is false, then
                    if !var_env.has_binding(&f) {
                        // i. Perform ! varEnv.CreateMutableBinding(F, true).
                        // ii. Perform ! varEnv.InitializeBinding(F, undefined).
                        let binding = var_env.create_mutable_binding(f, true);
                        let index = self.get_or_insert_binding(binding);
                        self.emit_opcode(Opcode::PushUndefined);
                        self.emit_with_varying_operand(Opcode::DefInitVar, index);
                    }
                }
            }
        }

        // 12. Let declaredVarNames be a new empty List.
        let mut declared_var_names = Vec::new();

        // 13. For each element d of varDeclarations, do
        for declaration in var_declarations {
            // a. If d is either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
            let VarScopedDeclaration::VariableDeclaration(declaration) = declaration else {
                continue;
            };

            // a.i. For each String vn of the BoundNames of d, do
            for name in bound_names(&declaration) {
                // 1. If declaredFunctionNames does not contain vn, then
                if !declared_function_names.contains(&name) {
                    // a. If varEnv is a Global Environment Record, then
                    if var_env.is_global() {
                        let index = self.get_or_insert_name(name);

                        // i. Let vnDefinable be ? varEnv.CanDeclareGlobalVar(vn).
                        self.emit_with_varying_operand(Opcode::CanDeclareGlobalVar, index);

                        // ii. If vnDefinable is false, throw a TypeError exception.
                        let exit = self.jump_if_true();
                        self.emit_type_error("cannot declare global function");
                        self.patch_jump(exit);
                    }

                    // b. If declaredVarNames does not contain vn, then
                    if !declared_var_names.contains(&name) {
                        // i. Append vn to declaredVarNames.
                        declared_var_names.push(name);
                    }
                }
            }
        }

        // 14. NOTE: No abnormal terminations occur after this algorithm step unless varEnv is a
        //           Global Environment Record and the global object is a Proxy exotic object.

        // 15. Let lexDeclarations be the LexicallyScopedDeclarations of body.
        // 16. For each element d of lexDeclarations, do
        for statement in &**body.statements() {
            // a. NOTE: Lexically declared names are only instantiated here but not initialized.
            // b. For each element dn of the BoundNames of d, do
            //     i. If IsConstantDeclaration of d is true, then
            //         1. Perform ? lexEnv.CreateImmutableBinding(dn, true).
            //     ii. Else,
            //         1. Perform ? lexEnv.CreateMutableBinding(dn, false).
            if let StatementListItem::Declaration(declaration) = statement {
                match declaration {
                    Declaration::Class(class) => {
                        for name in bound_names(class) {
                            let name = name.to_js_string(self.interner());
                            lex_env.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                        for name in bound_names(declaration) {
                            let name = name.to_js_string(self.interner());
                            lex_env.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                        for name in bound_names(declaration) {
                            let name = name.to_js_string(self.interner());
                            lex_env.create_immutable_binding(name, true);
                        }
                    }
                    _ => {}
                }
            }
        }

        // 17. For each Parse Node f of functionsToInitialize, do
        for function in functions_to_initialize {
            // a. Let fn be the sole element of the BoundNames of f.
            let (name, generator, r#async, parameters, body) = match &function {
                VarScopedDeclaration::Function(f) => {
                    (f.name(), false, false, f.parameters(), f.body())
                }
                VarScopedDeclaration::Generator(f) => {
                    (f.name(), true, false, f.parameters(), f.body())
                }
                VarScopedDeclaration::AsyncFunction(f) => {
                    (f.name(), false, true, f.parameters(), f.body())
                }
                VarScopedDeclaration::AsyncGenerator(f) => {
                    (f.name(), true, true, f.parameters(), f.body())
                }
                VarScopedDeclaration::VariableDeclaration(_) => {
                    continue;
                }
            };
            let name = name.expect("function declaration must have a name");
            let code = FunctionCompiler::new()
                .name(name.sym().to_js_string(self.interner()))
                .generator(generator)
                .r#async(r#async)
                .strict(self.strict())
                .in_with(self.in_with)
                .binding_identifier(Some(name.sym().to_js_string(self.interner())))
                .compile(
                    parameters,
                    body,
                    self.variable_environment.clone(),
                    self.lexical_environment.clone(),
                    self.interner,
                );

            // c. If varEnv is a Global Environment Record, then
            if var_env.is_global() {
                // Ensures global functions are printed when generating the global flowgraph.
                let index = self.push_function_to_constants(code.clone());

                // b. Let fo be InstantiateFunctionObject of f with arguments lexEnv and privateEnv.
                let dst = self.register_allocator.alloc();
                self.emit_get_function(&dst, index);
                self.push_from_register(&dst);
                self.register_allocator.dealloc(dst);

                // i. Perform ? varEnv.CreateGlobalFunctionBinding(fn, fo, true).
                let name_index = self.get_or_insert_name(name);
                self.emit(
                    Opcode::CreateGlobalFunctionBinding,
                    &[Operand::Bool(true), Operand::Varying(name_index)],
                );
            }
            // d. Else,
            else {
                // b. Let fo be InstantiateFunctionObject of f with arguments lexEnv and privateEnv.
                let index = self.push_function_to_constants(code);
                let dst = self.register_allocator.alloc();
                self.emit_get_function(&dst, index);
                self.push_from_register(&dst);
                self.register_allocator.dealloc(dst);

                let name = name.to_js_string(self.interner());

                // i. Let bindingExists be ! varEnv.HasBinding(fn).
                let binding_exists = var_env.has_binding(&name);

                // ii. If bindingExists is false, then
                // iii. Else,
                if binding_exists {
                    // 1. Perform ! varEnv.SetMutableBinding(fn, fo, false).
                    let binding = var_env.set_mutable_binding(name).expect("must not fail");
                    let index = self.get_or_insert_binding(binding);
                    self.emit_with_varying_operand(Opcode::SetName, index);
                } else {
                    // 1. NOTE: The following invocation cannot return an abrupt completion because of the validation preceding step 14.
                    // 2. Perform ! varEnv.CreateMutableBinding(fn, true).
                    // 3. Perform ! varEnv.InitializeBinding(fn, fo).
                    let binding = var_env.create_mutable_binding(name, !strict);
                    let index = self.get_or_insert_binding(binding);
                    self.emit_with_varying_operand(Opcode::DefInitVar, index);
                }
            }
        }

        // 18. For each String vn of declaredVarNames, do
        for name in declared_var_names {
            // a. If varEnv is a Global Environment Record, then
            if var_env.is_global() {
                let index = self.get_or_insert_name(name);

                // i. Perform ? varEnv.CreateGlobalVarBinding(vn, true).
                self.emit(
                    Opcode::CreateGlobalVarBinding,
                    &[Operand::Bool(true), Operand::Varying(index)],
                );
            }
            // b. Else,
            else {
                let name = name.to_js_string(self.interner());

                // i. Let bindingExists be ! varEnv.HasBinding(vn).
                let binding_exists = var_env.has_binding(&name);

                // ii. If bindingExists is false, then
                if !binding_exists {
                    // 1. NOTE: The following invocation cannot return an abrupt completion because of the validation preceding step 14.
                    // 2. Perform ! varEnv.CreateMutableBinding(vn, true).
                    // 3. Perform ! varEnv.InitializeBinding(vn, undefined).
                    let binding = var_env.create_mutable_binding(name, true);
                    let index = self.get_or_insert_binding(binding);
                    self.emit_opcode(Opcode::PushUndefined);
                    self.emit_with_varying_operand(Opcode::DefInitVar, index);
                }
            }
        }

        // 19. Return unused.
    }

    /// `FunctionDeclarationInstantiation ( func, argumentsList )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
    pub(crate) fn function_declaration_instantiation(
        &mut self,
        body: &FunctionBody,
        formals: &FormalParameterList,
        arrow: bool,
        strict: bool,
        generator: bool,
    ) {
        // 1. Let calleeContext be the running execution context.
        // 2. Let code be func.[[ECMAScriptCode]].
        // 3. Let strict be func.[[Strict]].
        // 4. Let formals be func.[[FormalParameters]].

        // 5. Let parameterNames be the BoundNames of formals.
        let mut parameter_names = bound_names(formals);

        // 6. If parameterNames has any duplicate entries, let hasDuplicates be true. Otherwise, let hasDuplicates be false.
        // let has_duplicates = formals.has_duplicates();

        // 7. Let simpleParameterList be IsSimpleParameterList of formals.
        // let simple_parameter_list = formals.is_simple();

        // 8. Let hasParameterExpressions be ContainsExpression of formals.
        let has_parameter_expressions = formals.has_expressions();

        // 9. Let varNames be the VarDeclaredNames of code.
        let var_names = var_declared_names(body);

        // 10. Let varDeclarations be the VarScopedDeclarations of code.
        let var_declarations = var_scoped_declarations(body);

        // 11. Let lexicalNames be the LexicallyDeclaredNames of code.
        let lexical_names = lexically_declared_names(body);

        // 12. Let functionNames be a new empty List.
        let mut function_names = Vec::new();

        // 13. Let functionsToInitialize be a new empty List.
        let mut functions_to_initialize = Vec::new();

        // 14. For each element d of varDeclarations, in reverse List order, do
        for declaration in var_declarations.iter().rev() {
            // a. If d is neither a VariableDeclaration nor a ForBinding nor a BindingIdentifier, then
            // a.i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
            let function = match declaration {
                VarScopedDeclaration::Function(f) => FunctionSpec::from(f),
                VarScopedDeclaration::Generator(f) => FunctionSpec::from(f),
                VarScopedDeclaration::AsyncFunction(f) => FunctionSpec::from(f),
                VarScopedDeclaration::AsyncGenerator(f) => FunctionSpec::from(f),
                VarScopedDeclaration::VariableDeclaration(_) => continue,
            };

            // a.ii. Let fn be the sole element of the BoundNames of d.
            let name = function
                .name
                .expect("function declaration must have a name");

            // a.iii. If functionNames does not contain fn, then
            if !function_names.contains(&name) {
                // 1. Insert fn as the first element of functionNames.
                function_names.push(name);

                // 2. NOTE: If there are multiple function declarations for the same name, the last declaration is used.
                // 3. Insert d as the first element of functionsToInitialize.
                functions_to_initialize.push(function);
            }
        }

        function_names.reverse();
        functions_to_initialize.reverse();

        // 15. Let argumentsObjectNeeded be true.
        let mut arguments_object_needed = true;

        let arguments = Sym::ARGUMENTS.into();

        // 16. If func.[[ThisMode]] is lexical, then
        // 17. Else if parameterNames contains "arguments", then
        if arrow || parameter_names.contains(&arguments) {
            // 16.a. NOTE: Arrow functions never have an arguments object.
            // 16.b. Set argumentsObjectNeeded to false.
            // 17.a. Set argumentsObjectNeeded to false.
            arguments_object_needed = false;
        }
        // 18. Else if hasParameterExpressions is false, then
        else if !has_parameter_expressions {
            //a. If functionNames contains "arguments" or lexicalNames contains "arguments", then
            if function_names.contains(&arguments) || lexical_names.contains(&arguments) {
                // i. Set argumentsObjectNeeded to false.
                arguments_object_needed = false;
            }
        }

        // 19. If strict is true or hasParameterExpressions is false, then
        if strict || !has_parameter_expressions {
            // a. NOTE: Only a single Environment Record is needed for the parameters,
            //    since calls to eval in strict mode code cannot create new bindings which are visible outside of the eval.
            // b. Let env be the LexicalEnvironment of calleeContext.
        }
        // 20. Else,
        else {
            // a. NOTE: A separate Environment Record is needed to ensure that bindings created by
            //    direct eval calls in the formal parameter list are outside the environment where parameters are declared.
            // b. Let calleeEnv be the LexicalEnvironment of calleeContext.
            // c. Let env be NewDeclarativeEnvironment(calleeEnv).
            // d. Assert: The VariableEnvironment of calleeContext is calleeEnv.
            // e. Set the LexicalEnvironment of calleeContext to env.
            let env_index = self.push_compile_environment(false);
            self.emit_with_varying_operand(Opcode::PushDeclarativeEnvironment, env_index);
        };

        let env = self.lexical_environment.clone();

        // 22. If argumentsObjectNeeded is true, then
        //
        // NOTE(HalidOdat): Has been moved up, so "arguments" gets registed as
        //     the first binding in the environment with index 0.
        if arguments_object_needed {
            let arguments = arguments.to_js_string(self.interner());

            // a. If strict is true or simpleParameterList is false, then
            if strict || !formals.is_simple() {
                // i. Let ao be CreateUnmappedArgumentsObject(argumentsList).
                self.emit_opcode(Opcode::CreateUnmappedArgumentsObject);
            }
            // b. Else,
            else {
                // i. NOTE: A mapped argument object is only provided for non-strict functions
                //          that don't have a rest parameter, any parameter
                //          default value initializers, or any destructured parameters.
                // ii. Let ao be CreateMappedArgumentsObject(func, formals, argumentsList, env).
                self.emit_opcode(Opcode::CreateMappedArgumentsObject);
                self.emitted_mapped_arguments_object_opcode = true;
            }

            // c. If strict is true, then
            if strict {
                // i. Perform ! env.CreateImmutableBinding("arguments", false).
                // ii. NOTE: In strict mode code early errors prevent attempting to assign
                //           to this binding, so its mutability is not observable.
                env.create_immutable_binding(arguments.clone(), false);
            }
            // d. Else,
            else {
                // i. Perform ! env.CreateMutableBinding("arguments", false).
                env.create_mutable_binding(arguments.clone(), false);
            }

            // e. Perform ! env.InitializeBinding("arguments", ao).
            self.emit_binding(BindingOpcode::InitLexical, arguments);
        }

        // 21. For each String paramName of parameterNames, do
        for param_name in &parameter_names {
            let param_name = param_name.to_js_string(self.interner());

            // a. Let alreadyDeclared be ! env.HasBinding(paramName).
            let already_declared = env.has_binding(&param_name);

            // b. NOTE: Early errors ensure that duplicate parameter names can only occur in non-strict
            //    functions that do not have parameter default values or rest parameters.

            // c. If alreadyDeclared is false, then
            if !already_declared {
                // i. Perform ! env.CreateMutableBinding(paramName, false).
                env.create_mutable_binding(param_name, false);

                // Note: These steps are not necessary in our implementation.
                // ii. If hasDuplicates is true, then
                //     1. Perform ! env.InitializeBinding(paramName, undefined).
            }
        }

        // 22. If argumentsObjectNeeded is true, then
        if arguments_object_needed {
            // MOVED: a-e.
            //
            // NOTE(HalidOdat): Has been moved up, see comment above.

            // f. Let parameterBindings be the list-concatenation of parameterNames and « "arguments" ».
            parameter_names.push(arguments);
        }

        // 23. Else,
        //     a. Let parameterBindings be parameterNames.
        let parameter_bindings = parameter_names.clone();

        // 24. Let iteratorRecord be CreateListIteratorRecord(argumentsList).
        // 25. If hasDuplicates is true, then
        //    a. Perform ? IteratorBindingInitialization of formals with arguments iteratorRecord and undefined.
        // 26. Else,
        //    a. Perform ? IteratorBindingInitialization of formals with arguments iteratorRecord and env.
        for (i, parameter) in formals.as_ref().iter().enumerate() {
            if parameter.is_rest_param() {
                self.emit_opcode(Opcode::RestParameterInit);
            } else {
                self.emit_with_varying_operand(Opcode::GetArgument, i as u32);
            }
            match parameter.variable().binding() {
                Binding::Identifier(ident) => {
                    let ident = ident.to_js_string(self.interner());
                    if let Some(init) = parameter.variable().init() {
                        let skip = self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                        self.compile_expr(init, true);
                        self.patch_jump(skip);
                    }
                    self.emit_binding(BindingOpcode::InitLexical, ident);
                }
                Binding::Pattern(pattern) => {
                    if let Some(init) = parameter.variable().init() {
                        let skip = self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                        self.compile_expr(init, true);
                        self.patch_jump(skip);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLexical);
                }
            }
        }

        if generator {
            self.emit(Opcode::Generator, &[Operand::Bool(self.is_async())]);
            self.emit_opcode(Opcode::Pop);
        }

        // 27. If hasParameterExpressions is false, then
        // 28. Else,
        #[allow(unused_variables, unused_mut)]
        let (mut instantiated_var_names, mut var_env) = if has_parameter_expressions {
            // a. NOTE: A separate Environment Record is needed to ensure that closures created by
            //          expressions in the formal parameter list do not have
            //          visibility of declarations in the function body.
            // b. Let varEnv be NewDeclarativeEnvironment(env).
            // c. Set the VariableEnvironment of calleeContext to varEnv.
            let env_index = self.push_compile_environment(false);
            self.emit_with_varying_operand(Opcode::PushDeclarativeEnvironment, env_index);

            let mut var_env = self.lexical_environment.clone();

            // d. Let instantiatedVarNames be a new empty List.
            let mut instantiated_var_names = Vec::new();

            // e. For each element n of varNames, do
            for n in var_names {
                // i. If instantiatedVarNames does not contain n, then
                if !instantiated_var_names.contains(&n) {
                    // 1. Append n to instantiatedVarNames.
                    instantiated_var_names.push(n);

                    let n_string = n.to_js_string(self.interner());

                    // 2. Perform ! varEnv.CreateMutableBinding(n, false).
                    let binding = var_env.create_mutable_binding(n_string.clone(), false);

                    // 3. If parameterBindings does not contain n, or if functionNames contains n, then
                    if !parameter_bindings.contains(&n) || function_names.contains(&n) {
                        // a. Let initialValue be undefined.
                        self.emit_opcode(Opcode::PushUndefined);
                    }
                    // 4. Else,
                    else {
                        // a. Let initialValue be ! env.GetBindingValue(n, false).
                        let binding = env.get_binding(&n_string).expect("must have binding");
                        let index = self.get_or_insert_binding(binding);
                        self.emit_with_varying_operand(Opcode::GetName, index);
                    }

                    // 5. Perform ! varEnv.InitializeBinding(n, initialValue).
                    let index = self.get_or_insert_binding(binding);
                    self.emit_opcode(Opcode::PushUndefined);
                    self.emit_with_varying_operand(Opcode::DefInitVar, index);

                    // 6. NOTE: A var with the same name as a formal parameter initially has
                    //          the same value as the corresponding initialized parameter.
                }
            }

            (instantiated_var_names, var_env)
        } else {
            // a. NOTE: Only a single Environment Record is needed for the parameters and top-level vars.
            // b. Let instantiatedVarNames be a copy of the List parameterBindings.
            let mut instantiated_var_names = parameter_bindings;

            // c. For each element n of varNames, do
            for n in var_names {
                // i. If instantiatedVarNames does not contain n, then
                if !instantiated_var_names.contains(&n) {
                    // 1. Append n to instantiatedVarNames.
                    instantiated_var_names.push(n);

                    let n = n.to_js_string(self.interner());

                    // 2. Perform ! env.CreateMutableBinding(n, false).
                    // 3. Perform ! env.InitializeBinding(n, undefined).
                    let binding = env.create_mutable_binding(n, true);
                    let index = self.get_or_insert_binding(binding);
                    self.emit_opcode(Opcode::PushUndefined);
                    self.emit_with_varying_operand(Opcode::DefInitVar, index);
                }
            }

            // d. Let varEnv be env.
            (instantiated_var_names, env)
        };

        // 29. NOTE: Annex B.3.2.1 adds additional steps at this point.
        // 29. If strict is false, then
        #[cfg(feature = "annex-b")]
        if !strict {
            // a. For each FunctionDeclaration f that is directly contained in the StatementList
            //    of a Block, CaseClause, or DefaultClause, do
            for f in annex_b_function_declarations_names(body) {
                // i. Let F be StringValue of the BindingIdentifier of f.
                // ii. If replacing the FunctionDeclaration f with a VariableStatement that has F
                //     as a BindingIdentifier would not produce any Early Errors
                //     for func and parameterNames does not contain F, then
                if !lexical_names.contains(&f) && !parameter_names.contains(&f) {
                    // 1. NOTE: A var binding for F is only instantiated here if it is neither a
                    //    VarDeclaredName, the name of a formal parameter, or another FunctionDeclaration.

                    // 2. If initializedBindings does not contain F and F is not "arguments", then
                    if !instantiated_var_names.contains(&f) && f != arguments {
                        let f_string = f.to_js_string(self.interner());

                        // a. Perform ! varEnv.CreateMutableBinding(F, false).
                        // b. Perform ! varEnv.InitializeBinding(F, undefined).
                        let binding = var_env.create_mutable_binding(f_string, false);
                        let index = self.get_or_insert_binding(binding);
                        self.emit_opcode(Opcode::PushUndefined);
                        self.emit_with_varying_operand(Opcode::DefInitVar, index);

                        // c. Append F to instantiatedVarNames.
                        instantiated_var_names.push(f);
                    }

                    // 3. When the FunctionDeclaration f is evaluated, perform the following steps
                    //    in place of the FunctionDeclaration Evaluation algorithm provided in 15.2.6:
                    //     a. Let fenv be the running execution context's VariableEnvironment.
                    //     b. Let benv be the running execution context's LexicalEnvironment.
                    //     c. Let fobj be ! benv.GetBindingValue(F, false).
                    //     d. Perform ! fenv.SetMutableBinding(F, fobj, false).
                    //     e. Return unused.
                    self.annex_b_function_names.push(f);
                }
            }
        }

        // 30. If strict is false, then
        // 31. Else,
        let lex_env = if strict {
            // a. Let lexEnv be varEnv.
            var_env
        } else {
            // a. Let lexEnv be NewDeclarativeEnvironment(varEnv).
            // b. NOTE: Non-strict functions use a separate Environment Record for top-level lexical
            //    declarations so that a direct eval can determine whether any var scoped declarations
            //    introduced by the eval code conflict with pre-existing top-level lexically scoped declarations.
            //    This is not needed for strict functions because a strict direct eval always
            //    places all declarations into a new Environment Record.
            let env_index = self.push_compile_environment(false);
            self.emit_with_varying_operand(Opcode::PushDeclarativeEnvironment, env_index);
            self.lexical_environment.clone()
        };

        // 32. Set the LexicalEnvironment of calleeContext to lexEnv.

        // 33. Let lexDeclarations be the LexicallyScopedDeclarations of code.
        // 34. For each element d of lexDeclarations, do
        //     a. NOTE: A lexically declared name cannot be the same as a function/generator declaration,
        //        formal parameter, or a var name. Lexically declared names are only instantiated here but not initialized.
        //     b. For each element dn of the BoundNames of d, do
        //         i. If IsConstantDeclaration of d is true, then
        //             1. Perform ! lexEnv.CreateImmutableBinding(dn, true).
        //         ii. Else,
        //             1. Perform ! lexEnv.CreateMutableBinding(dn, false).
        for statement in &**body.statements() {
            if let StatementListItem::Declaration(declaration) = statement {
                match declaration {
                    Declaration::Class(class) => {
                        for name in bound_names(class) {
                            let name = name.to_js_string(self.interner());
                            lex_env.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                        for name in bound_names(declaration) {
                            let name = name.to_js_string(self.interner());
                            lex_env.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                        for name in bound_names(declaration) {
                            let name = name.to_js_string(self.interner());
                            lex_env.create_immutable_binding(name, true);
                        }
                    }
                    _ => {}
                }
            }
        }

        // 35. Let privateEnv be the PrivateEnvironment of calleeContext.
        // 36. For each Parse Node f of functionsToInitialize, do
        for function in functions_to_initialize {
            // a. Let fn be the sole element of the BoundNames of f.
            // b. Let fo be InstantiateFunctionObject of f with arguments lexEnv and privateEnv.
            // c. Perform ! varEnv.SetMutableBinding(fn, fo, false).
            self.function_with_binding(function, NodeKind::Declaration, false);
        }

        // 37. Return unused.
    }
}
