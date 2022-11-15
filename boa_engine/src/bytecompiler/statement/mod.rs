use boa_ast::{
    statement::{If, Labelled, LabelledItem},
    Statement,
};

use crate::JsResult;

use super::{ByteCompiler, NodeKind};

mod kontinue;

pub(crate) use kontinue::compile_continue;

pub(crate) fn compile_if<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
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

pub(crate) fn compile_labeled<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
    labelled: &Labelled,
    use_expr: bool,
    configurable_globals: bool,
) -> JsResult<()> {
    match labelled.item() {
        LabelledItem::Statement(stmt) => match stmt {
            Statement::ForLoop(for_loop) => {
                byte_compiler.compile_for_loop(
                    for_loop,
                    Some(labelled.label()),
                    configurable_globals,
                )?;
            }
            Statement::ForInLoop(for_in_loop) => {
                byte_compiler.compile_for_in_loop(
                    for_in_loop,
                    Some(labelled.label()),
                    configurable_globals,
                )?;
            }
            Statement::ForOfLoop(for_of_loop) => {
                byte_compiler.compile_for_of_loop(
                    for_of_loop,
                    Some(labelled.label()),
                    configurable_globals,
                )?;
            }
            Statement::WhileLoop(while_loop) => {
                byte_compiler.compile_while_loop(
                    while_loop,
                    Some(labelled.label()),
                    configurable_globals,
                )?;
            }
            Statement::DoWhileLoop(do_while_loop) => {
                byte_compiler.compile_do_while_loop(
                    do_while_loop,
                    Some(labelled.label()),
                    configurable_globals,
                )?;
            }
            Statement::Block(block) => {
                byte_compiler.compile_block(
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
