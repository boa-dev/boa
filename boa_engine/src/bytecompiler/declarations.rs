use crate::{
    bytecompiler::{ByteCompiler, FunctionCompiler, FunctionSpec, Label, NodeKind},
    environments::BindingLocatorError,
    vm::{
        create_function_object_fast, create_generator_function_object, BindingOpcode,
        CodeBlockFlags, Opcode,
    },
    JsNativeError, JsResult,
};
use boa_ast::{
    declaration::{Binding, LexicalDeclaration, VariableList},
    function::{FormalParameterList, FunctionBody},
    operations::{
        all_private_identifiers_valid, bound_names, lexically_declared_names,
        lexically_scoped_declarations, var_declared_names, var_scoped_declarations,
        LexicallyScopedDeclaration, VarScopedDeclaration,
    },
    visitor::NodeRef,
    Declaration, Script, StatementListItem,
};
use boa_interner::Sym;

#[cfg(feature = "annex-b")]
use boa_ast::operations::annex_b_function_declarations_names;

impl ByteCompiler<'_, '_> {
    /// `GlobalDeclarationInstantiation ( script, env )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-globaldeclarationinstantiation
    pub(crate) fn global_declaration_instantiation(&mut self, script: &Script) -> JsResult<()> {
        // 1. Let lexNames be the LexicallyDeclaredNames of script.
        let lex_names = lexically_declared_names(script);

        // 2. Let varNames be the VarDeclaredNames of script.
        let var_names = var_declared_names(script);

        // 3. For each element name of lexNames, do
        for name in lex_names {
            // Note: Our implementation differs from the spec here.
            // a. If env.HasVarDeclaration(name) is true, throw a SyntaxError exception.

            // b. If env.HasLexicalDeclaration(name) is true, throw a SyntaxError exception.
            if self.has_binding(name) {
                return Err(JsNativeError::syntax()
                    .with_message("duplicate lexical declaration")
                    .into());
            }

            // c. Let hasRestrictedGlobal be ? env.HasRestrictedGlobalProperty(name).
            let has_restricted_global = self.context.has_restricted_global_property(name)?;

            // d. If hasRestrictedGlobal is true, throw a SyntaxError exception.
            if has_restricted_global {
                return Err(JsNativeError::syntax()
                    .with_message("cannot redefine non-configurable global property")
                    .into());
            }
        }

        // 4. For each element name of varNames, do
        for name in var_names {
            // a. If env.HasLexicalDeclaration(name) is true, throw a SyntaxError exception.
            if self.has_binding(name) {
                return Err(JsNativeError::syntax()
                    .with_message("duplicate lexical declaration")
                    .into());
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
                let fn_definable = self.context.can_declare_global_function(name)?;

                // 2. If fnDefinable is false, throw a TypeError exception.
                if !fn_definable {
                    return Err(JsNativeError::typ()
                        .with_message("cannot declare global function")
                        .into());
                }

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
                    let definable = self.context.can_declare_global_var(name)?;

                    // b. If vnDefinable is false, throw a TypeError exception.
                    if !definable {
                        return Err(JsNativeError::typ()
                            .with_message("cannot declare global variable")
                            .into());
                    }

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
        #[cfg(feature = "annex-b")]
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
                    // a. If env.HasLexicalDeclaration(F) is false, then
                    if !self.current_environment.has_lex_binding(f) {
                        // i. Let fnDefinable be ? env.CanDeclareGlobalVar(F).
                        let fn_definable = self.context.can_declare_global_function(f)?;

                        // ii. If fnDefinable is true, then
                        if fn_definable {
                            // i. NOTE: A var binding for F is only instantiated here if it is neither
                            //          a VarDeclaredName nor the name of another FunctionDeclaration.
                            // ii. If declaredFunctionOrVarNames does not contain F, then
                            if !declared_function_names.contains(&f)
                                && !declared_var_names.contains(&f)
                            {
                                // i. Perform ? env.CreateGlobalVarBinding(F, false).
                                self.context.create_global_var_binding(f, false)?;

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
                            self.annex_b_function_names.push(f);
                        }
                    }
                }
            }
        }

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
                            self.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                        for name in bound_names(declaration) {
                            self.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                        for name in bound_names(declaration) {
                            self.create_immutable_binding(name, true);
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
                .name(name.sym())
                .generator(generator)
                .r#async(r#async)
                .strict(self.strict())
                .binding_identifier(Some(name.sym()))
                .compile(
                    parameters,
                    body,
                    self.current_environment.clone(),
                    self.context,
                );

            // Ensures global functions are printed when generating the global flowgraph.
            self.functions.push(code.clone());

            // b. Let fo be InstantiateFunctionObject of f with arguments env and privateEnv.
            let function = if generator {
                create_generator_function_object(code, r#async, None, self.context)
            } else {
                create_function_object_fast(code, r#async, false, false, self.context)
            };

            // c. Perform ? env.CreateGlobalFunctionBinding(fn, fo, false).
            self.context
                .create_global_function_binding(name, function, false)?;
        }

        // 17. For each String vn of declaredVarNames, do
        for var in declared_var_names {
            // a. Perform ? env.CreateGlobalVarBinding(vn, false).
            self.context.create_global_var_binding(var, false)?;
        }

        // 18. Return unused.
        Ok(())
    }

    /// `BlockDeclarationInstantiation ( code, env )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-blockdeclarationinstantiation
    pub(crate) fn block_declaration_instantiation<'a, N>(&mut self, block: &'a N)
    where
        &'a N: Into<NodeRef<'a>>,
    {
        let env = &self.current_environment;

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
                    env.create_immutable_binding(dn, true);
                }
            }
            // ii. Else,
            else {
                // a. For each element dn of the BoundNames of d, do
                for dn in d.bound_names() {
                    #[cfg(not(feature = "annex-b"))]
                    // 1. Perform ! env.CreateMutableBinding(dn, false). NOTE: This step is replaced in section B.3.2.6.
                    env.create_mutable_binding(dn, false);

                    #[cfg(feature = "annex-b")]
                    // 1. If ! env.HasBinding(dn) is false, then
                    if !env.has_binding(dn) {
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
    ) -> JsResult<()> {
        let var_environment_is_global = self
            .context
            .vm
            .environments
            .is_next_outer_function_environment_global()
            && !strict;

        // 2. Let varDeclarations be the VarScopedDeclarations of body.
        let var_declarations = var_scoped_declarations(body);

        // 3. If strict is false, then
        if !strict {
            // 1. Let varNames be the VarDeclaredNames of body.
            let var_names = var_declared_names(body);

            // a. If varEnv is a Global Environment Record, then
            //      i. For each element name of varNames, do
            //          1. If varEnv.HasLexicalDeclaration(name) is true, throw a SyntaxError exception.
            //          2. NOTE: eval will not create a global var declaration that would be shadowed by a global lexical declaration.
            // b. Let thisEnv be lexEnv.
            // c. Assert: The following loop will terminate.
            // d. Repeat, while thisEnv is not varEnv,
            //     i. If thisEnv is not an Object Environment Record, then
            //         1. NOTE: The environment of with statements cannot contain any lexical
            //            declaration so it doesn't need to be checked for var/let hoisting conflicts.
            //         2. For each element name of varNames, do
            //             a. If ! thisEnv.HasBinding(name) is true, then
            //                 i. Throw a SyntaxError exception.
            //                 ii. NOTE: Annex B.3.4 defines alternate semantics for the above step.
            //             b. NOTE: A direct eval will not hoist var declaration over a like-named lexical declaration.
            //     ii. Set thisEnv to thisEnv.[[OuterEnv]].
            if let Some(name) = self
                .context
                .vm
                .environments
                .has_lex_binding_until_function_environment(&var_names)
            {
                let name = self.context.interner().resolve_expect(name.sym());
                let msg = format!("variable declaration {name} in eval function already exists as a lexical variable");
                return Err(JsNativeError::syntax().with_message(msg).into());
            }
        }

        // 4. Let privateIdentifiers be a new empty List.
        // 5. Let pointer be privateEnv.
        // 6. Repeat, while pointer is not null,
        //     a. For each Private Name binding of pointer.[[Names]], do
        //         i. If privateIdentifiers does not contain binding.[[Description]],
        //            append binding.[[Description]] to privateIdentifiers.
        //     b. Set pointer to pointer.[[OuterPrivateEnvironment]].
        let private_identifiers = self.context.vm.environments.private_name_descriptions();
        let private_identifiers = private_identifiers
            .into_iter()
            .map(|ident| {
                self.context
                    .interner()
                    .get(ident.as_slice())
                    .expect("string should be in interner")
            })
            .collect();

        // 7. If AllPrivateIdentifiersValid of body with argument privateIdentifiers is false, throw a SyntaxError exception.
        if !all_private_identifiers_valid(body, private_identifiers) {
            return Err(JsNativeError::syntax()
                .with_message("invalid private identifier")
                .into());
        }

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
                if var_environment_is_global {
                    // a. Let fnDefinable be ? varEnv.CanDeclareGlobalFunction(fn).
                    let fn_definable = self.context.can_declare_global_function(name)?;

                    // b. If fnDefinable is false, throw a TypeError exception.
                    if !fn_definable {
                        return Err(JsNativeError::typ()
                            .with_message("cannot declare global function")
                            .into());
                    }
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
                    // 2. Let thisEnv be lexEnv.
                    // 3. Assert: The following loop will terminate.
                    // 4. Repeat, while thisEnv is not varEnv,
                    // a. If thisEnv is not an Object Environment Record, then
                    // i. If ! thisEnv.HasBinding(F) is true, then
                    // i. Let bindingExists be true.
                    // b. Set thisEnv to thisEnv.[[OuterEnv]].
                    let binding_exists = self.has_binding_until_var(f);

                    // 5. If bindingExists is false and varEnv is a Global Environment Record, then
                    let fn_definable = if !binding_exists && var_environment_is_global {
                        // a. If varEnv.HasLexicalDeclaration(F) is false, then
                        // b. Else,
                        if self.current_environment.has_lex_binding(f) {
                            // i. Let fnDefinable be false.
                            false
                        } else {
                            // i. Let fnDefinable be ? varEnv.CanDeclareGlobalVar(F).
                            self.context.can_declare_global_var(f)?
                        }
                    }
                    // 6. Else,
                    else {
                        // a. Let fnDefinable be true.
                        true
                    };

                    // 7. If bindingExists is false and fnDefinable is true, then
                    if !binding_exists && fn_definable {
                        let mut function_names = Vec::new();

                        // a. If declaredFunctionOrVarNames does not contain F, then
                        if !declared_function_names.contains(&f)
                            //&& !var_names.contains(&f)
                            && !function_names.contains(&f)
                        {
                            // i. If varEnv is a Global Environment Record, then
                            if var_environment_is_global {
                                // i. Perform ? varEnv.CreateGlobalVarBinding(F, true).
                                self.context.create_global_var_binding(f, true)?;
                            }
                            // ii. Else,
                            else {
                                // i. Let bindingExists be ! varEnv.HasBinding(F).
                                // ii. If bindingExists is false, then
                                if !self.has_binding(f) {
                                    // i. Perform ! varEnv.CreateMutableBinding(F, true).
                                    self.create_mutable_binding(f, true);

                                    // ii. Perform ! varEnv.InitializeBinding(F, undefined).
                                    let binding = self.initialize_mutable_binding(f, true);
                                    let index = self.get_or_insert_binding(binding);
                                    self.emit_opcode(Opcode::PushUndefined);
                                    self.emit(Opcode::DefInitVar, &[index]);
                                }
                            }

                            // iii. Append F to declaredFunctionOrVarNames.
                            function_names.push(f);
                        }

                        // b. When the FunctionDeclaration f is evaluated, perform the following steps
                        //    in place of the FunctionDeclaration Evaluation algorithm provided in 15.2.6:
                        //     i. Let genv be the running execution context's VariableEnvironment.
                        //     ii. Let benv be the running execution context's LexicalEnvironment.
                        //     iii. Let fobj be ! benv.GetBindingValue(F, false).
                        //     iv. Perform ? genv.SetMutableBinding(F, fobj, false).
                        //     v. Return unused.
                        self.annex_b_function_names.push(f);
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
                    if var_environment_is_global {
                        // i. Let vnDefinable be ? varEnv.CanDeclareGlobalVar(vn).
                        let vn_definable = self.context.can_declare_global_var(name)?;

                        // ii. If vnDefinable is false, throw a TypeError exception.
                        if !vn_definable {
                            return Err(JsNativeError::typ()
                                .with_message("cannot declare global variable")
                                .into());
                        }
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
                            self.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                        for name in bound_names(declaration) {
                            self.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                        for name in bound_names(declaration) {
                            self.create_immutable_binding(name, true);
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
                .name(name.sym())
                .generator(generator)
                .r#async(r#async)
                .strict(self.strict())
                .binding_identifier(Some(name.sym()))
                .compile(
                    parameters,
                    body,
                    self.context.vm.environments.current_compile_environment(),
                    self.context,
                );

            // c. If varEnv is a Global Environment Record, then
            if var_environment_is_global {
                // Ensures global functions are printed when generating the global flowgraph.
                self.functions.push(code.clone());

                // b. Let fo be InstantiateFunctionObject of f with arguments lexEnv and privateEnv.
                let function = if generator {
                    create_generator_function_object(code, r#async, None, self.context)
                } else {
                    create_function_object_fast(code, r#async, false, false, self.context)
                };

                // i. Perform ? varEnv.CreateGlobalFunctionBinding(fn, fo, true).
                self.context
                    .create_global_function_binding(name, function, true)?;
            }
            // d. Else,
            else {
                // b. Let fo be InstantiateFunctionObject of f with arguments lexEnv and privateEnv.
                let index = self.functions.len() as u32;
                self.functions.push(code);
                if r#async && generator {
                    self.emit(Opcode::GetGeneratorAsync, &[index]);
                } else if generator {
                    self.emit(Opcode::GetGenerator, &[index]);
                } else if r#async {
                    self.emit(Opcode::GetFunctionAsync, &[index]);
                } else {
                    self.emit(Opcode::GetFunction, &[index]);
                }
                if !generator {
                    self.emit_u8(0);
                }

                // i. Let bindingExists be ! varEnv.HasBinding(fn).
                let binding_exists = self.has_binding_eval(name, strict);

                // ii. If bindingExists is false, then
                // iii. Else,
                if binding_exists {
                    // 1. Perform ! varEnv.SetMutableBinding(fn, fo, false).
                    match self.set_mutable_binding(name) {
                        Ok(binding) => {
                            let index = self.get_or_insert_binding(binding);
                            self.emit(Opcode::SetName, &[index]);
                        }
                        Err(BindingLocatorError::MutateImmutable) => {
                            let index = self.get_or_insert_name(name);
                            self.emit(Opcode::ThrowMutateImmutable, &[index]);
                        }
                        Err(BindingLocatorError::Silent) => {
                            self.emit(Opcode::Pop, &[]);
                        }
                    }
                } else {
                    // 1. NOTE: The following invocation cannot return an abrupt completion because of the validation preceding step 14.
                    // 2. Perform ! varEnv.CreateMutableBinding(fn, true).
                    // 3. Perform ! varEnv.InitializeBinding(fn, fo).
                    self.create_mutable_binding(name, !strict);
                    let binding = self.initialize_mutable_binding(name, !strict);
                    let index = self.get_or_insert_binding(binding);
                    self.emit(Opcode::DefInitVar, &[index]);
                }
            }
        }

        // 18. For each String vn of declaredVarNames, do
        for name in declared_var_names {
            // a. If varEnv is a Global Environment Record, then
            if var_environment_is_global {
                // i. Perform ? varEnv.CreateGlobalVarBinding(vn, true).
                self.context.create_global_var_binding(name, true)?;
            }
            // b. Else,
            else {
                // i. Let bindingExists be ! varEnv.HasBinding(vn).
                let binding_exists = self.has_binding_eval(name, strict);

                // ii. If bindingExists is false, then
                if !binding_exists {
                    // 1. NOTE: The following invocation cannot return an abrupt completion because of the validation preceding step 14.
                    // 2. Perform ! varEnv.CreateMutableBinding(vn, true).
                    // 3. Perform ! varEnv.InitializeBinding(vn, undefined).
                    self.create_mutable_binding(name, !strict);
                    let binding = self.initialize_mutable_binding(name, !strict);
                    let index = self.get_or_insert_binding(binding);
                    self.emit_opcode(Opcode::PushUndefined);
                    self.emit(Opcode::DefInitVar, &[index]);
                }
            }
        }

        // 19. Return unused.
        Ok(())
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
    ) -> (Option<Label>, bool) {
        let mut env_label = None;
        let mut additional_env = false;

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
        //     a. NOTE: Only a single Environment Record is needed for the parameters,
        //        since calls to eval in strict mode code cannot create new bindings which are visible outside of the eval.
        //     b. Let env be the LexicalEnvironment of calleeContext.
        // 20. Else,
        if !strict && has_parameter_expressions {
            // a. NOTE: A separate Environment Record is needed to ensure that bindings created by
            //    direct eval calls in the formal parameter list are outside the environment where parameters are declared.
            // b. Let calleeEnv be the LexicalEnvironment of calleeContext.
            // c. Let env be NewDeclarativeEnvironment(calleeEnv).
            // d. Assert: The VariableEnvironment of calleeContext is calleeEnv.
            // e. Set the LexicalEnvironment of calleeContext to env.
            self.push_compile_environment(false);
            additional_env = true;
        }

        // 22. If argumentsObjectNeeded is true, then
        //
        // NOTE(HalidOdat): Has been moved up, so "arguments" gets registed as
        //     the first binding in the environment with index 0.
        if arguments_object_needed {
            // Note: This happens at runtime.
            // a. If strict is true or simpleParameterList is false, then
            //     i. Let ao be CreateUnmappedArgumentsObject(argumentsList).
            // b. Else,
            //     i. NOTE: A mapped argument object is only provided for non-strict functions
            //              that don't have a rest parameter, any parameter
            //              default value initializers, or any destructured parameters.
            //     ii. Let ao be CreateMappedArgumentsObject(func, formals, argumentsList, env).

            // c. If strict is true, then
            if strict {
                // i. Perform ! env.CreateImmutableBinding("arguments", false).
                // ii. NOTE: In strict mode code early errors prevent attempting to assign
                //           to this binding, so its mutability is not observable.
                self.create_immutable_binding(arguments, false);
            }
            // d. Else,
            else {
                // i. Perform ! env.CreateMutableBinding("arguments", false).
                self.create_mutable_binding(arguments, false);
            }

            self.code_block_flags |= CodeBlockFlags::NEEDS_ARGUMENTS_OBJECT;
        }

        // 21. For each String paramName of parameterNames, do
        for param_name in &parameter_names {
            // a. Let alreadyDeclared be ! env.HasBinding(paramName).
            let already_declared = self.has_binding(*param_name);

            // b. NOTE: Early errors ensure that duplicate parameter names can only occur in non-strict
            //    functions that do not have parameter default values or rest parameters.

            // c. If alreadyDeclared is false, then
            if !already_declared {
                // i. Perform ! env.CreateMutableBinding(paramName, false).
                self.create_mutable_binding(*param_name, false);

                // Note: These steps are not necessary in our implementation.
                // ii. If hasDuplicates is true, then
                //     1. Perform ! env.InitializeBinding(paramName, undefined).
            }
        }

        // 22. If argumentsObjectNeeded is true, then
        if arguments_object_needed {
            // MOVED: a-d.
            //
            // NOTE(HalidOdat): Has been moved up, see comment above.

            // Note: This happens at runtime.
            // e. Perform ! env.InitializeBinding("arguments", ao).

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
        for parameter in formals.as_ref() {
            if parameter.is_rest_param() {
                self.emit_opcode(Opcode::RestParameterInit);
            }
            match parameter.variable().binding() {
                Binding::Identifier(ident) => {
                    self.create_mutable_binding(*ident, false);
                    if let Some(init) = parameter.variable().init() {
                        let skip = self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                        self.compile_expr(init, true);
                        self.patch_jump(skip);
                    }
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_mutable_binding(ident, false);
                    }
                    if let Some(init) = parameter.variable().init() {
                        let skip = self.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                        self.compile_expr(init, true);
                        self.patch_jump(skip);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLet);
                }
            }
        }
        if !formals.has_rest_parameter() {
            self.emit_opcode(Opcode::RestParameterPop);
        }
        if generator {
            self.emit_opcode(Opcode::PushUndefined);
            // Don't need to use `AsyncGeneratorYield` since
            // we just want to stop the execution of the generator.
            self.emit_opcode(Opcode::GeneratorYield);
            self.emit_opcode(Opcode::Pop);
        }

        // 27. If hasParameterExpressions is false, then
        // 28. Else,
        #[allow(unused_variables, unused_mut)]
        let mut instantiated_var_names = if has_parameter_expressions {
            // a. NOTE: A separate Environment Record is needed to ensure that closures created by
            //          expressions in the formal parameter list do not have
            //          visibility of declarations in the function body.
            // b. Let varEnv be NewDeclarativeEnvironment(env).
            // c. Set the VariableEnvironment of calleeContext to varEnv.
            self.push_compile_environment(true);
            env_label = Some(self.emit_opcode_with_operand(Opcode::PushFunctionEnvironment));

            // d. Let instantiatedVarNames be a new empty List.
            let mut instantiated_var_names = Vec::new();

            // e. For each element n of varNames, do
            for n in var_names {
                // i. If instantiatedVarNames does not contain n, then
                if !instantiated_var_names.contains(&n) {
                    // 1. Append n to instantiatedVarNames.
                    instantiated_var_names.push(n);

                    // 2. Perform ! varEnv.CreateMutableBinding(n, false).
                    self.create_mutable_binding(n, true);

                    // 3. If parameterBindings does not contain n, or if functionNames contains n, then
                    if !parameter_bindings.contains(&n) || function_names.contains(&n) {
                        // a. Let initialValue be undefined.
                        self.emit_opcode(Opcode::PushUndefined);
                    }
                    // 4. Else,
                    else {
                        // a. Let initialValue be ! env.GetBindingValue(n, false).
                        let binding = self.get_binding_value(n);
                        let index = self.get_or_insert_binding(binding);
                        self.emit(Opcode::GetName, &[index]);
                    }

                    // 5. Perform ! varEnv.InitializeBinding(n, initialValue).
                    let binding = self.initialize_mutable_binding(n, true);
                    let index = self.get_or_insert_binding(binding);
                    self.emit_opcode(Opcode::PushUndefined);
                    self.emit(Opcode::DefInitVar, &[index]);

                    // 6. NOTE: A var with the same name as a formal parameter initially has
                    //          the same value as the corresponding initialized parameter.
                }
            }

            instantiated_var_names
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

                    // 2. Perform ! env.CreateMutableBinding(n, false).
                    self.create_mutable_binding(n, true);

                    // 3. Perform ! env.InitializeBinding(n, undefined).
                    let binding = self.initialize_mutable_binding(n, true);
                    let index = self.get_or_insert_binding(binding);
                    self.emit_opcode(Opcode::PushUndefined);
                    self.emit(Opcode::DefInitVar, &[index]);
                }
            }

            // d. Let varEnv be env.
            instantiated_var_names
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
                        // a. Perform ! varEnv.CreateMutableBinding(F, false).
                        self.create_mutable_binding(f, true);

                        // b. Perform ! varEnv.InitializeBinding(F, undefined).
                        let binding = self.initialize_mutable_binding(f, true);
                        let index = self.get_or_insert_binding(binding);
                        self.emit_opcode(Opcode::PushUndefined);
                        self.emit(Opcode::DefInitVar, &[index]);

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
        // 30.a. Let lexEnv be NewDeclarativeEnvironment(varEnv).
        // 30.b. NOTE: Non-strict functions use a separate Environment Record for top-level lexical
        //      declarations so that a direct eval can determine whether any var scoped declarations
        //      introduced by the eval code conflict with pre-existing top-level lexically scoped declarations.
        //      This is not needed for strict functions because a strict direct eval always
        //      places all declarations into a new Environment Record.
        // 31. Else,
        //     a. Let lexEnv be varEnv.
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
                            self.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                        for name in bound_names(declaration) {
                            self.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                        for name in bound_names(declaration) {
                            self.create_immutable_binding(name, true);
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
        (env_label, additional_env)
    }
}
