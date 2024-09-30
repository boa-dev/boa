use crate::{
    builtins::function::ThisMode,
    bytecompiler::ByteCompiler,
    js_string,
    vm::{CodeBlock, CodeBlockFlags, Opcode},
    JsString,
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
    name_scope: Option<Scope>,
}

impl FunctionCompiler {
    /// Create a new `FunctionCompiler`.
    pub(crate) fn new() -> Self {
        Self {
            name: js_string!(),
            generator: false,
            r#async: false,
            strict: false,
            arrow: false,
            method: false,
            in_with: false,
            name_scope: None,
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

    /// Compile a function statement list and it's parameters into bytecode.
    pub(crate) fn compile(
        mut self,
        parameters: &FormalParameterList,
        body: &FunctionBody,
        variable_environment: Scope,
        lexical_environment: Scope,
        scopes: &FunctionScopes,
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
        );
        compiler.length = length;
        compiler.code_block_flags.set(
            CodeBlockFlags::HAS_PROTOTYPE_PROPERTY,
            !self.arrow && !self.method && !self.r#async && !self.generator,
        );

        if self.arrow {
            compiler.this_mode = ThisMode::Lexical;
        }

        if let Some(scope) = self.name_scope {
            compiler.code_block_flags |= CodeBlockFlags::HAS_BINDING_IDENTIFIER;
            let _ = compiler.push_scope(&scope);
        }
        // Function environment
        let _ = compiler.push_scope(scopes.function_scope());

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
            //
            // Note: If the promise capability is already set, then we do nothing.
            // This is a deviation from the spec, but it allows to set the promise capability by
            // ExecuteAsyncModule ( module ): <https://tc39.es/ecma262/#sec-execute-async-module>
            compiler.emit_opcode(Opcode::CreatePromiseCapability);

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

        compiler.compile_statement_list(body.statement_list(), false, false);

        compiler.params = parameters.clone();

        let code = compiler.finish();

        Gc::new(code)
    }
}
