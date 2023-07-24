use crate::{
    bytecompiler::{jump_control::JumpControlInfoFlags, ByteCompiler},
    vm::{BindingOpcode, Opcode},
};
use boa_ast::{
    declaration::Binding,
    operations::bound_names,
    statement::{Catch, Finally, Try},
};

impl ByteCompiler<'_, '_> {
    /// Compile try statement.
    ///
    ///
    pub(crate) fn compile_try(&mut self, t: &Try, use_expr: bool) {
        let has_catch = t.catch().is_some();
        let has_finally = t.finally().is_some();

        // stack:
        if has_finally {
            self.push_try_with_finally_control_info(use_expr);
        }

        let try_handler = self.push_handler();

        // Compile try block
        self.compile_block(t.block(), use_expr);

        if has_finally {
            self.emit_opcode(Opcode::PushZero);
            self.emit_opcode(Opcode::PushFalse);

            // stack: false, 0
        }

        let finally = self.jump();

        self.patch_handler(try_handler);

        // If it has a finally but no catch and we are in a generator, then we still need it
        // to handle `return()` call on generators.
        let catch_handler = if has_finally && (self.in_generator || has_catch) {
            Some(self.push_handler())
        } else {
            None
        };

        self.emit_opcode(Opcode::Exception);
        if let Some(catch) = t.catch() {
            self.compile_catch_stmt(catch, has_finally, use_expr);
        } else {
            // Note: implicit !has_catch
            if self.in_generator && has_finally {
                // Is this a generator `return()` empty exception?
                //
                // This is false because when the `Exception` opcode is executed,
                // it rethrows the empty exception, so if we reached this section,
                // that means it's not an `return()` generator exception.
                self.emit_opcode(Opcode::PushFalse);
            }

            // Should we rethrow the exception?
            self.emit_opcode(Opcode::PushTrue);
        }

        if has_finally {
            if has_catch {
                self.emit_opcode(Opcode::PushZero);
                self.emit_opcode(Opcode::PushFalse);
            }

            let exit = self.jump();

            if let Some(catch_handler) = catch_handler {
                self.patch_handler(catch_handler);
            }

            // Note: implicit has_finally
            if !has_catch && self.in_generator {
                // Is this a generator `return()` empty exception?
                self.emit_opcode(Opcode::PushTrue);
            }

            // Should we rethrow the exception?
            self.emit_opcode(Opcode::PushTrue);

            self.patch_jump(exit);
        }

        self.patch_jump(finally);

        let finally_start = self.next_opcode_location();
        if let Some(finally) = t.finally() {
            self.jump_info
                .last_mut()
                .expect("there should be a try block")
                .flags |= JumpControlInfoFlags::IN_FINALLY;

            // Compile finally statement body
            self.compile_finally_stmt(finally, has_catch);
        }

        if has_finally {
            self.pop_try_with_finally_control_info(finally_start);
        }
    }

    pub(crate) fn compile_catch_stmt(&mut self, catch: &Catch, _has_finally: bool, use_expr: bool) {
        // stack: exception

        self.push_compile_environment(false);
        let push_env = self.emit_opcode_with_operand(Opcode::PushDeclarativeEnvironment);

        if let Some(binding) = catch.parameter() {
            match binding {
                Binding::Identifier(ident) => {
                    self.create_mutable_binding(*ident, false);
                    self.emit_binding(BindingOpcode::InitLet, *ident);
                }
                Binding::Pattern(pattern) => {
                    for ident in bound_names(pattern) {
                        self.create_mutable_binding(ident, false);
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLet);
                }
            }
        } else {
            self.emit_opcode(Opcode::Pop);
        }

        self.compile_block(catch.block(), use_expr);

        let env_index = self.pop_compile_environment();
        self.patch_jump_with_target(push_env, env_index);
        self.emit_opcode(Opcode::PopEnvironment);
    }

    pub(crate) fn compile_finally_stmt(&mut self, finally: &Finally, has_catch: bool) {
        // TODO: We could probably remove the Get/SetReturnValue if we check that there is no break/continues statements.
        self.emit_opcode(Opcode::GetReturnValue);
        self.compile_block(finally.block(), true);
        self.emit_opcode(Opcode::SetReturnValue);

        // Rethrow error if error happend!
        let do_not_throw_exit = self.jump_if_false();

        if has_catch {
            self.emit_opcode(Opcode::ReThrow);
        } else if self.in_generator {
            let is_generator_exit = self.jump_if_true();
            self.emit_opcode(Opcode::Throw);
            self.patch_jump(is_generator_exit);

            self.emit_opcode(Opcode::ReThrow);
        } else {
            self.emit_opcode(Opcode::Throw);
        }

        self.patch_jump(do_not_throw_exit);
    }
}
