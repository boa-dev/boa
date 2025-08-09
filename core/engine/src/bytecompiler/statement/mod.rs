use super::jump_control::{JumpRecord, JumpRecordAction, JumpRecordKind};
use crate::bytecompiler::ByteCompiler;
use boa_ast::Statement;

mod block;
mod r#break;
mod r#continue;
mod r#if;
mod labelled;
mod r#loop;
mod switch;
mod r#try;
mod with;

impl ByteCompiler<'_> {
    /// Compiles a [`Statement`] `boa_ast` node.
    pub fn compile_stmt(&mut self, node: &Statement, use_expr: bool, root_statement: bool) {
        match node {
            Statement::Var(var) => self.compile_var_decl(var),
            Statement::If(node) => self.compile_if(node, use_expr),
            Statement::ForLoop(for_loop) => {
                self.compile_for_loop(for_loop, None, use_expr);
            }
            Statement::ForInLoop(for_in_loop) => {
                self.compile_for_in_loop(for_in_loop, None, use_expr);
            }
            Statement::ForOfLoop(for_of_loop) => {
                self.compile_for_of_loop(for_of_loop, None, use_expr);
            }
            Statement::WhileLoop(while_loop) => {
                self.compile_while_loop(while_loop, None, use_expr);
            }
            Statement::DoWhileLoop(do_while_loop) => {
                self.compile_do_while_loop(do_while_loop, None, use_expr);
            }
            Statement::Block(block) => {
                self.compile_block(block, use_expr);
            }
            Statement::Labelled(labelled) => {
                self.compile_labelled(labelled, use_expr);
            }
            Statement::Continue(node) => {
                if root_statement && (use_expr || self.jump_control_info_has_use_expr()) {
                    let value = self.register_allocator.alloc();
                    self.bytecode.emit_push_undefined(value.variable());
                    self.bytecode.emit_set_accumulator(value.variable());
                    self.register_allocator.dealloc(value);
                }
                self.compile_continue(*node, use_expr);
            }
            Statement::Break(node) => {
                if root_statement && (use_expr || self.jump_control_info_has_use_expr()) {
                    let value = self.register_allocator.alloc();
                    self.bytecode.emit_push_undefined(value.variable());
                    self.bytecode.emit_set_accumulator(value.variable());
                    self.register_allocator.dealloc(value);
                }
                self.compile_break(*node, use_expr);
            }
            Statement::Throw(throw) => {
                let mut compiler = self.position_guard(throw.target());

                let error = compiler.register_allocator.alloc();
                compiler.compile_expr(throw.target(), &error);
                compiler.bytecode.emit_throw(error.variable());
                compiler.register_allocator.dealloc(error);
            }
            Statement::Switch(switch) => {
                self.compile_switch(switch, use_expr);
            }
            Statement::Return(ret) => {
                let value = self.register_allocator.alloc();
                if let Some(expr) = ret.target() {
                    self.compile_expr(expr, &value);

                    if self.is_async_generator() {
                        self.bytecode.emit_await(value.variable());
                        let resume_kind = self.register_allocator.alloc();
                        self.pop_into_register(&resume_kind);
                        self.pop_into_register(&value);
                        self.bytecode
                            .emit_generator_next(resume_kind.variable(), value.variable());
                        self.register_allocator.dealloc(resume_kind);
                    }
                } else {
                    self.bytecode.emit_push_undefined(value.variable());
                }

                self.push_from_register(&value);
                self.register_allocator.dealloc(value);
                self.r#return(true);
            }
            Statement::Try(t) => self.compile_try(t, use_expr),
            Statement::Expression(expr) => {
                let value = self.register_allocator.alloc();
                self.compile_expr(expr, &value);
                if use_expr {
                    self.bytecode.emit_set_accumulator(value.variable());
                }
                self.register_allocator.dealloc(value);
            }
            Statement::With(with) => self.compile_with(with, use_expr),
            Statement::Empty => {}
        }
    }

    pub(crate) fn r#return(&mut self, return_value_on_stack: bool) {
        let actions = self.return_jump_record_actions();

        JumpRecord::new(
            JumpRecordKind::Return {
                return_value_on_stack,
            },
            actions,
        )
        .perform_actions(Self::DUMMY_ADDRESS, self);
    }

    fn return_jump_record_actions(&self) -> Vec<JumpRecordAction> {
        let mut actions = Vec::default();
        for (i, info) in self.jump_info.iter().enumerate().rev() {
            let count = self.jump_info_open_environment_count(i);
            actions.push(JumpRecordAction::PopEnvironments { count });

            if !info.in_finally()
                && let Some((finally_throw_flag, finally_throw_index)) = info.finally_throw
            {
                actions.push(JumpRecordAction::HandleFinally {
                    index: info.jumps.len() as u32,
                    finally_throw_flag,
                    finally_throw_index,
                });
                actions.push(JumpRecordAction::Transfer { index: i as u32 });
            }

            if info.iterator_loop() {
                actions.push(JumpRecordAction::CloseIterator {
                    r#async: info.for_await_of_loop(),
                });
            }
        }

        actions.reverse();
        actions
    }
}
