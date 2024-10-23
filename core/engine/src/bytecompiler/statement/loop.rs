use boa_ast::{
    declaration::Binding,
    operations::bound_names,
    scope::BindingLocatorError,
    statement::{
        iteration::{ForLoopInitializer, IterableLoopInitializer},
        DoWhileLoop, ForInLoop, ForLoop, ForOfLoop, WhileLoop,
    },
};
use boa_interner::Sym;

use crate::{
    bytecompiler::{Access, ByteCompiler, Operand, ToJsString},
    vm::{BindingOpcode, Opcode},
};

impl ByteCompiler<'_> {
    pub(crate) fn compile_for_loop(
        &mut self,
        for_loop: &ForLoop,
        label: Option<Sym>,
        use_expr: bool,
    ) {
        let mut let_binding_indices = None;
        let mut outer_scope_local = None;
        let mut outer_scope = None;

        if let Some(init) = for_loop.init() {
            match init {
                ForLoopInitializer::Expression(expr) => self.compile_expr(expr, false),
                ForLoopInitializer::Var(decl) => {
                    self.compile_var_decl(decl);
                }
                ForLoopInitializer::Lexical(decl) => {
                    let scope_index = if decl.scope().all_bindings_local() {
                        outer_scope_local = Some(self.lexical_scope.clone());
                        self.lexical_scope = decl.scope().clone();
                        None
                    } else {
                        outer_scope = Some(self.lexical_scope.clone());
                        let scope_index = self.push_scope(decl.scope());
                        self.emit_with_varying_operand(Opcode::PushScope, scope_index);
                        Some(scope_index)
                    };

                    let names = bound_names(decl.declaration());
                    if decl.declaration().is_const() {
                    } else {
                        let mut indices = Vec::new();
                        for name in &names {
                            let name = name.to_js_string(self.interner());
                            let binding = decl
                                .scope()
                                .get_binding_reference(&name)
                                .expect("binding must exist");
                            let index = self.get_or_insert_binding(binding);
                            indices.push(index);
                        }
                        let_binding_indices = Some((indices, scope_index));
                    }
                    self.compile_lexical_decl(decl.declaration());
                }
            }
        }

        self.push_empty_loop_jump_control(use_expr);

        if let Some((let_binding_indices, scope_index)) = &let_binding_indices {
            for index in let_binding_indices {
                self.emit_binding_access(Opcode::GetName, index);
            }

            if let Some(index) = scope_index {
                self.emit_opcode(Opcode::PopEnvironment);
                self.emit_with_varying_operand(Opcode::PushScope, *index);
            }

            for index in let_binding_indices.iter().rev() {
                self.emit_binding_access(Opcode::PutLexicalValue, index);
            }
        }

        let initial_jump = self.jump();
        let start_address = self.next_opcode_location();

        self.current_jump_control_mut()
            .expect("jump_control must exist as it was just pushed")
            .set_label(label);
        self.current_jump_control_mut()
            .expect("jump_control must exist as it was just pushed")
            .set_start_address(start_address);

        if let Some((let_binding_indices, scope_index)) = &let_binding_indices {
            for index in let_binding_indices {
                self.emit_binding_access(Opcode::GetName, index);
            }

            if let Some(index) = scope_index {
                self.emit_opcode(Opcode::PopEnvironment);
                self.emit_with_varying_operand(Opcode::PushScope, *index);
            }

            for index in let_binding_indices.iter().rev() {
                self.emit_binding_access(Opcode::PutLexicalValue, index);
            }
        }

        self.emit_opcode(Opcode::IncrementLoopIteration);

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

        self.compile_stmt(for_loop.body(), use_expr, true);

        self.emit(Opcode::Jump, &[Operand::U32(start_address)]);

        self.patch_jump(exit);
        self.pop_loop_control_info();

        if let Some(outer_scope_local) = outer_scope_local {
            self.lexical_scope = outer_scope_local;
        }
        self.pop_declarative_scope(outer_scope);
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
                let ident = ident.to_js_string(self.interner());
                if let Some(init) = var.init() {
                    self.compile_expr(init, true);
                    self.emit_binding(BindingOpcode::InitVar, ident);
                }
            }
        }
        let outer_scope = self.push_declarative_scope(for_in_loop.target_scope());
        self.compile_expr(for_in_loop.target(), true);
        self.pop_declarative_scope(outer_scope);

        let early_exit = self.jump_if_null_or_undefined();
        self.emit_opcode(Opcode::CreateForInIterator);

        let start_address = self.next_opcode_location();
        self.push_loop_control_info_for_of_in_loop(label, start_address, use_expr);
        self.emit_opcode(Opcode::IncrementLoopIteration);

        self.emit_opcode(Opcode::IteratorNext);
        self.emit_opcode(Opcode::IteratorDone);
        let exit = self.jump_if_true();

        self.emit_opcode(Opcode::IteratorValue);

        let outer_scope = self.push_declarative_scope(for_in_loop.scope());

        match for_in_loop.initializer() {
            IterableLoopInitializer::Identifier(ident) => {
                let ident = ident.to_js_string(self.interner());
                self.emit_binding(BindingOpcode::InitVar, ident);
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
                    let ident = ident.to_js_string(self.interner());
                    self.emit_binding(BindingOpcode::InitVar, ident);
                }
                Binding::Pattern(pattern) => {
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar);
                }
            },
            IterableLoopInitializer::Let(declaration)
            | IterableLoopInitializer::Const(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    let ident = ident.to_js_string(self.interner());
                    self.emit_binding(BindingOpcode::InitLexical, ident);
                }
                Binding::Pattern(pattern) => {
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLexical);
                }
            },
            IterableLoopInitializer::Pattern(pattern) => {
                self.compile_declaration_pattern(pattern, BindingOpcode::SetName);
            }
        }

        self.compile_stmt(for_in_loop.body(), use_expr, true);
        self.pop_declarative_scope(outer_scope);

        self.emit(Opcode::Jump, &[Operand::U32(start_address)]);

        self.patch_jump(exit);
        self.pop_loop_control_info();

        self.iterator_close(false);

        let skip_early_exit = self.jump();
        self.patch_jump(early_exit);
        self.emit_opcode(Opcode::PushUndefined);
        self.patch_jump(skip_early_exit);
    }

    pub(crate) fn compile_for_of_loop(
        &mut self,
        for_of_loop: &ForOfLoop,
        label: Option<Sym>,
        use_expr: bool,
    ) {
        let outer_scope = self.push_declarative_scope(for_of_loop.iterable_scope());
        self.compile_expr(for_of_loop.iterable(), true);
        self.pop_declarative_scope(outer_scope);

        if for_of_loop.r#await() {
            self.emit_opcode(Opcode::GetAsyncIterator);
        } else {
            self.emit_opcode(Opcode::GetIterator);
        }

        let start_address = self.next_opcode_location();
        if for_of_loop.r#await() {
            self.push_loop_control_info_for_await_of_loop(label, start_address, use_expr);
        } else {
            self.push_loop_control_info_for_of_in_loop(label, start_address, use_expr);
        }
        self.emit_opcode(Opcode::IncrementLoopIteration);

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

        let outer_scope = self.push_declarative_scope(for_of_loop.scope());
        let handler_index = self.push_handler();

        match for_of_loop.initializer() {
            IterableLoopInitializer::Identifier(ref ident) => {
                let ident = ident.to_js_string(self.interner());
                match self.lexical_scope.set_mutable_binding(ident.clone()) {
                    Ok(binding) => {
                        let index = self.get_or_insert_binding(binding);
                        self.emit_binding_access(Opcode::DefInitVar, &index);
                    }
                    Err(BindingLocatorError::MutateImmutable) => {
                        let index = self.get_or_insert_string(ident);
                        self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                    }
                    Err(BindingLocatorError::Silent) => {
                        self.emit_opcode(Opcode::Pop);
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
                        let ident = ident.to_js_string(self.interner());
                        self.emit_binding(BindingOpcode::InitVar, ident);
                    }
                    Binding::Pattern(pattern) => {
                        self.compile_declaration_pattern(pattern, BindingOpcode::InitVar);
                    }
                }
            }
            IterableLoopInitializer::Let(declaration)
            | IterableLoopInitializer::Const(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    let ident = ident.to_js_string(self.interner());
                    self.emit_binding(BindingOpcode::InitLexical, ident);
                }
                Binding::Pattern(pattern) => {
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLexical);
                }
            },
            IterableLoopInitializer::Pattern(pattern) => {
                self.compile_declaration_pattern(pattern, BindingOpcode::SetName);
            }
        }

        self.compile_stmt(for_of_loop.body(), use_expr, true);

        {
            let exit = self.jump();
            self.patch_handler(handler_index);

            self.emit_opcode(Opcode::Exception);

            self.current_stack_value_count += 1;
            // NOTE: Capture throw of the iterator close and ignore it.
            {
                let handler_index = self.push_handler();
                self.iterator_close(for_of_loop.r#await());
                self.patch_handler(handler_index);
            }
            self.current_stack_value_count -= 1;

            self.emit_opcode(Opcode::Throw);
            self.patch_jump(exit);
        }

        self.pop_declarative_scope(outer_scope);
        self.emit(Opcode::Jump, &[Operand::U32(start_address)]);

        self.patch_jump(exit);
        self.pop_loop_control_info();

        self.iterator_close(for_of_loop.r#await());
    }

    pub(crate) fn compile_while_loop(
        &mut self,
        while_loop: &WhileLoop,
        label: Option<Sym>,
        use_expr: bool,
    ) {
        let start_address = self.next_opcode_location();
        self.emit_opcode(Opcode::IncrementLoopIteration);
        self.push_loop_control_info(label, start_address, use_expr);

        self.compile_expr(while_loop.condition(), true);
        let exit = self.jump_if_false();

        self.compile_stmt(while_loop.body(), use_expr, true);

        self.emit(Opcode::Jump, &[Operand::U32(start_address)]);

        self.patch_jump(exit);
        self.pop_loop_control_info();
    }

    pub(crate) fn compile_do_while_loop(
        &mut self,
        do_while_loop: &DoWhileLoop,
        label: Option<Sym>,
        use_expr: bool,
    ) {
        let initial_label = self.jump();

        let start_address = self.next_opcode_location();

        self.push_loop_control_info(label, start_address, use_expr);

        let condition_label_address = self.next_opcode_location();
        self.emit_opcode(Opcode::IncrementLoopIteration);
        self.compile_expr(do_while_loop.cond(), true);
        let exit = self.jump_if_false();

        self.patch_jump(initial_label);

        self.compile_stmt(do_while_loop.body(), use_expr, true);

        self.emit(Opcode::Jump, &[Operand::U32(condition_label_address)]);
        self.patch_jump(exit);

        self.pop_loop_control_info();
    }
}
