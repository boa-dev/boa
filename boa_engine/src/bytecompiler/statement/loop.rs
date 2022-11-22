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
    JsResult,
};

pub(crate) fn compile_for_loop<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
    for_loop: &ForLoop,
    label: Option<Sym>,
    configurable_globals: bool,
) -> JsResult<()> {
    byte_compiler.context.push_compile_time_environment(false);
    let push_env = byte_compiler.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

    if let Some(init) = for_loop.init() {
        match init {
            ForLoopInitializer::Expression(expr) => byte_compiler.compile_expr(expr, false)?,
            ForLoopInitializer::Var(decl) => {
                byte_compiler.create_decls_from_var_decl(decl, configurable_globals);
                byte_compiler.compile_var_decl(decl)?;
            }
            ForLoopInitializer::Lexical(decl) => {
                byte_compiler.create_decls_from_lexical_decl(decl);
                byte_compiler.compile_lexical_decl(decl)?;
            }
        }
    }

    byte_compiler.emit_opcode(Opcode::LoopStart);
    let initial_jump = byte_compiler.jump();

    let start_address = byte_compiler.next_opcode_location();
    byte_compiler.push_loop_control_info(label, start_address);

    byte_compiler.emit_opcode(Opcode::LoopContinue);
    if let Some(final_expr) = for_loop.final_expr() {
        byte_compiler.compile_expr(final_expr, false)?;
    }

    byte_compiler.patch_jump(initial_jump);

    if let Some(condition) = for_loop.condition() {
        byte_compiler.compile_expr(condition, true)?;
    } else {
        byte_compiler.emit_opcode(Opcode::PushTrue);
    }
    let exit = byte_compiler.jump_if_false();

    byte_compiler.compile_stmt(for_loop.body(), false, configurable_globals)?;

    byte_compiler.emit(Opcode::Jump, &[start_address]);

    byte_compiler.patch_jump(exit);
    byte_compiler.pop_loop_control_info();
    byte_compiler.emit_opcode(Opcode::LoopEnd);

    let (num_bindings, compile_environment) = byte_compiler.context.pop_compile_time_environment();
    let index_compile_environment = byte_compiler.push_compile_environment(compile_environment);
    byte_compiler.patch_jump_with_target(push_env.0, num_bindings as u32);
    byte_compiler.patch_jump_with_target(push_env.1, index_compile_environment as u32);
    byte_compiler.emit_opcode(Opcode::PopEnvironment);
    Ok(())
}

pub(crate) fn compile_for_in_loop<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
    for_in_loop: &ForInLoop,
    label: Option<Sym>,
    configurable_globals: bool,
) -> JsResult<()> {
    let init_bound_names = bound_names(for_in_loop.initializer());
    if init_bound_names.is_empty() {
        byte_compiler.compile_expr(for_in_loop.target(), true)?;
    } else {
        byte_compiler.context.push_compile_time_environment(false);
        let push_env =
            byte_compiler.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        for name in init_bound_names {
            byte_compiler
                .context
                .create_mutable_binding(name, false, false);
        }
        byte_compiler.compile_expr(for_in_loop.target(), true)?;

        let (num_bindings, compile_environment) =
            byte_compiler.context.pop_compile_time_environment();
        let index_compile_environment = byte_compiler.push_compile_environment(compile_environment);
        byte_compiler.patch_jump_with_target(push_env.0, num_bindings as u32);
        byte_compiler.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        byte_compiler.emit_opcode(Opcode::PopEnvironment);
    }

    let early_exit = byte_compiler.emit_opcode_with_operand(Opcode::ForInLoopInitIterator);

    byte_compiler.emit_opcode(Opcode::LoopStart);
    let start_address = byte_compiler.next_opcode_location();
    byte_compiler.push_loop_control_info_for_of_in_loop(label, start_address);
    byte_compiler.emit_opcode(Opcode::LoopContinue);

    byte_compiler.context.push_compile_time_environment(false);
    let push_env = byte_compiler.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
    let exit = byte_compiler.emit_opcode_with_operand(Opcode::ForInLoopNext);

    match for_in_loop.initializer() {
        IterableLoopInitializer::Identifier(ident) => {
            byte_compiler
                .context
                .create_mutable_binding(*ident, true, true);
            let binding = byte_compiler.context.set_mutable_binding(*ident);
            let index = byte_compiler.get_or_insert_binding(binding);
            byte_compiler.emit(Opcode::DefInitVar, &[index]);
        }
        IterableLoopInitializer::Access(access) => {
            byte_compiler.access_set(
                Access::Property { access },
                false,
                ByteCompiler::access_set_top_of_stack_expr_fn,
            )?;
        }
        IterableLoopInitializer::Var(declaration) => match declaration {
            Binding::Identifier(ident) => {
                byte_compiler
                    .context
                    .create_mutable_binding(*ident, true, configurable_globals);
                byte_compiler.emit_binding(BindingOpcode::InitVar, *ident);
            }
            Binding::Pattern(pattern) => {
                for ident in bound_names(pattern) {
                    byte_compiler
                        .context
                        .create_mutable_binding(ident, true, false);
                }
                byte_compiler.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
            }
        },
        IterableLoopInitializer::Let(declaration) => match declaration {
            Binding::Identifier(ident) => {
                byte_compiler
                    .context
                    .create_mutable_binding(*ident, false, false);
                byte_compiler.emit_binding(BindingOpcode::InitLet, *ident);
            }
            Binding::Pattern(pattern) => {
                for ident in bound_names(pattern) {
                    byte_compiler
                        .context
                        .create_mutable_binding(ident, false, false);
                }
                byte_compiler.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
            }
        },
        IterableLoopInitializer::Const(declaration) => match declaration {
            Binding::Identifier(ident) => {
                byte_compiler.context.create_immutable_binding(*ident, true);
                byte_compiler.emit_binding(BindingOpcode::InitConst, *ident);
            }
            Binding::Pattern(pattern) => {
                for ident in bound_names(pattern) {
                    byte_compiler.context.create_immutable_binding(ident, true);
                }
                byte_compiler.compile_declaration_pattern(pattern, BindingOpcode::InitConst)?;
            }
        },
        IterableLoopInitializer::Pattern(pattern) => {
            for ident in bound_names(pattern) {
                byte_compiler
                    .context
                    .create_mutable_binding(ident, true, true);
            }
            byte_compiler.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
        }
    }

    byte_compiler.compile_stmt(for_in_loop.body(), false, configurable_globals)?;

    let (num_bindings, compile_environment) = byte_compiler.context.pop_compile_time_environment();
    let index_compile_environment = byte_compiler.push_compile_environment(compile_environment);
    byte_compiler.patch_jump_with_target(push_env.0, num_bindings as u32);
    byte_compiler.patch_jump_with_target(push_env.1, index_compile_environment as u32);
    byte_compiler.emit_opcode(Opcode::PopEnvironment);

    byte_compiler.emit(Opcode::Jump, &[start_address]);

    byte_compiler.patch_jump(exit);
    byte_compiler.pop_loop_control_info();
    byte_compiler.emit_opcode(Opcode::LoopEnd);
    byte_compiler.emit_opcode(Opcode::IteratorClose);

    byte_compiler.patch_jump(early_exit);
    Ok(())
}

pub(crate) fn compile_for_of_loop<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
    for_of_loop: &ForOfLoop,
    label: Option<Sym>,
    configurable_globals: bool,
) -> JsResult<()> {
    let init_bound_names = bound_names(for_of_loop.initializer());
    if init_bound_names.is_empty() {
        byte_compiler.compile_expr(for_of_loop.iterable(), true)?;
    } else {
        byte_compiler.context.push_compile_time_environment(false);
        let push_env =
            byte_compiler.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        for name in init_bound_names {
            byte_compiler
                .context
                .create_mutable_binding(name, false, false);
        }
        byte_compiler.compile_expr(for_of_loop.iterable(), true)?;

        let (num_bindings, compile_environment) =
            byte_compiler.context.pop_compile_time_environment();
        let index_compile_environment = byte_compiler.push_compile_environment(compile_environment);
        byte_compiler.patch_jump_with_target(push_env.0, num_bindings as u32);
        byte_compiler.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        byte_compiler.emit_opcode(Opcode::PopEnvironment);
    }

    if for_of_loop.r#await() {
        byte_compiler.emit_opcode(Opcode::InitIteratorAsync);
    } else {
        byte_compiler.emit_opcode(Opcode::InitIterator);
    }

    byte_compiler.emit_opcode(Opcode::LoopStart);
    let start_address = byte_compiler.next_opcode_location();
    byte_compiler.push_loop_control_info_for_of_in_loop(label, start_address);
    byte_compiler.emit_opcode(Opcode::LoopContinue);

    byte_compiler.context.push_compile_time_environment(false);
    let push_env = byte_compiler.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

    let exit = if for_of_loop.r#await() {
        byte_compiler.emit_opcode(Opcode::ForAwaitOfLoopIterate);
        byte_compiler.emit_opcode(Opcode::Await);
        byte_compiler.emit_opcode(Opcode::GeneratorNext);
        byte_compiler.emit_opcode_with_operand(Opcode::ForAwaitOfLoopNext)
    } else {
        byte_compiler.emit_opcode_with_operand(Opcode::ForInLoopNext)
    };

    match for_of_loop.initializer() {
        IterableLoopInitializer::Identifier(ref ident) => {
            byte_compiler
                .context
                .create_mutable_binding(*ident, true, true);
            let binding = byte_compiler.context.set_mutable_binding(*ident);
            let index = byte_compiler.get_or_insert_binding(binding);
            byte_compiler.emit(Opcode::DefInitVar, &[index]);
        }
        IterableLoopInitializer::Access(access) => {
            byte_compiler.access_set(
                Access::Property { access },
                false,
                ByteCompiler::access_set_top_of_stack_expr_fn,
            )?;
        }
        IterableLoopInitializer::Var(declaration) => match declaration {
            Binding::Identifier(ident) => {
                byte_compiler
                    .context
                    .create_mutable_binding(*ident, true, false);
                byte_compiler.emit_binding(BindingOpcode::InitVar, *ident);
            }
            Binding::Pattern(pattern) => {
                for ident in bound_names(pattern) {
                    byte_compiler
                        .context
                        .create_mutable_binding(ident, true, false);
                }
                byte_compiler.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
            }
        },
        IterableLoopInitializer::Let(declaration) => match declaration {
            Binding::Identifier(ident) => {
                byte_compiler
                    .context
                    .create_mutable_binding(*ident, false, false);
                byte_compiler.emit_binding(BindingOpcode::InitLet, *ident);
            }
            Binding::Pattern(pattern) => {
                for ident in bound_names(pattern) {
                    byte_compiler
                        .context
                        .create_mutable_binding(ident, false, false);
                }
                byte_compiler.compile_declaration_pattern(pattern, BindingOpcode::InitLet)?;
            }
        },
        IterableLoopInitializer::Const(declaration) => match declaration {
            Binding::Identifier(ident) => {
                byte_compiler.context.create_immutable_binding(*ident, true);
                byte_compiler.emit_binding(BindingOpcode::InitConst, *ident);
            }
            Binding::Pattern(pattern) => {
                for ident in bound_names(pattern) {
                    byte_compiler.context.create_immutable_binding(ident, true);
                }
                byte_compiler.compile_declaration_pattern(pattern, BindingOpcode::InitConst)?;
            }
        },
        IterableLoopInitializer::Pattern(pattern) => {
            for ident in bound_names(pattern) {
                byte_compiler
                    .context
                    .create_mutable_binding(ident, true, true);
            }
            byte_compiler.compile_declaration_pattern(pattern, BindingOpcode::InitVar)?;
        }
    }

    byte_compiler.compile_stmt(for_of_loop.body(), false, configurable_globals)?;

    let (num_bindings, compile_environment) = byte_compiler.context.pop_compile_time_environment();
    let index_compile_environment = byte_compiler.push_compile_environment(compile_environment);
    byte_compiler.patch_jump_with_target(push_env.0, num_bindings as u32);
    byte_compiler.patch_jump_with_target(push_env.1, index_compile_environment as u32);
    byte_compiler.emit_opcode(Opcode::PopEnvironment);

    byte_compiler.emit(Opcode::Jump, &[start_address]);

    byte_compiler.patch_jump(exit);
    byte_compiler.pop_loop_control_info();
    byte_compiler.emit_opcode(Opcode::LoopEnd);
    byte_compiler.emit_opcode(Opcode::IteratorClose);
    Ok(())
}

pub(crate) fn compile_while_loop<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
    while_loop: &WhileLoop,
    label: Option<Sym>,
    configurable_globals: bool,
) -> JsResult<()> {
    byte_compiler.emit_opcode(Opcode::LoopStart);
    let start_address = byte_compiler.next_opcode_location();
    byte_compiler.push_loop_control_info(label, start_address);
    byte_compiler.emit_opcode(Opcode::LoopContinue);

    byte_compiler.compile_expr(while_loop.condition(), true)?;
    let exit = byte_compiler.jump_if_false();
    byte_compiler.compile_stmt(while_loop.body(), false, configurable_globals)?;
    byte_compiler.emit(Opcode::Jump, &[start_address]);
    byte_compiler.patch_jump(exit);

    byte_compiler.pop_loop_control_info();
    byte_compiler.emit_opcode(Opcode::LoopEnd);
    Ok(())
}

pub(crate) fn compile_do_while_loop<'b>(
    byte_compiler: &mut ByteCompiler<'b>,
    do_while_loop: &DoWhileLoop,
    label: Option<Sym>,
    configurable_globals: bool,
) -> JsResult<()> {
    byte_compiler.emit_opcode(Opcode::LoopStart);
    let initial_label = byte_compiler.jump();

    let start_address = byte_compiler.next_opcode_location();
    byte_compiler.push_loop_control_info(label, start_address);
    byte_compiler.emit_opcode(Opcode::LoopContinue);

    let condition_label_address = byte_compiler.next_opcode_location();
    byte_compiler.compile_expr(do_while_loop.cond(), true)?;
    let exit = byte_compiler.jump_if_false();

    byte_compiler.patch_jump(initial_label);

    byte_compiler.compile_stmt(do_while_loop.body(), false, configurable_globals)?;
    byte_compiler.emit(Opcode::Jump, &[condition_label_address]);
    byte_compiler.patch_jump(exit);

    byte_compiler.pop_loop_control_info();
    byte_compiler.emit_opcode(Opcode::LoopEnd);
    Ok(())
}
