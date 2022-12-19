use boa_ast::{
    statement::{Block, Break, If, Labelled, LabelledItem, Switch},
    Statement,
};
use boa_interner::Sym;

use crate::{vm::Opcode, JsNativeError, JsResult};

use super::{ByteCompiler, JumpControlInfoKind, NodeKind};

mod r#continue;
mod r#loop;
mod r#try;

impl ByteCompiler<'_> {
    pub(crate) fn compile_if(&mut self, node: &If, configurable_globals: bool) -> JsResult<()> {
        self.compile_expr(node.cond(), true)?;
        let jelse = self.jump_if_false();

        self.compile_stmt(node.body(), false, configurable_globals)?;

        match node.else_node() {
            None => {
                self.patch_jump(jelse);
            }
            Some(else_body) => {
                let exit = self.jump();
                self.patch_jump(jelse);
                self.compile_stmt(else_body, false, configurable_globals)?;
                self.patch_jump(exit);
            }
        }

        Ok(())
    }

    pub(crate) fn compile_labelled(
        &mut self,
        labelled: &Labelled,
        use_expr: bool,
        configurable_globals: bool,
    ) -> JsResult<()> {
        match labelled.item() {
            LabelledItem::Statement(stmt) => match stmt {
                Statement::ForLoop(for_loop) => {
                    self.compile_for_loop(for_loop, Some(labelled.label()), configurable_globals)?;
                }
                Statement::ForInLoop(for_in_loop) => {
                    self.compile_for_in_loop(
                        for_in_loop,
                        Some(labelled.label()),
                        configurable_globals,
                    )?;
                }
                Statement::ForOfLoop(for_of_loop) => {
                    self.compile_for_of_loop(
                        for_of_loop,
                        Some(labelled.label()),
                        configurable_globals,
                    )?;
                }
                Statement::WhileLoop(while_loop) => {
                    self.compile_while_loop(
                        while_loop,
                        Some(labelled.label()),
                        configurable_globals,
                    )?;
                }
                Statement::DoWhileLoop(do_while_loop) => {
                    self.compile_do_while_loop(
                        do_while_loop,
                        Some(labelled.label()),
                        configurable_globals,
                    )?;
                }
                Statement::Block(block) => {
                    self.compile_block(
                        block,
                        Some(labelled.label()),
                        use_expr,
                        configurable_globals,
                    )?;
                }
                stmt => self.compile_stmt(stmt, use_expr, configurable_globals)?,
            },
            LabelledItem::Function(f) => {
                self.function(f.into(), NodeKind::Declaration, false)?;
            }
        }

        Ok(())
    }

    pub(crate) fn compile_break(&mut self, node: Break) -> JsResult<()> {
        let next = self.next_opcode_location();
        if let Some(info) = self
            .jump_info
            .last()
            .filter(|info| info.kind == JumpControlInfoKind::Try)
        {
            let in_finally = if let Some(finally_start) = info.finally_start {
                next >= finally_start.index
            } else {
                false
            };
            let in_catch_no_finally = !info.has_finally && info.in_catch;

            if in_finally {
                self.emit_opcode(Opcode::PopIfThrown);
            }
            if in_finally || in_catch_no_finally {
                self.emit_opcode(Opcode::CatchEnd2);
            } else {
                self.emit_opcode(Opcode::TryEnd);
            }
            self.emit(Opcode::FinallySetJump, &[u32::MAX]);
        }
        let label = self.jump();
        if let Some(label_name) = node.label() {
            let mut found = false;
            for info in self.jump_info.iter_mut().rev() {
                if info.label == Some(label_name) {
                    info.breaks.push(label);
                    found = true;
                    break;
                }
            }
            // TODO: promote to an early error.
            if !found {
                return Err(JsNativeError::syntax()
                    .with_message(format!(
                        "Cannot use the undeclared label '{}'",
                        self.interner().resolve_expect(label_name)
                    ))
                    .into());
            }
        } else {
            self.jump_info
                .last_mut()
                // TODO: promote to an early error.
                .ok_or_else(|| {
                    JsNativeError::syntax()
                        .with_message("unlabeled break must be inside loop or switch")
                })?
                .breaks
                .push(label);
        }

        Ok(())
    }

    pub(crate) fn compile_switch(
        &mut self,
        switch: &Switch,
        configurable_globals: bool,
    ) -> JsResult<()> {
        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
        for case in switch.cases() {
            self.create_decls(case.body(), configurable_globals);
        }
        self.emit_opcode(Opcode::LoopStart);

        let start_address = self.next_opcode_location();
        self.push_switch_control_info(None, start_address);

        self.compile_expr(switch.val(), true)?;
        let mut labels = Vec::with_capacity(switch.cases().len());
        for case in switch.cases() {
            self.compile_expr(case.condition(), true)?;
            labels.push(self.emit_opcode_with_operand(Opcode::Case));
        }

        let exit = self.emit_opcode_with_operand(Opcode::Default);

        for (label, case) in labels.into_iter().zip(switch.cases()) {
            self.patch_jump(label);
            self.compile_statement_list(case.body(), false, configurable_globals)?;
        }

        self.patch_jump(exit);
        if let Some(body) = switch.default() {
            self.create_decls(body, configurable_globals);
            self.compile_statement_list(body, false, configurable_globals)?;
        }

        self.pop_switch_control_info();

        self.emit_opcode(Opcode::LoopEnd);

        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        self.emit_opcode(Opcode::PopEnvironment);

        Ok(())
    }

    pub(crate) fn compile_block(
        &mut self,
        block: &Block,
        label: Option<Sym>,
        use_expr: bool,
        configurable_globals: bool,
    ) -> JsResult<()> {
        if let Some(label) = label {
            let next = self.next_opcode_location();
            self.push_labelled_block_control_info(label, next);
        }

        self.context.push_compile_time_environment(false);
        let push_env = self.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
        self.create_decls(block.statement_list(), configurable_globals);
        self.compile_statement_list(block.statement_list(), use_expr, configurable_globals)?;
        let (num_bindings, compile_environment) = self.context.pop_compile_time_environment();
        let index_compile_environment = self.push_compile_environment(compile_environment);
        self.patch_jump_with_target(push_env.0, num_bindings as u32);
        self.patch_jump_with_target(push_env.1, index_compile_environment as u32);

        if label.is_some() {
            self.pop_labelled_block_control_info();
        }

        self.emit_opcode(Opcode::PopEnvironment);
        Ok(())
    }
}
