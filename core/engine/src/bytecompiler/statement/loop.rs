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
                ForLoopInitializer::Expression(expr) => {
                    let value = self.register_allocator.alloc();
                    self.compile_expr(expr, &value);
                    self.register_allocator.dealloc(value);
                }
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
            let mut values = Vec::with_capacity(let_binding_indices.len());
            for index in let_binding_indices {
                let value = self.register_allocator.alloc();
                self.emit_binding_access(Opcode::GetName, index, &value);
                values.push((index, value));
            }

            if let Some(index) = scope_index {
                self.emit_opcode(Opcode::PopEnvironment);
                self.emit_with_varying_operand(Opcode::PushScope, *index);
            }

            for (index, value) in values {
                self.emit_binding_access(Opcode::PutLexicalValue, index, &value);
                self.register_allocator.dealloc(value);
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
            let mut values = Vec::with_capacity(let_binding_indices.len());
            for index in let_binding_indices {
                let value = self.register_allocator.alloc();
                self.emit_binding_access(Opcode::GetName, index, &value);
                values.push((index, value));
            }

            if let Some(index) = scope_index {
                self.emit_opcode(Opcode::PopEnvironment);
                self.emit_with_varying_operand(Opcode::PushScope, *index);
            }

            for (index, value) in values {
                self.emit_binding_access(Opcode::PutLexicalValue, index, &value);
                self.register_allocator.dealloc(value);
            }
        }

        self.emit_opcode(Opcode::IncrementLoopIteration);

        if let Some(final_expr) = for_loop.final_expr() {
            let value = self.register_allocator.alloc();
            self.compile_expr(final_expr, &value);
            self.register_allocator.dealloc(value);
        }

        self.patch_jump(initial_jump);

        let value = self.register_allocator.alloc();
        if let Some(condition) = for_loop.condition() {
            self.compile_expr(condition, &value);
        } else {
            self.push_true(&value);
        }
        let exit = self.jump_if_false(&value);
        self.register_allocator.dealloc(value);

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
                    let value = self.register_allocator.alloc();
                    self.compile_expr(init, &value);
                    self.emit_binding(BindingOpcode::InitVar, ident, &value);
                    self.register_allocator.dealloc(value);
                }
            }
        }
        let outer_scope = self.push_declarative_scope(for_in_loop.target_scope());
        let value = self.register_allocator.alloc();
        self.compile_expr(for_in_loop.target(), &value);
        self.pop_declarative_scope(outer_scope);

        let early_exit = self.jump_if_null_or_undefined(&value);

        self.emit(Opcode::CreateForInIterator, &[Operand::Register(&value)]);

        self.register_allocator.dealloc(value);

        let start_address = self.next_opcode_location();
        self.push_loop_control_info_for_of_in_loop(label, start_address, use_expr);
        self.emit_opcode(Opcode::IncrementLoopIteration);

        self.emit_opcode(Opcode::IteratorNext);

        let value = self.register_allocator.alloc();
        self.emit(Opcode::IteratorDone, &[Operand::Register(&value)]);
        let exit = self.jump_if_true(&value);
        self.emit(Opcode::IteratorValue, &[Operand::Register(&value)]);

        let outer_scope = self.push_declarative_scope(for_in_loop.scope());

        match for_in_loop.initializer() {
            IterableLoopInitializer::Identifier(ident) => {
                let ident = ident.to_js_string(self.interner());
                self.emit_binding(BindingOpcode::InitVar, ident, &value);
            }
            IterableLoopInitializer::Access(access) => {
                self.access_set(Access::Property { access }, |_| &value);
            }
            IterableLoopInitializer::Var(declaration) => match declaration.binding() {
                Binding::Identifier(ident) => {
                    let ident = ident.to_js_string(self.interner());
                    self.emit_binding(BindingOpcode::InitVar, ident, &value);
                }
                Binding::Pattern(pattern) => {
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitVar, &value);
                }
            },
            IterableLoopInitializer::Let(declaration)
            | IterableLoopInitializer::Const(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    let ident = ident.to_js_string(self.interner());
                    self.emit_binding(BindingOpcode::InitLexical, ident, &value);
                }
                Binding::Pattern(pattern) => {
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLexical, &value);
                }
            },
            IterableLoopInitializer::Pattern(pattern) => {
                self.compile_declaration_pattern(pattern, BindingOpcode::SetName, &value);
            }
        }

        self.register_allocator.dealloc(value);

        self.compile_stmt(for_in_loop.body(), use_expr, true);
        self.pop_declarative_scope(outer_scope);

        self.emit(Opcode::Jump, &[Operand::U32(start_address)]);

        self.patch_jump(exit);
        self.pop_loop_control_info();

        self.iterator_close(false);

        let skip_early_exit = self.jump();
        self.patch_jump(early_exit);
        let value = self.register_allocator.alloc();
        self.push_undefined(&value);
        self.register_allocator.dealloc(value);
        self.patch_jump(skip_early_exit);
    }

    pub(crate) fn compile_for_of_loop(
        &mut self,
        for_of_loop: &ForOfLoop,
        label: Option<Sym>,
        use_expr: bool,
    ) {
        let outer_scope = self.push_declarative_scope(for_of_loop.iterable_scope());
        let object = self.register_allocator.alloc();
        self.compile_expr(for_of_loop.iterable(), &object);
        self.pop_declarative_scope(outer_scope);

        if for_of_loop.r#await() {
            self.emit(Opcode::GetAsyncIterator, &[Operand::Register(&object)]);
        } else {
            self.emit(Opcode::GetIterator, &[Operand::Register(&object)]);
        }

        self.register_allocator.dealloc(object);

        let start_address = self.next_opcode_location();
        if for_of_loop.r#await() {
            self.push_loop_control_info_for_await_of_loop(label, start_address, use_expr);
        } else {
            self.push_loop_control_info_for_of_in_loop(label, start_address, use_expr);
        }
        self.emit_opcode(Opcode::IncrementLoopIteration);

        self.emit_opcode(Opcode::IteratorNext);
        if for_of_loop.r#await() {
            let value = self.register_allocator.alloc();
            self.emit(Opcode::IteratorResult, &[Operand::Register(&value)]);
            self.emit(Opcode::Await, &[Operand::Register(&value)]);
            let resume_kind = self.register_allocator.alloc();
            self.pop_into_register(&resume_kind);
            self.pop_into_register(&value);

            self.emit(
                Opcode::IteratorFinishAsyncNext,
                &[Operand::Register(&resume_kind), Operand::Register(&value)],
            );
            self.emit(
                Opcode::GeneratorNext,
                &[Operand::Register(&resume_kind), Operand::Register(&value)],
            );
            self.register_allocator.dealloc(value);
            self.register_allocator.dealloc(resume_kind);
        }

        let value = self.register_allocator.alloc();
        self.emit(Opcode::IteratorDone, &[Operand::Register(&value)]);
        let exit = self.jump_if_true(&value);
        self.emit(Opcode::IteratorValue, &[Operand::Register(&value)]);

        let outer_scope = self.push_declarative_scope(for_of_loop.scope());
        let handler_index = self.push_handler();

        match for_of_loop.initializer() {
            IterableLoopInitializer::Identifier(ref ident) => {
                let ident = ident.to_js_string(self.interner());
                match self.lexical_scope.set_mutable_binding(ident.clone()) {
                    Ok(binding) => {
                        let index = self.get_or_insert_binding(binding);
                        self.emit_binding_access(Opcode::DefInitVar, &index, &value);
                    }
                    Err(BindingLocatorError::MutateImmutable) => {
                        let index = self.get_or_insert_string(ident);
                        self.emit_with_varying_operand(Opcode::ThrowMutateImmutable, index);
                    }
                    Err(BindingLocatorError::Silent) => {}
                }
            }
            IterableLoopInitializer::Access(access) => {
                self.access_set(Access::Property { access }, |_| &value);
            }
            IterableLoopInitializer::Var(declaration) => {
                // ignore initializers since those aren't allowed on for-of loops.
                assert!(declaration.init().is_none());
                match declaration.binding() {
                    Binding::Identifier(ident) => {
                        let ident = ident.to_js_string(self.interner());
                        self.emit_binding(BindingOpcode::InitVar, ident, &value);
                    }
                    Binding::Pattern(pattern) => {
                        self.compile_declaration_pattern(pattern, BindingOpcode::InitVar, &value);
                    }
                }
            }
            IterableLoopInitializer::Let(declaration)
            | IterableLoopInitializer::Const(declaration) => match declaration {
                Binding::Identifier(ident) => {
                    let ident = ident.to_js_string(self.interner());
                    self.emit_binding(BindingOpcode::InitLexical, ident, &value);
                }
                Binding::Pattern(pattern) => {
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLexical, &value);
                }
            },
            IterableLoopInitializer::Pattern(pattern) => {
                self.compile_declaration_pattern(pattern, BindingOpcode::SetName, &value);
            }
        }

        self.register_allocator.dealloc(value);

        self.compile_stmt(for_of_loop.body(), use_expr, true);

        {
            let exit = self.jump();
            self.patch_handler(handler_index);

            let error = self.register_allocator.alloc();
            self.emit(Opcode::Exception, &[Operand::Register(&error)]);

            // NOTE: Capture throw of the iterator close and ignore it.
            let handler_index = self.push_handler();
            self.iterator_close(for_of_loop.r#await());
            self.patch_handler(handler_index);

            self.emit(Opcode::Throw, &[Operand::Register(&error)]);
            self.register_allocator.dealloc(error);
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

        let value = self.register_allocator.alloc();
        self.compile_expr(while_loop.condition(), &value);
        let exit = self.jump_if_false(&value);
        self.register_allocator.dealloc(value);

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

        let value = self.register_allocator.alloc();
        self.compile_expr(do_while_loop.cond(), &value);
        let exit = self.jump_if_false(&value);
        self.register_allocator.dealloc(value);

        self.patch_jump(initial_label);

        self.compile_stmt(do_while_loop.body(), use_expr, true);

        self.emit(Opcode::Jump, &[Operand::U32(condition_label_address)]);
        self.patch_jump(exit);

        self.pop_loop_control_info();
    }
}
