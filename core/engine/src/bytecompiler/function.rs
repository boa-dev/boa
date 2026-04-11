use crate::{
    JsString, SpannedSourceText,
    builtins::function::ThisMode,
    bytecompiler::ByteCompiler,
    js_string,
    vm::{CodeBlock, CodeBlockFlags, source_info::SourcePath},
};
use boa_ast::{
    function::{FormalParameterList, FunctionBody},
    scope::{FunctionScopes, Scope},
};
use boa_gc::Gc;
use boa_interner::Interner;

/// `FunctionCompiler` is used to compile AST functions to bytecode.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct FunctionCompiler {
    name: JsString,
    generator: bool,
    r#async: bool,
    strict: bool,
    arrow: bool,
    method: bool,
    in_with: bool,
    force_function_scope: bool,
    name_scope: Option<Scope>,
    spanned_source_text: SpannedSourceText,
    source_path: SourcePath,
}

impl FunctionCompiler {
    /// Create a new `FunctionCompiler`.
    pub(crate) fn new(spanned_source_text: SpannedSourceText) -> Self {
        Self {
            name: js_string!(),
            generator: false,
            r#async: false,
            strict: false,
            arrow: false,
            method: false,
            in_with: false,
            force_function_scope: false,
            name_scope: None,
            spanned_source_text,
            source_path: SourcePath::None,
        }
    }

    /// Set the name of the function.
    pub(crate) fn name<N>(mut self, name: N) -> Self
    where
        N: Into<Option<JsString>>,
    {
        let name = name.into();
        if let Some(name) = name {
            self.name = name;
        }
        self
    }

    /// Indicate if the function is an arrow function.
    pub(crate) const fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }
    /// Indicate if the function is a method function.
    pub(crate) const fn method(mut self, method: bool) -> Self {
        self.method = method;
        self
    }
    /// Indicate if the function is a generator function.
    pub(crate) const fn generator(mut self, generator: bool) -> Self {
        self.generator = generator;
        self
    }

    /// Indicate if the function is an async function.
    pub(crate) const fn r#async(mut self, r#async: bool) -> Self {
        self.r#async = r#async;
        self
    }

    /// Indicate if the function is in a strict context.
    pub(crate) const fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Provide the name scope of the function.
    pub(crate) fn name_scope(mut self, name_scope: Option<Scope>) -> Self {
        self.name_scope = name_scope;
        self
    }

    /// Indicate if the function is in a `with` statement.
    pub(crate) const fn in_with(mut self, in_with: bool) -> Self {
        self.in_with = in_with;
        self
    }

    /// Indicate if the function is in a `with` statement.
    pub(crate) const fn force_function_scope(mut self, force_function_scope: bool) -> Self {
        self.force_function_scope = force_function_scope;
        self
    }

    /// Set source map file path.
    pub(crate) fn source_path(mut self, source_path: SourcePath) -> Self {
        self.source_path = source_path;
        self
    }

    /// Compile a function statement list and it's parameters into bytecode.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compile(
        mut self,
        parameters: &FormalParameterList,
        body: &FunctionBody,
        variable_environment: Scope,
        lexical_environment: Scope,
        scopes: &FunctionScopes,
        contains_direct_eval: bool,
        interner: &mut Interner,
    ) -> Gc<CodeBlock> {
        self.strict = self.strict || body.strict();

        let length = parameters.length();

        let mut compiler = ByteCompiler::new(
            self.name,
            self.strict,
            false,
            variable_environment,
            lexical_environment,
            self.r#async,
            self.generator,
            interner,
            self.in_with,
            self.spanned_source_text,
            self.source_path,
        );

        compiler.length = length;
        compiler.code_block_flags.set(
            CodeBlockFlags::HAS_PROTOTYPE_PROPERTY,
            !self.arrow && !self.method && !self.r#async && !self.generator,
        );

        if self.arrow {
            compiler.this_mode = ThisMode::Lexical;
        }

        if let Some(scope) = self.name_scope
            && !scope.all_bindings_local()
        {
            compiler.code_block_flags |= CodeBlockFlags::HAS_BINDING_IDENTIFIER;
            let _ = compiler.push_scope(&scope);
        }

        if contains_direct_eval || !scopes.function_scope().all_bindings_local() {
            compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
        } else if !self.arrow {
            compiler.code_block_flags.set(
                CodeBlockFlags::HAS_FUNCTION_SCOPE,
                self.force_function_scope || scopes.requires_function_scope(),
            );
        }

        if compiler.code_block_flags.has_function_scope() {
            let _ = compiler.push_scope(scopes.function_scope());
        } else {
            compiler.variable_scope = scopes.function_scope().clone();
            compiler.lexical_scope = scopes.function_scope().clone();
        }

        // Taken from:
        //  - 15.9.3 Runtime Semantics: EvaluateAsyncConciseBody: <https://tc39.es/ecma262/#sec-runtime-semantics-evaluateasyncconcisebody>
        //  - 15.8.4 Runtime Semantics: EvaluateAsyncFunctionBody: <https://tc39.es/ecma262/#sec-runtime-semantics-evaluateasyncfunctionbody>
        //
        // Note: In `EvaluateAsyncGeneratorBody` unlike the async non-generator functions we don't handle exceptions thrown by
        // `FunctionDeclarationInstantiation` (so they are propagated).
        //
        // See: 15.6.2 Runtime Semantics: EvaluateAsyncGeneratorBody: https://tc39.es/ecma262/#sec-runtime-semantics-evaluateasyncgeneratorbody
        if compiler.is_async() && !compiler.is_generator() {
            // 1. Let promiseCapability be ! NewPromiseCapability(%Promise%).
            compiler.bytecode.emit_create_promise_capability();

            // 2. Let declResult be Completion(FunctionDeclarationInstantiation(functionObject, argumentsList)).
            //
            // Note: We push an exception handler so we catch exceptions that are thrown by the
            // `FunctionDeclarationInstantiation` abstract function.
            //
            // Patched in `ByteCompiler::finish()`.
            compiler.async_handler = Some(compiler.push_handler());
        }

        compiler.function_declaration_instantiation(
            body,
            parameters,
            self.arrow,
            self.strict,
            self.generator,
            scopes,
        );

        // Taken from:
        // - 27.6.3.2 AsyncGeneratorStart ( generator, generatorBody ): <https://tc39.es/ecma262/#sec-asyncgeneratorstart>
        //
        // Note: We do handle exceptions thrown by generator body in `AsyncGeneratorStart`.
        if compiler.is_generator() {
            assert!(compiler.async_handler.is_none());

            if compiler.is_async() {
                // Patched in `ByteCompiler::finish()`.
                compiler.async_handler = Some(compiler.push_handler());
            }
        }

        {
            let mut compiler = compiler.position_guard(body);
            
            // Check if the function body contains `using` declarations
            #[cfg(feature = "experimental")]
            let using_count: u32 = {
                use boa_ast::{
                    declaration::LexicalDeclaration,
                    operations::{LexicallyScopedDeclaration, lexically_scoped_declarations},
                };
                
                lexically_scoped_declarations(body.statement_list())
                    .iter()
                    .filter_map(|decl| {
                        if let LexicallyScopedDeclaration::LexicalDeclaration(
                            LexicalDeclaration::Using(u) | LexicalDeclaration::AwaitUsing(u),
                        ) = decl
                        {
                            Some(u.as_ref().len() as u32)
                        } else {
                            None
                        }
                    })
                    .sum()
            };

            #[cfg(not(feature = "experimental"))]
            let using_count: u32 = 0;

            if using_count > 0 {
                #[cfg(feature = "experimental")]
                {
                    use crate::bytecompiler::jump_control::JumpControlInfoFlags;
                    
                    // Function body with `using` declarations needs try-finally semantics
                    let finally_re_throw = compiler.register_allocator.alloc();
                    let finally_jump_index = compiler.register_allocator.alloc();

                    compiler.bytecode.emit_store_true(finally_re_throw.variable());
                    compiler.bytecode.emit_store_zero(finally_jump_index.variable());

                    // Push jump control info to handle break/continue/return through disposal
                    compiler.push_try_with_finally_control_info(
                        &finally_re_throw,
                        &finally_jump_index,
                        false,
                    );

                    // Push exception handler
                    let handler = compiler.push_handler();

                    // Compile the function body
                    compiler.compile_statement_list(body.statement_list(), false, false);

                    // Normal exit: mark that we don't need to re-throw
                    compiler.bytecode.emit_store_false(finally_re_throw.variable());

                    let finally_jump = compiler.jump();

                    // Exception path: patch the handler
                    compiler.patch_handler(handler);

                    // Push a second handler for exceptions during exception handling
                    let catch_handler = compiler.push_handler();
                    let error = compiler.register_allocator.alloc();
                    compiler.bytecode.emit_exception(error.variable());
                    compiler.bytecode.emit_store_true(finally_re_throw.variable());

                    let no_throw = compiler.jump();
                    compiler.patch_handler(catch_handler);

                    compiler.patch_jump(no_throw);
                    compiler.patch_jump(finally_jump);

                    // Finally block: dispose resources
                    let finally_start = compiler.next_opcode_location();
                    compiler.jump_info
                        .last_mut()
                        .expect("there should be a jump control info")
                        .flags |= JumpControlInfoFlags::IN_FINALLY;

                    // Save accumulator
                    let value = compiler.register_allocator.alloc();
                    compiler.bytecode
                        .emit_set_register_from_accumulator(value.variable());

                    // Emit disposal logic
                    compiler.bytecode.emit_dispose_resources(using_count.into());

                    // Restore accumulator
                    compiler.bytecode.emit_set_accumulator(value.variable());
                    compiler.register_allocator.dealloc(value);

                    // Re-throw if there was an exception
                    let do_not_throw_exit = compiler.jump_if_false(&finally_re_throw);
                    compiler.bytecode.emit_throw(error.variable());
                    compiler.register_allocator.dealloc(error);
                    compiler.patch_jump(do_not_throw_exit);

                    // Pop jump control info (this handles break/continue/return via jump table)
                    compiler.pop_try_with_finally_control_info(finally_start);

                    compiler.register_allocator.dealloc(finally_re_throw);
                    compiler.register_allocator.dealloc(finally_jump_index);
                }
            } else {
                // Normal function body compilation (no using declarations)
                compiler.compile_statement_list(body.statement_list(), false, false);
            }
        }

        compiler.params = parameters.clone();
        compiler.parameter_scope = scopes.parameter_scope();

        let code = compiler.finish();

        Gc::new(code)
    }
}
