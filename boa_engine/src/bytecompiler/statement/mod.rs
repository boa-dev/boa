use crate::{bytecompiler::ByteCompiler, vm::Opcode};

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

impl ByteCompiler<'_, '_> {
    /// Compiles a [`Statement`] `boa_ast` node.
    pub fn compile_stmt(&mut self, node: &Statement, use_expr: bool) {
        match node {
            Statement::Var(var) => self.compile_var_decl(var),
            Statement::If(node) => self.compile_if(node, use_expr),
            Statement::ForLoop(for_loop) => {
                self.compile_for_loop(for_loop, None);
            }
            Statement::ForInLoop(for_in_loop) => {
                self.compile_for_in_loop(for_in_loop, None);
            }
            Statement::ForOfLoop(for_of_loop) => {
                self.compile_for_of_loop(for_of_loop, None);
            }
            Statement::WhileLoop(while_loop) => {
                self.compile_while_loop(while_loop, None);
            }
            Statement::DoWhileLoop(do_while_loop) => {
                self.compile_do_while_loop(do_while_loop, None);
            }
            Statement::Block(block) => {
                self.compile_block(block, use_expr);
            }
            Statement::Labelled(labelled) => {
                self.compile_labelled(labelled, use_expr);
            }
            Statement::Continue(node) => self.compile_continue(*node),
            Statement::Break(node) => self.compile_break(*node),
            Statement::Throw(throw) => {
                self.compile_expr(throw.target(), true);
                self.emit(Opcode::Throw, &[]);
            }
            Statement::Switch(switch) => {
                self.compile_switch(switch);
            }
            Statement::Return(ret) => {
                if let Some(expr) = ret.target() {
                    self.compile_expr(expr, true);
                } else {
                    self.emit(Opcode::PushUndefined, &[]);
                }
                self.emit(Opcode::Return, &[]);
            }
            Statement::Try(t) => self.compile_try(t, use_expr),
            Statement::Expression(expr) => self.compile_expr(expr, use_expr),
            Statement::With(with) => self.compile_with(with),
            Statement::Empty => {}
        }
    }
}
