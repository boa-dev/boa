use std::rc::Rc;

use crate::{
    builtins::function::ThisMode,
    bytecompiler::{ByteCompiler, ByteCompilerFlags},
    environments::CompileTimeEnvironment,
    vm::{CodeBlock, CodeBlockFlags, Opcode},
    Context,
};
use boa_ast::{
    function::{FormalParameterList, FunctionBody},
    operations::can_optimize_local_variables,
};
use boa_gc::Gc;
use boa_interner::Sym;

/// `FunctionCompiler` is used to compile AST functions to bytecode.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct FunctionCompiler {
    name: Sym,
    generator: bool,
    r#async: bool,
    strict: bool,
    arrow: bool,
    binding_identifier: Option<Sym>,

    can_optimize: bool,
}

impl FunctionCompiler {
    /// Create a new `FunctionCompiler`.
    pub(crate) const fn new() -> Self {
        Self {
            name: Sym::EMPTY_STRING,
            generator: false,
            r#async: false,
            strict: false,
            arrow: false,
            binding_identifier: None,
            can_optimize: true,
        }
    }

    /// Set the name of the function.
    pub(crate) fn name<N>(mut self, name: N) -> Self
    where
        N: Into<Option<Sym>>,
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

    /// Indicate if the function has a binding identifier.
    pub(crate) const fn binding_identifier(mut self, binding_identifier: Option<Sym>) -> Self {
        self.binding_identifier = binding_identifier;
        self
    }

    /// Indicate if the function can be optimized.
    pub(crate) const fn can_optimize(mut self, can_optimize: bool) -> Self {
        self.can_optimize = can_optimize;
        self
    }

    /// Compile a function statement list and it's parameters into bytecode.
    pub(crate) fn compile(
        mut self,
        parameters: &FormalParameterList,
        body: &FunctionBody,
        outer_env: Rc<CompileTimeEnvironment>,
        context: &mut Context<'_>,
    ) -> Gc<CodeBlock> {
        self.strict = self.strict || body.strict();

        let length = parameters.length();

        let mut compiler = ByteCompiler::new(self.name, self.strict, false, outer_env, context);
        compiler.length = length;

        compiler.flags.set(ByteCompilerFlags::ASYNC, self.r#async);
        compiler
            .flags
            .set(ByteCompilerFlags::GENERATOR, self.generator);

        if self.arrow {
            compiler.this_mode = ThisMode::Lexical;
        }

        if let Some(binding_identifier) = self.binding_identifier {
            compiler.code_block_flags |= CodeBlockFlags::HAS_BINDING_IDENTIFIER;
            let _ = compiler.push_compile_environment(false);
            compiler.create_immutable_binding(binding_identifier.into(), self.strict);
        }

        // Function environment
        let _ = compiler.push_compile_environment(true);

        compiler.function_environment_index =
            Some(compiler.current_environment.environment_index());

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

            // Note: We set it to one so we don't pop return value when we return.
            compiler.current_stack_value_count += 1;

            // 2. Let declResult be Completion(FunctionDeclarationInstantiation(functionObject, argumentsList)).
            //
            // Note: We push an exception handler so we catch exceptions that are thrown by the
            // `FunctionDeclarationInstantiation` abstract function.
            //
            // Patched in `ByteCompiler::finish()`.
            compiler.async_handler = Some(compiler.push_handler());
        }

        let can_optimize_params = can_optimize_local_variables(parameters);
        let can_optimize_body = can_optimize_local_variables(body);
        // println!("Can optimize params: {can_optimize_params}");
        // println!("Can optimize body: {can_optimize_body}");

        let can_optimize =
            can_optimize_params && can_optimize_body && parameters.is_simple() && self.can_optimize;

        println!("Can optimize function: {can_optimize}");

        compiler.can_optimize_local_variables =
            can_optimize && compiler.function_environment_index.is_some();

        let (env_label, additional_env) = compiler.function_declaration_instantiation(
            body,
            parameters,
            self.arrow,
            self.strict,
            self.generator,
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

        compiler.compile_statement_list(body.statements(), false, false);

        if env_label {
            compiler.pop_compile_environment();
        }

        if additional_env {
            compiler.pop_compile_environment();
            compiler.code_block_flags |= CodeBlockFlags::PARAMETERS_ENV_BINDINGS;
        }

        compiler.pop_compile_environment();

        if self.binding_identifier.is_some() {
            compiler.pop_compile_environment();
        }

        compiler.params = parameters.clone();

        Gc::new(compiler.finish())
    }
}
