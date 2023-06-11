use boa_ast::{
    declaration::Binding,
    operations::{bound_names, returns_value},
    statement::{
        iteration::{ForLoopInitializer, IterableLoopInitializer},
        DoWhileLoop, ForInLoop, ForLoop, ForOfLoop, WhileLoop,
    },
};
use boa_interner::Sym;

use crate::{
    bytecompiler::{Access, ByteCompiler},
    vm::{BindingOpcode, Opcode},
};

impl ByteCompiler<'_, '_> {
    pub(crate) fn compile_for_loop(
        &mut self,
        for_loop: &ForLoop,
        label: Option<Sym>,
        use_expr: bool,
    ) {
        let mut let_binding_indices = None;
        let mut env_labels = None;
        let mut iteration_env_labels = None;

        if let Some(init) = for_loop.init() {
            match init {
                ForLoopInitializer::Expression(expr) => self.compile_expr(expr, false),
                ForLoopInitializer::Var(decl) => {
                    self.compile_var_decl(decl);
                }
                ForLoopInitializer::Lexical(decl) => {
                    self.push_compile_environment(false);
                    env_labels =
                        Some(self.emit_opcode_with_operand(Opcode::PushDeclarativeEnvironment));

                    let names = bound_names(decl);
                    if decl.is_const() {
                        for name in &names {
                            self.create_immutable_binding(*name, true);
                        }
                    } else {
                        let mut indices = Vec::new();
                        for name in &names {
                            self.create_mutable_binding(*name, false);
                            let binding = self.initialize_mutable_binding(*name, false);
                            let index = self.get_or_insert_binding(binding);
                            indices.push(index);
                        }
                        let_binding_indices = Some(indices);
                    }
                    self.compile_lexical_decl(decl);
                }
            }
        }

        self.push_empty_loop_jump_control();
        let (loop_start, loop_exit) = self.emit_opcode_with_two_operands(Opcode::LoopStart);
        let initial_jump = self.jump();
        let start_address = self.next_opcode_location();

        self.current_jump_control_mut()
            .expect("jump_control must exist as it was just pushed")
            .set_label(label);
        self.current_jump_control_mut()
            .expect("jump_control must exist as it was just pushed")
            .set_start_address(start_address);

        if let Some(let_binding_indices) = let_binding_indices {
            for index in &let_binding_indices {
                self.emit(Opcode::GetName, &[*index]);
            }
            self.emit_opcode(Opcode::PopEnvironment);
            iteration_env_labels =
                Some(self.emit_opcode_with_operand(Opcode::PushDeclarativeEnvironment));
            for index in let_binding_indices.iter().rev() {
                self.emit(Opcode::PutLexicalValue, &[*index]);
            }
        }

        self.emit_opcode(Opcode::LoopContinue);
        self.patch_jump_with_target(loop_start, start_address);

        if let Some(final_expr) = for_loop.final_expr() {
            self.compile_expr(final_expr, false);
        }

        self.patch_jump(initial_jump);

        if let Some(condition) = for_loop.condition() {
            self.compile_expr(condition, true);
        } else {
            self.emit_opcode(Opcode::PushTrue);
        }
        let exit = self.jump_if_false();

        if !returns_value(for_loop.body()) {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.compile_stmt(for_loop.body(), true);
        self.emit_opcode(Opcode::LoopUpdateReturnValue);

        self.emit(Opcode::Jump, &[start_address]);

        self.patch_jump(exit);
        self.patch_jump(loop_exit);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }

        if let Some(env_labels) = env_labels {
            let env_index = self.pop_compile_environment();
            self.patch_jump_with_target(env_labels, env_index);
            if let Some(iteration_env_labels) = iteration_env_labels {
                self.patch_jump_with_target(iteration_env_labels, env_index);
            }
            self.emit_opcode(Opcode::PopEnvironment);
        }
    }

    pub(crate) fn compile_for_in_loop(
        &mut self,
        for_in_loop: &ForInLoop,
        label: Option<Sym>,
        use_expr: bool,
    ) {
        // Handle https://tc39.es/ecma262/#prod-annexB-ForInOfStatement
        if let IterableLoopInitializer::Var(var) = for_in_loop.initializer() {
            if let Binding::Identifier(ident) = var.binding() {
                if let Some(init) = var.init() {
                    self.compile_expr(init, true);
                    self.emit_binding(BindingOpcode::InitVar, *ident);
                }
            }
        }
        let initializer_bound_names = match for_in_loop.initializer() {
            IterableLoopInitializer::Let(declaration)
            | IterableLoopInitializer::Const(declaration) => bound_names(declaration),
            _ => Vec::new(),
        };
        if initializer_bound_names.is_empty() {
            self.compile_expr(for_in_loop.target(), true);
        } else {
            self.push_compile_environment(false);
            let push_env = self.emit_opcode_with_operand(Opcode::PushDeclarativeEnvironment);

            for name in &initializer_bound_names {
                self.create_mutable_binding(*name, false);
            }
            self.compile_expr(for_in_loop.target(), true);

            let env_index = self.pop_compile_environment();
            self.patch_jump_with_target(push_env, env_index);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        let early_exit = self.jump_if_null_or_undefined();
        self.emit_opcode(Opcode::CreateForInIterator);

        let (loop_start, exit_label) =
            self.emit_opcode_with_two_operands(Opcode::IteratorLoopStart);
        let start_address = self.next_opcode_location();
        self.push_loop_control_info_for_of_in_loop(label, start_address);
        self.emit_opcode(Opcode::LoopContinue);
        self.patch_jump_with_target(loop_start, start_address);

        self.emit_opcode(Opcode::IteratorNext);
        self.emit_opcode(Opcode::IteratorDone);
        let exit = self.jump_if_true();

        self.emit_opcode(Opcode::IteratorValue);

        let iteration_environment = if initializer_bound_names.is_empty() {
            None
        } else {
            self.push_compile_environment(false);
            Some(self.emit_opcode_with_operand(Opcode::PushDeclarativeEnvironment))
        };

        match for_in_loop.initializer() {
            IterableLoopInitializer::Identifier(ident) => {
                self.emit_binding(BindingOpcode::InitVar, *ident);
            }
            IterableLoopInitializer::Access(access) => {
                self.access_set(
                    Access::Property { access },
                    false,
                    ByteCompiler::access_set_top_of_stack_expr_fn,
                );
            }
            IterableLoopInitializer::Var(declaration) => match declaration.binding() {
                Binding::Identifier(ident) => {
                    self.emit_binding(BindingOpcode::InitVar, *ident);
                }
                Binding::Pattern(pattern) => {
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar);
                }
            },
            IterableLoopInitializer::Let(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.create_mutable_binding(*ident, false);
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_mutable_binding(ident, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLet);
                }
            },
            IterableLoopInitializer::Const(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.create_immutable_binding(*ident, true);
                    self.emit_binding(BindingOpcode::InitConst, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_immutable_binding(ident, true);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitConst);
                }
            },
            IterableLoopInitializer::Pattern(pattern) => {
                self.compile_declaration_pattern(pattern, BindingOpcode::SetName);
            }
        }

        if !returns_value(for_in_loop.body()) {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.compile_stmt(for_in_loop.body(), true);
        self.emit_opcode(Opcode::LoopUpdateReturnValue);

        if let Some(iteration_environment) = iteration_environment {
            let env_index = self.pop_compile_environment();
            self.patch_jump_with_target(iteration_environment, env_index);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        self.emit(Opcode::Jump, &[start_address]);

        self.patch_jump(exit);
        self.patch_jump(exit_label);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);

        self.iterator_close(false);

        let skip_early_exit = self.jump();
        self.patch_jump(early_exit);
        self.emit_opcode(Opcode::PushUndefined);
        self.patch_jump(skip_early_exit);

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }

    pub(crate) fn compile_for_of_loop(
        &mut self,
        for_of_loop: &ForOfLoop,
        label: Option<Sym>,
        use_expr: bool,
    ) {
        let initializer_bound_names = match for_of_loop.initializer() {
            IterableLoopInitializer::Let(declaration)
            | IterableLoopInitializer::Const(declaration) => bound_names(declaration),
            _ => Vec::new(),
        };
        if initializer_bound_names.is_empty() {
            self.compile_expr(for_of_loop.iterable(), true);
        } else {
            self.push_compile_environment(false);
            let push_env = self.emit_opcode_with_operand(Opcode::PushDeclarativeEnvironment);

            for name in &initializer_bound_names {
                self.create_mutable_binding(*name, false);
            }
            self.compile_expr(for_of_loop.iterable(), true);

            let env_index = self.pop_compile_environment();
            self.patch_jump_with_target(push_env, env_index);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        if for_of_loop.r#await() {
            self.emit_opcode(Opcode::GetAsyncIterator);
        } else {
            self.emit_opcode(Opcode::GetIterator);
        }

        let (loop_start, loop_exit) = self.emit_opcode_with_two_operands(Opcode::IteratorLoopStart);
        let start_address = self.next_opcode_location();
        if for_of_loop.r#await() {
            self.push_loop_control_info_for_await_of_loop(label, start_address);
        } else {
            self.push_loop_control_info_for_of_in_loop(label, start_address);
        }
        self.emit_opcode(Opcode::LoopContinue);
        self.patch_jump_with_target(loop_start, start_address);

        self.emit_opcode(Opcode::IteratorNext);
        if for_of_loop.r#await() {
            self.emit_opcode(Opcode::IteratorResult);
            self.emit_opcode(Opcode::Await);
            self.emit_opcode(Opcode::IteratorFinishAsyncNext);
            self.emit_opcode(Opcode::GeneratorNext);
        }
        self.emit_opcode(Opcode::IteratorDone);
        let exit = self.jump_if_true();
        self.emit_opcode(Opcode::IteratorValue);

        let iteration_environment = if initializer_bound_names.is_empty() {
            None
        } else {
            self.push_compile_environment(false);
            Some(self.emit_opcode_with_operand(Opcode::PushDeclarativeEnvironment))
        };

        match for_of_loop.initializer() {
            IterableLoopInitializer::Identifier(ref ident) => {
                match self.set_mutable_binding(*ident) {
                    Ok(binding) => {
                        let index = self.get_or_insert_binding(binding);
                        self.emit(Opcode::DefInitVar, &[index]);
                    }
                    Err(()) => {
                        let index = self.get_or_insert_name(*ident);
                        self.emit(Opcode::ThrowMutateImmutable, &[index]);
                    }
                }
            }
            IterableLoopInitializer::Access(access) => {
                self.access_set(
                    Access::Property { access },
                    false,
                    ByteCompiler::access_set_top_of_stack_expr_fn,
                );
            }
            IterableLoopInitializer::Var(declaration) => {
                // ignore initializers since those aren't allowed on for-of loops.
                assert!(declaration.init().is_none());
                match declaration.binding() {
                    Binding::Identifier(ident) => {
                        self.emit_binding(BindingOpcode::InitVar, *ident);
                    }
                    Binding::Pattern(pattern) => {
                        self.compile_declaration_pattern(pattern, BindingOpcode::InitVar);
                    }
                }
            }
            IterableLoopInitializer::Let(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.create_mutable_binding(*ident, false);
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_mutable_binding(ident, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLet);
                }
            },
            IterableLoopInitializer::Const(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.create_immutable_binding(*ident, true);
                    self.emit_binding(BindingOpcode::InitConst, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_immutable_binding(ident, true);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitConst);
                }
            },
            IterableLoopInitializer::Pattern(pattern) => {
                self.compile_declaration_pattern(pattern, BindingOpcode::SetName);
            }
        }

        if !returns_value(for_of_loop.body()) {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.compile_stmt(for_of_loop.body(), true);
        self.emit_opcode(Opcode::LoopUpdateReturnValue);

        if let Some(iteration_environment) = iteration_environment {
            let env_index = self.pop_compile_environment();
            self.patch_jump_with_target(iteration_environment, env_index);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        self.emit(Opcode::Jump, &[start_address]);

        self.patch_jump(exit);
        self.patch_jump(loop_exit);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);

        self.iterator_close(for_of_loop.r#await());

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }

    pub(crate) fn compile_while_loop(
        &mut self,
        while_loop: &WhileLoop,
        label: Option<Sym>,
        use_expr: bool,
    ) {
        let (loop_start, loop_exit) = self.emit_opcode_with_two_operands(Opcode::LoopStart);
        let start_address = self.next_opcode_location();
        self.emit_opcode(Opcode::LoopContinue);
        self.push_loop_control_info(label, start_address);
        self.patch_jump_with_target(loop_start, start_address);

        self.compile_expr(while_loop.condition(), true);
        let exit = self.jump_if_false();

        if !returns_value(while_loop.body()) {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.compile_stmt(while_loop.body(), true);
        self.emit_opcode(Opcode::LoopUpdateReturnValue);

        self.emit(Opcode::Jump, &[start_address]);

        self.patch_jump(exit);
        self.patch_jump(loop_exit);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }

    pub(crate) fn compile_do_while_loop(
        &mut self,
        do_while_loop: &DoWhileLoop,
        label: Option<Sym>,
        use_expr: bool,
    ) {
        let (loop_start, loop_exit) = self.emit_opcode_with_two_operands(Opcode::LoopStart);
        let initial_label = self.jump();

        let start_address = self.next_opcode_location();

        self.patch_jump_with_target(loop_start, start_address);
        self.push_loop_control_info(label, start_address);

        let condition_label_address = self.next_opcode_location();
        self.emit_opcode(Opcode::LoopContinue);
        self.compile_expr(do_while_loop.cond(), true);
        let exit = self.jump_if_false();

        self.patch_jump(initial_label);

        if !returns_value(do_while_loop.body()) {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.compile_stmt(do_while_loop.body(), true);
        self.emit_opcode(Opcode::LoopUpdateReturnValue);

        self.emit(Opcode::Jump, &[condition_label_address]);
        self.patch_jump(exit);
        self.patch_jump(loop_exit);

        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }
}
