use crate::{
    builtins::function::ThisMode,
    bytecompiler::ByteCompiler,
    environments::CompileTimeEnvironment,
    vm::{BindingOpcode, CodeBlock, Opcode},
    Context,
};
use boa_ast::{
    declaration::Binding, function::FormalParameterList, operations::bound_names, StatementList,
};
use boa_gc::{Gc, GcRefCell};
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
    class_name: Option<Sym>,
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
            class_name: None,
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

    /// Indicate if the function has a class associated with it.
    pub(crate) const fn class_name(mut self, class_name: Sym) -> Self {
        self.class_name = Some(class_name);
        self
    }

    /// Compile a function statement list and it's parameters into bytecode.
    pub(crate) fn compile(
        mut self,
        parameters: &FormalParameterList,
        body: &StatementList,
        outer_env: Gc<GcRefCell<CompileTimeEnvironment>>,
        context: &mut Context<'_>,
    ) -> Gc<CodeBlock> {
        self.strict = self.strict || body.strict();

        let length = parameters.length();

        let mut compiler = ByteCompiler::new(self.name, self.strict, false, outer_env, context);
        compiler.length = length;
        compiler.in_async_generator = self.generator && self.r#async;

        if self.arrow {
            compiler.this_mode = ThisMode::Lexical;
        }

        if let Some(class_name) = self.class_name {
            compiler.push_compile_environment(false);
            compiler.create_immutable_binding(class_name.into(), true);
        }

        if let Some(binding_identifier) = self.binding_identifier {
            compiler.has_binding_identifier = true;
            compiler.push_compile_environment(false);
            compiler.create_immutable_binding(binding_identifier.into(), self.strict);
        }

        // Function environment
        compiler.push_compile_environment(true);

        // Only used to initialize bindings
        if !self.strict && parameters.has_expressions() {
            compiler.push_compile_environment(false);
        };

        // An arguments object is added when all of the following conditions are met
        // - If not in an arrow function (10.2.11.16)
        // - If the parameter list does not contain `arguments` (10.2.11.17)
        // Note: This following just means, that we add an extra environment for the arguments.
        // - If there are default parameters or if lexical names and function names do not contain `arguments` (10.2.11.18)
        if !(self.arrow) && !parameters.has_arguments() {
            let arguments = Sym::ARGUMENTS.into();
            compiler.arguments_binding = Some(if self.strict {
                compiler.create_immutable_binding(arguments, true);
                compiler.initialize_immutable_binding(arguments)
            } else {
                compiler.create_mutable_binding(arguments, false, false);
                compiler.initialize_mutable_binding(arguments, false)
            });
        }

        for parameter in parameters.as_ref() {
            if parameter.is_rest_param() {
                compiler.emit_opcode(Opcode::RestParameterInit);
            }

            match parameter.variable().binding() {
                Binding::Identifier(ident) => {
                    compiler.create_mutable_binding(*ident, false, false);
                    // TODO: throw custom error if ident is in init
                    if let Some(init) = parameter.variable().init() {
                        let skip = compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                        compiler.compile_expr(init, true);
                        compiler.patch_jump(skip);
                    }
                    compiler.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        compiler.create_mutable_binding(ident, false, false);
                    }
                    // TODO: throw custom error if ident is in init
                    if let Some(init) = parameter.variable().init() {
                        let skip = compiler.emit_opcode_with_operand(Opcode::JumpIfNotUndefined);
                        compiler.compile_expr(init, true);
                        compiler.patch_jump(skip);
                    }
                    compiler.compile_declaration_pattern(pattern, BindingOpcode::InitLet);
                }
            }
        }

        if !parameters.has_rest_parameter() {
            compiler.emit_opcode(Opcode::RestParameterPop);
        }

        let env_label = if parameters.has_expressions() {
            compiler.push_compile_environment(true);
            compiler.function_environment_push_location = compiler.next_opcode_location();
            Some(compiler.emit_opcode_with_two_operands(Opcode::PushFunctionEnvironment))
        } else {
            None
        };

        // When a generator object is created from a generator function, the generator executes until here to init parameters.
        if self.generator {
            compiler.emit_opcode(Opcode::PushUndefined);
            compiler.emit_opcode(Opcode::Yield);
        }

        compiler.create_script_decls(body, false);
        compiler.compile_statement_list(body, false, false);

        if let Some(env_label) = env_label {
            let env_info = compiler.pop_compile_environment();
            compiler.patch_jump_with_target(env_label.0, env_info.num_bindings as u32);
            compiler.patch_jump_with_target(env_label.1, env_info.index as u32);
        }

        if !self.strict && parameters.has_expressions() {
            compiler.parameters_env_bindings =
                Some(compiler.pop_compile_environment().num_bindings);
        }

        compiler.num_bindings = compiler.pop_compile_environment().num_bindings;

        if self.binding_identifier.is_some() {
            compiler.pop_compile_environment();
        }

        if self.class_name.is_some() {
            compiler.pop_compile_environment();
        }

        compiler.params = parameters.clone();

        // TODO These are redundant if a function returns so may need to check if a function returns and adding these if it doesn't
        compiler.emit(Opcode::PushUndefined, &[]);
        compiler.emit(Opcode::Return, &[]);

        Gc::new(compiler.finish())
    }
}
