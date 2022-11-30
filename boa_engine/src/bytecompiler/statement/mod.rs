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

pub(crate) use r#continue::compile_continue;
pub(crate) use r#loop::*;
pub(crate) use r#try::compile_try;

pub(crate) fn compile_if(
    byte_compiler: &mut ByteCompiler<'_>,
    node: &If,
    configurable_globals: bool,
) -> JsResult<()> {
    byte_compiler.compile_expr(node.cond(), true)?;
    let jelse = byte_compiler.jump_if_false();

    byte_compiler.compile_stmt(node.body(), false, configurable_globals)?;

    match node.else_node() {
        None => {
            byte_compiler.patch_jump(jelse);
        }
        Some(else_body) => {
            let exit = byte_compiler.jump();
            byte_compiler.patch_jump(jelse);
            byte_compiler.compile_stmt(else_body, false, configurable_globals)?;
            byte_compiler.patch_jump(exit);
        }
    }

    Ok(())
}

pub(crate) fn compile_labeled(
    byte_compiler: &mut ByteCompiler<'_>,
    labelled: &Labelled,
    use_expr: bool,
    configurable_globals: bool,
) -> JsResult<()> {
    match labelled.item() {
        LabelledItem::Statement(stmt) => match stmt {
            Statement::ForLoop(for_loop) => {
                compile_for_loop(
                    byte_compiler,
                    for_loop,
                    Some(labelled.label()),
                    configurable_globals,
                )?;
            }
            Statement::ForInLoop(for_in_loop) => {
                compile_for_in_loop(
                    byte_compiler,
                    for_in_loop,
                    Some(labelled.label()),
                    configurable_globals,
                )?;
            }
            Statement::ForOfLoop(for_of_loop) => {
                compile_for_of_loop(
                    byte_compiler,
                    for_of_loop,
                    Some(labelled.label()),
                    configurable_globals,
                )?;
            }
            Statement::WhileLoop(while_loop) => {
                compile_while_loop(
                    byte_compiler,
                    while_loop,
                    Some(labelled.label()),
                    configurable_globals,
                )?;
            }
            Statement::DoWhileLoop(do_while_loop) => {
                compile_do_while_loop(
                    byte_compiler,
                    do_while_loop,
                    Some(labelled.label()),
                    configurable_globals,
                )?;
            }
            Statement::Block(block) => {
                compile_block(
                    byte_compiler,
                    block,
                    Some(labelled.label()),
                    use_expr,
                    configurable_globals,
                )?;
            }
            stmt => byte_compiler.compile_stmt(stmt, use_expr, configurable_globals)?,
        },
        LabelledItem::Function(f) => {
            byte_compiler.function(f.into(), NodeKind::Declaration, false)?;
        }
    }

    Ok(())
}

pub(crate) fn compile_break(byte_compiler: &mut ByteCompiler<'_>, node: Break) -> JsResult<()> {
    let next = byte_compiler.next_opcode_location();
    if let Some(info) = byte_compiler
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
            byte_compiler.emit_opcode(Opcode::PopIfThrown);
        }
        if in_finally || in_catch_no_finally {
            byte_compiler.emit_opcode(Opcode::CatchEnd2);
        } else {
            byte_compiler.emit_opcode(Opcode::TryEnd);
        }
        byte_compiler.emit(Opcode::FinallySetJump, &[u32::MAX]);
    }
    let label = byte_compiler.jump();
    if let Some(label_name) = node.label() {
        let mut found = false;
        for info in byte_compiler.jump_info.iter_mut().rev() {
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
                    byte_compiler.interner().resolve_expect(label_name)
                ))
                .into());
        }
    } else {
        byte_compiler
            .jump_info
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
    byte_compiler: &mut ByteCompiler<'_>,
    switch: &Switch,
    configurable_globals: bool,
) -> JsResult<()> {
    byte_compiler.context.push_compile_time_environment(false);
    let push_env = byte_compiler.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
    for case in switch.cases() {
        byte_compiler.create_decls(case.body(), configurable_globals);
    }
    byte_compiler.emit_opcode(Opcode::LoopStart);

    let start_address = byte_compiler.next_opcode_location();
    byte_compiler.push_switch_control_info(None, start_address);

    byte_compiler.compile_expr(switch.val(), true)?;
    let mut labels = Vec::with_capacity(switch.cases().len());
    for case in switch.cases() {
        byte_compiler.compile_expr(case.condition(), true)?;
        labels.push(byte_compiler.emit_opcode_with_operand(Opcode::Case));
    }

    let exit = byte_compiler.emit_opcode_with_operand(Opcode::Default);

    for (label, case) in labels.into_iter().zip(switch.cases()) {
        byte_compiler.patch_jump(label);
        byte_compiler.compile_statement_list(case.body(), false, configurable_globals)?;
    }

    byte_compiler.patch_jump(exit);
    if let Some(body) = switch.default() {
        byte_compiler.create_decls(body, configurable_globals);
        byte_compiler.compile_statement_list(body, false, configurable_globals)?;
    }

    byte_compiler.pop_switch_control_info();

    byte_compiler.emit_opcode(Opcode::LoopEnd);

    let (num_bindings, compile_environment) = byte_compiler.context.pop_compile_time_environment();
    let index_compile_environment = byte_compiler.push_compile_environment(compile_environment);
    byte_compiler.patch_jump_with_target(push_env.0, num_bindings as u32);
    byte_compiler.patch_jump_with_target(push_env.1, index_compile_environment as u32);
    byte_compiler.emit_opcode(Opcode::PopEnvironment);

    Ok(())
}

pub(crate) fn compile_block(
    byte_compiler: &mut ByteCompiler<'_>,
    block: &Block,
    label: Option<Sym>,
    use_expr: bool,
    configurable_globals: bool,
) -> JsResult<()> {
    if let Some(label) = label {
        let next = byte_compiler.next_opcode_location();
        byte_compiler.push_labelled_block_control_info(label, next);
    }

    byte_compiler.context.push_compile_time_environment(false);
    let push_env = byte_compiler.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
    byte_compiler.create_decls(block.statement_list(), configurable_globals);
    byte_compiler.compile_statement_list(block.statement_list(), use_expr, configurable_globals)?;
    let (num_bindings, compile_environment) = byte_compiler.context.pop_compile_time_environment();
    let index_compile_environment = byte_compiler.push_compile_environment(compile_environment);
    byte_compiler.patch_jump_with_target(push_env.0, num_bindings as u32);
    byte_compiler.patch_jump_with_target(push_env.1, index_compile_environment as u32);

    if label.is_some() {
        byte_compiler.pop_labelled_block_control_info();
    }

    byte_compiler.emit_opcode(Opcode::PopEnvironment);
    Ok(())
}
