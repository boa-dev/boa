use boa_ast::{declaration::Binding, operations::bound_names, statement::Try};

use crate::{
    bytecompiler::{ByteCompiler, Label},
    vm::{BindingOpcode, Opcode},
    JsResult,
};

pub(crate) fn compile_try(
    byte_compiler: &mut ByteCompiler<'_>,
    t: &Try,
    use_expr: bool,
    configurable_globals: bool,
) -> JsResult<()> {
    byte_compiler.push_try_control_info(t.finally().is_some());
    let try_start = byte_compiler.next_opcode_location();
    byte_compiler.emit(Opcode::TryStart, &[ByteCompiler::DUMMY_ADDRESS, 0]);
    byte_compiler.context.push_compile_time_environment(false);
    let push_env = byte_compiler.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

    byte_compiler.create_decls(t.block().statement_list(), configurable_globals);
    byte_compiler.compile_statement_list(
        t.block().statement_list(),
        use_expr,
        configurable_globals,
    )?;

    let (num_bindings, compile_environment) = byte_compiler.context.pop_compile_time_environment();
    let index_compile_environment = byte_compiler.push_compile_environment(compile_environment);
    byte_compiler.patch_jump_with_target(push_env.0, num_bindings as u32);
    byte_compiler.patch_jump_with_target(push_env.1, index_compile_environment as u32);
    byte_compiler.emit_opcode(Opcode::PopEnvironment);
    byte_compiler.emit_opcode(Opcode::TryEnd);

    let finally = byte_compiler.jump();
    byte_compiler.patch_jump(Label { index: try_start });

    if let Some(catch) = t.catch() {
        byte_compiler.push_try_control_info_catch_start();
        let catch_start = if t.finally().is_some() {
            Some(byte_compiler.emit_opcode_with_operand(Opcode::CatchStart))
        } else {
            None
        };
        byte_compiler.context.push_compile_time_environment(false);
        let push_env =
            byte_compiler.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);
        if let Some(binding) = catch.parameter() {
            match binding {
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
            }
        } else {
            byte_compiler.emit_opcode(Opcode::Pop);
        }

        byte_compiler.create_decls(catch.block().statement_list(), configurable_globals);
        byte_compiler.compile_statement_list(
            catch.block().statement_list(),
            use_expr,
            configurable_globals,
        )?;

        let (num_bindings, compile_environment) =
            byte_compiler.context.pop_compile_time_environment();
        let index_compile_environment = byte_compiler.push_compile_environment(compile_environment);
        byte_compiler.patch_jump_with_target(push_env.0, num_bindings as u32);
        byte_compiler.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        byte_compiler.emit_opcode(Opcode::PopEnvironment);
        if let Some(catch_start) = catch_start {
            byte_compiler.emit_opcode(Opcode::CatchEnd);
            byte_compiler.patch_jump(catch_start);
        } else {
            byte_compiler.emit_opcode(Opcode::CatchEnd2);
        }
    }

    byte_compiler.patch_jump(finally);

    if let Some(finally) = t.finally() {
        byte_compiler.emit_opcode(Opcode::FinallyStart);
        let finally_start_address = byte_compiler.next_opcode_location();
        byte_compiler.push_try_control_info_finally_start(Label {
            index: finally_start_address,
        });
        byte_compiler.patch_jump_with_target(
            Label {
                index: try_start + 4,
            },
            finally_start_address,
        );

        byte_compiler.context.push_compile_time_environment(false);
        let push_env =
            byte_compiler.emit_opcode_with_two_operands(Opcode::PushDeclarativeEnvironment);

        byte_compiler.create_decls(finally.block().statement_list(), configurable_globals);
        byte_compiler.compile_statement_list(
            finally.block().statement_list(),
            false,
            configurable_globals,
        )?;

        let (num_bindings, compile_environment) =
            byte_compiler.context.pop_compile_time_environment();
        let index_compile_environment = byte_compiler.push_compile_environment(compile_environment);
        byte_compiler.patch_jump_with_target(push_env.0, num_bindings as u32);
        byte_compiler.patch_jump_with_target(push_env.1, index_compile_environment as u32);
        byte_compiler.emit_opcode(Opcode::PopEnvironment);

        byte_compiler.emit_opcode(Opcode::FinallyEnd);
        byte_compiler.pop_try_control_info(Some(finally_start_address));
    } else {
        byte_compiler.pop_try_control_info(None);
    }

    Ok(())
}
