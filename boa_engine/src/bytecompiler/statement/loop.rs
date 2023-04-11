use boa_ast::{
    declaration::Binding,
    operations::bound_names,
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
        configurable_globals: bool,
    ) {
        self.push_compile_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
        self.push_empty_loop_jump_control();

        if let Some(init) = for_loop.init() {
            match init {
                ForLoopInitializer::Expression(expr) => self.compile_expr(expr, false),
                ForLoopInitializer::Var(decl) => {
                    self.create_decls_from_var_decl(decl, configurable_globals);
                    self.compile_var_decl(decl);
                }
                ForLoopInitializer::Lexical(decl) => {
                    self.create_decls_from_lexical_decl(decl);
                    self.compile_lexical_decl(decl);
                }
            }
        }

        let (loop_start, loop_exit) = self.emit_opcode_with_two_operands(Opcode::LoopStart);
        let initial_jump = self.jump();
        let start_address = self.next_opcode_location();

        self.current_jump_control_mut()
            .expect("jump_control must exist as it was just pushed")
            .set_label(label);
        self.current_jump_control_mut()
            .expect("jump_control must exist as it was just pushed")
            .set_start_address(start_address);

        let (continue_start, continue_exit) =
            self.emit_opcode_with_two_operands(Opcode::LoopContinue);
        self.patch_jump_with_target(loop_start, start_address);
        self.patch_jump_with_target(continue_start, start_address);

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

        self.compile_stmt(for_loop.body(), false, configurable_globals);

        self.emit(Opcode::Jump, &[start_address]);

        let env_info = self.pop_compile_environment();
        self.patch_jump_with_target(push_env.0, env_info.num_bindings as u32);
        self.patch_jump_with_target(push_env.1, env_info.index as u32);

        self.patch_jump(exit);
        self.patch_jump(loop_exit);
        self.patch_jump(continue_exit);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
        self.emit_opcode(Opcode::PopEnvironment);
    }

    pub(crate) fn compile_for_in_loop(
        &mut self,
        for_in_loop: &ForInLoop,
        label: Option<Sym>,
        configurable_globals: bool,
    ) {
        let initializer_bound_names = match for_in_loop.initializer() {
            IterableLoopInitializer::Let(declaration)
            | IterableLoopInitializer::Const(declaration) => bound_names(declaration),
            _ => Vec::new(),
        };
        if initializer_bound_names.is_empty() {
            self.compile_expr(for_in_loop.target(), true);
        } else {
            self.push_compile_environment(false);
            let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

            for name in &initializer_bound_names {
                self.create_mutable_binding(*name, false, false);
            }
            self.compile_expr(for_in_loop.target(), true);

            let env_info = self.pop_compile_environment();
            self.patch_jump_with_target(push_env.0, env_info.num_bindings as u32);
            self.patch_jump_with_target(push_env.1, env_info.index as u32);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        let early_exit = self.jump_if_null_or_undefined();
        self.emit_opcode(Opcode::CreateForInIterator);
        self.emit_opcode(Opcode::PushFalse);

        let (loop_start, exit_label) = self.emit_opcode_with_two_operands(Opcode::LoopStart);
        let start_address = self.next_opcode_location();
        self.push_loop_control_info_for_of_in_loop(label, start_address);
        let (continue_label, cont_exit_label) =
            self.emit_opcode_with_two_operands(Opcode::LoopContinue);
        self.patch_jump_with_target(continue_label, start_address);
        self.patch_jump_with_target(loop_start, start_address);

        self.emit_opcode(Opcode::Pop); // pop the `done` value.
        self.emit_opcode(Opcode::IteratorNext);
        let exit = self.emit_opcode_with_operand(Opcode::IteratorUnwrapNextOrJump);

        let iteration_environment = if initializer_bound_names.is_empty() {
            None
        } else {
            self.push_compile_environment(false);
            Some(self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment))
        };

        match for_in_loop.initializer() {
            IterableLoopInitializer::Identifier(ident) => {
                self.create_mutable_binding(*ident, true, true);
                let binding = self.set_mutable_binding(*ident);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefInitVar, &[index]);
            }
            IterableLoopInitializer::Access(access) => {
                self.access_set(
                    Access::Property { access },
                    false,
                    ByteCompiler::access_set_top_of_stack_expr_fn,
                );
            }
            IterableLoopInitializer::Var(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.create_mutable_binding(*ident, true, configurable_globals);
                    self.emit_binding(BindingOpcode::InitVar, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_mutable_binding(ident, true, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar);
                }
            },
            IterableLoopInitializer::Let(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.create_mutable_binding(*ident, false, false);
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_mutable_binding(ident, false, false);
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

        self.compile_stmt(for_in_loop.body(), false, configurable_globals);

        if let Some(iteration_environment) = iteration_environment {
            let env_info = self.pop_compile_environment();
            self.patch_jump_with_target(iteration_environment.0, env_info.num_bindings as u32);
            self.patch_jump_with_target(iteration_environment.1, env_info.index as u32);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        self.emit(Opcode::Jump, &[start_address]);

        self.patch_jump(exit);
        self.patch_jump(exit_label);
        self.patch_jump(cont_exit_label);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
        self.emit_opcode(Opcode::Pop);
        self.emit_opcode(Opcode::Pop);
        self.emit_opcode(Opcode::Pop);

        self.patch_jump(early_exit);
    }

    pub(crate) fn compile_for_of_loop(
        &mut self,
        for_of_loop: &ForOfLoop,
        label: Option<Sym>,
        configurable_globals: bool,
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
            let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

            for name in &initializer_bound_names {
                self.create_mutable_binding(*name, false, false);
            }
            self.compile_expr(for_of_loop.iterable(), true);

            let env_info = self.pop_compile_environment();
            self.patch_jump_with_target(push_env.0, env_info.num_bindings as u32);
            self.patch_jump_with_target(push_env.1, env_info.index as u32);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        if for_of_loop.r#await() {
            self.emit_opcode(Opcode::GetAsyncIterator);
        } else {
            self.emit_opcode(Opcode::GetIterator);
        }
        self.emit_opcode(Opcode::PushFalse);

        let (loop_start, loop_exit) = self.emit_opcode_with_two_operands(Opcode::LoopStart);
        let start_address = self.next_opcode_location();
        self.push_loop_control_info_for_of_in_loop(label, start_address);
        let (cont_label, cont_exit_label) =
            self.emit_opcode_with_two_operands(Opcode::LoopContinue);
        self.patch_jump_with_target(loop_start, start_address);
        self.patch_jump_with_target(cont_label, start_address);

        self.emit_opcode(Opcode::Pop); // pop the `done` value.
        self.emit_opcode(Opcode::IteratorNext);
        if for_of_loop.r#await() {
            self.emit_opcode(Opcode::Await);
            self.emit_opcode(Opcode::GeneratorNext);
        }
        let exit = self.emit_opcode_with_operand(Opcode::IteratorUnwrapNextOrJump);

        let iteration_environment = if initializer_bound_names.is_empty() {
            None
        } else {
            self.push_compile_environment(false);
            Some(self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment))
        };

        match for_of_loop.initializer() {
            IterableLoopInitializer::Identifier(ref ident) => {
                self.create_mutable_binding(*ident, true, true);
                let binding = self.set_mutable_binding(*ident);
                let index = self.get_or_insert_binding(binding);
                self.emit(Opcode::DefInitVar, &[index]);
            }
            IterableLoopInitializer::Access(access) => {
                self.access_set(
                    Access::Property { access },
                    false,
                    ByteCompiler::access_set_top_of_stack_expr_fn,
                );
            }
            IterableLoopInitializer::Var(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.create_mutable_binding(*ident, true, false);
                    self.emit_binding(BindingOpcode::InitVar, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_mutable_binding(ident, true, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar);
                }
            },
            IterableLoopInitializer::Let(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    self.create_mutable_binding(*ident, false, false);
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_mutable_binding(ident, false, false);
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

        self.compile_stmt(for_of_loop.body(), false, configurable_globals);

        if let Some(iteration_environment) = iteration_environment {
            let env_info = self.pop_compile_environment();
            self.patch_jump_with_target(iteration_environment.0, env_info.num_bindings as u32);
            self.patch_jump_with_target(iteration_environment.1, env_info.index as u32);
            self.emit_opcode(Opcode::PopEnvironment);
        }

        self.emit(Opcode::Jump, &[start_address]);

        self.patch_jump(exit);
        self.patch_jump(loop_exit);
        self.patch_jump(cont_exit_label);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
        self.iterator_close(for_of_loop.r#await());
    }

    pub(crate) fn compile_while_loop(
        &mut self,
        while_loop: &WhileLoop,
        label: Option<Sym>,
        configurable_globals: bool,
    ) {
        let (loop_start, loop_exit) = self.emit_opcode_with_two_operands(Opcode::LoopStart);
        let start_address = self.next_opcode_location();
        let (continue_start, continue_exit) =
            self.emit_opcode_with_two_operands(Opcode::LoopContinue);
        self.push_loop_control_info(label, start_address);
        self.patch_jump_with_target(loop_start, start_address);
        self.patch_jump_with_target(continue_start, start_address);

        self.compile_expr(while_loop.condition(), true);
        let exit = self.jump_if_false();
        self.compile_stmt(while_loop.body(), false, configurable_globals);
        self.emit(Opcode::Jump, &[start_address]);

        self.patch_jump(exit);
        self.patch_jump(loop_exit);
        self.patch_jump(continue_exit);
        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
    }

    pub(crate) fn compile_do_while_loop(
        &mut self,
        do_while_loop: &DoWhileLoop,
        label: Option<Sym>,
        configurable_globals: bool,
    ) {
        let (loop_start, loop_exit) = self.emit_opcode_with_two_operands(Opcode::LoopStart);
        let initial_label = self.jump();

        let start_address = self.next_opcode_location();
        let (continue_start, continue_exit) =
            self.emit_opcode_with_two_operands(Opcode::LoopContinue);
        self.patch_jump_with_target(continue_start, start_address);
        self.patch_jump_with_target(loop_start, start_address);
        self.push_loop_control_info(label, start_address);

        let condition_label_address = self.next_opcode_location();
        self.compile_expr(do_while_loop.cond(), true);
        let exit = self.jump_if_false();

        self.patch_jump(initial_label);

        self.compile_stmt(do_while_loop.body(), false, configurable_globals);
        self.emit(Opcode::Jump, &[condition_label_address]);
        self.patch_jump(exit);
        self.patch_jump(loop_exit);
        self.patch_jump(continue_exit);

        self.pop_loop_control_info();
        self.emit_opcode(Opcode::LoopEnd);
    }
}
