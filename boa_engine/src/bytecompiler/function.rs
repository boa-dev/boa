use crate::{
    builtins::function::ThisMode,
    bytecompiler::ByteCompiler,
    vm::{BindingOpcode, CodeBlock, Opcode},
    Context, JsResult,
};
use boa_ast::{
    declaration::Binding, function::FormalParameterList, operations::bound_names, StatementList,
};
use boa_gc::Gc;
use boa_interner::Sym;
use rustc_hash::FxHashMap;

/// `FunctionCompiler` is used to compile AST functions to bytecode.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct FunctionCompiler {
    name: Sym,
    generator: bool,
    r#async: bool,
    strict: bool,
    arrow: bool,
    has_binding_identifier: bool,
}

impl FunctionCompiler {
    /// Create a new `FunctionCompiler`.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            name: Sym::EMPTY_STRING,
            generator: false,
            r#async: false,
            strict: false,
            arrow: false,
            has_binding_identifier: false,
        }
    }

    /// Set the name of the function.
    #[inline]
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
    #[inline]
    pub(crate) fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }
    /// Indicate if the function is a generator function.
    #[inline]
    pub(crate) fn generator(mut self, generator: bool) -> Self {
        self.generator = generator;
        self
    }

    /// Indicate if the function is an async function.
    #[inline]
    pub(crate) fn r#async(mut self, r#async: bool) -> Self {
        self.r#async = r#async;
        self
    }

    /// Indicate if the function is in a strict context.
    #[inline]
    pub(crate) fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Indicate if the function has a binding identifier.
    #[inline]
    pub(crate) fn has_binding_identifier(mut self, has_binding_identifier: bool) -> Self {
        self.has_binding_identifier = has_binding_identifier;
        self
    }

    /// Compile a function statement list and it's parameters into bytecode.
    pub(crate) fn compile(
        mut self,
        parameters: &FormalParameterList,
        body: &StatementList,
        context: &mut Context,
    ) -> JsResult<Gc<CodeBlock>> {
        self.strict = self.strict || body.strict();

        let length = parameters.length();
        let mut code = CodeBlock::new(self.name, length, self.strict);

        if self.arrow {
            code.this_mode = ThisMode::Lexical;
        }

        let mut compiler = ByteCompiler {
            code_block: code,
            literals_map: FxHashMap::default(),
            names_map: FxHashMap::default(),
            bindings_map: FxHashMap::default(),
            jump_info: Vec::new(),
            in_async_generator: self.generator && self.r#async,
            json_parse: false,
            context,
        };

        if self.has_binding_identifier {
            compiler.code_block.has_binding_identifier = true;
            compiler.context.push_compile_time_environment(false);
            compiler
                .context
                .create_immutable_binding(self.name.into(), self.strict);
        }

        compiler.context.push_compile_time_environment(true);

        // An arguments object is added when all of the following conditions are met
        // - If not in an arrow function (10.2.11.16)
        // - If the parameter list does not contain `arguments` (10.2.11.17)
        // Note: This following just means, that we add an extra environment for the arguments.
        // - If there are default parameters or if lexical names and function names do not contain `arguments` (10.2.11.18)
        if !(self.arrow) && !parameters.has_arguments() {
            compiler
                .context
                .create_mutable_binding(Sym::ARGUMENTS.into(), false, false);
            compiler.code_block.arguments_binding = Some(
                compiler
                    .context
                    .initialize_mutable_binding(Sym::ARGUMENTS.into(), false),
            );
        }

        for parameter in parameters.as_ref() {
            if parameter.is_rest_param() {
                compiler.emit_opcode(Opcode::RestParameterInit);
            }

            match parameter.variable().binding() {
                Binding::Identifier(ident) => {
                    compiler
                        .context
                        .create_mutable_binding(*ident, false, false);
                    // TODO: throw custom error if ident is in init
                    if let Some(init) = parameter.variable().init() {
                        let skip = compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                        compiler.compile_expr(init, true)?;
                        compiler.patch_jump(skip);
                    }
                    compiler.emit_binding(BindingOpcode::InitArg, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        compiler.context.create_mutable_binding(ident, false, false);
                    }
                    // TODO: throw custom error if ident is in init
                    if let Some(init) = parameter.variable().init() {
                        let skip = compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                        compiler.compile_expr(init, true)?;
                        compiler.patch_jump(skip);
                    }
                    compiler.compile_declaration_pattern(pattern, BindingOpcode::InitArg)?;
                }
            }
        }

        if !parameters.has_rest_parameter() {
            compiler.emit_opcode(Opcode::RestParameterPop);
        }

        let env_label = if parameters.has_expressions() {
            compiler.code_block.num_bindings = compiler.context.get_binding_number();
            compiler.context.push_compile_time_environment(true);
            compiler.code_block.function_environment_push_location =
                compiler.next_opcode_location();
            Some(compiler.emit_opcode_with_two_operands(Opcode::PushFunctionEnvironment))
        } else {
            None
        };

        // When a generator object is created from a generator function, the generator executes until here to init parameters.
        if self.generator {
            compiler.emit_opcode(Opcode::PushUndefined);
            compiler.emit_opcode(Opcode::Yield);
        }

        compiler.create_decls(body, false);
        compiler.compile_statement_list(body, false, false)?;

        if let Some(env_label) = env_label {
            let (num_bindings, compile_environment) =
                compiler.context.pop_compile_time_environment();
            let index_compile_environment = compiler.push_compile_environment(compile_environment);
            compiler.patch_jump_with_target(env_label.0, num_bindings as u32);
            compiler.patch_jump_with_target(env_label.1, index_compile_environment as u32);

            let (_, compile_environment) = compiler.context.pop_compile_time_environment();
            compiler.push_compile_environment(compile_environment);
        } else {
            let (num_bindings, compile_environment) =
                compiler.context.pop_compile_time_environment();
            compiler.push_compile_environment(compile_environment);
            compiler.code_block.num_bindings = num_bindings;
        }

        if self.has_binding_identifier {
            let (_, compile_environment) = compiler.context.pop_compile_time_environment();
            compiler.push_compile_environment(compile_environment);
        }

        compiler.code_block.params = parameters.clone();

        // TODO These are redundant if a function returns so may need to check if a function returns and adding these if it doesn't
        compiler.emit(Opcode::PushUndefined, &[]);
        compiler.emit(Opcode::Return, &[]);

        Ok(Gc::new(compiler.finish()))
    }
}
