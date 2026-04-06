use crate::bytecompiler::ByteCompiler;
#[cfg(feature = "experimental")]
use crate::bytecompiler::jump_control::JumpControlInfoFlags;
use boa_ast::statement::Block;
#[cfg(feature = "experimental")]
use boa_ast::{
    declaration::LexicalDeclaration,
    operations::{LexicallyScopedDeclaration, lexically_scoped_declarations},
};

impl ByteCompiler<'_> {
    /// Compile a [`Block`] `boa_ast` node
    pub(crate) fn compile_block(&mut self, block: &Block, use_expr: bool) {
        let scope = self.push_declarative_scope(block.scope());
        self.block_declaration_instantiation(block);

        // Count how many `using` bindings are in this block (statically known at compile time)
        #[cfg(feature = "experimental")]
        let using_count: u32 = lexically_scoped_declarations(block)
            .iter()
            .filter_map(|decl| {
                if let LexicallyScopedDeclaration::LexicalDeclaration(
                    LexicalDeclaration::Using(u) | LexicalDeclaration::AwaitUsing(u),
                ) = decl
                {
                    Some(u.as_ref().len() as u32)
                } else {
                    None
                }
            })
            .sum();

        #[cfg(feature = "experimental")]
        let has_using = using_count > 0;

        #[cfg(not(feature = "experimental"))]
        let has_using = false;

        if has_using {
            #[cfg(feature = "experimental")]
            {
                // Blocks with `using` declarations need try-finally semantics
                // Allocate registers for finally control flow (same pattern as try-finally)
                let finally_re_throw = self.register_allocator.alloc();
                let finally_jump_index = self.register_allocator.alloc();

                self.bytecode.emit_store_true(finally_re_throw.variable());
                self.bytecode.emit_store_zero(finally_jump_index.variable());

                // Push jump control info to handle break/continue/return through disposal
                self.push_try_with_finally_control_info(
                    &finally_re_throw,
                    &finally_jump_index,
                    use_expr,
                );

                // Push exception handler to catch any exceptions during block execution
                let handler = self.push_handler();

                // Compile the block body (this includes the `using` declarations)
                self.compile_statement_list(block.statement_list(), use_expr, true);

                // Normal exit: mark that we don't need to re-throw
                self.bytecode.emit_store_false(finally_re_throw.variable());

                let finally_jump = self.jump();

                // Exception path: patch the handler
                self.patch_handler(handler);

                // Push a second handler for exceptions during exception handling
                let catch_handler = self.push_handler();
                let error = self.register_allocator.alloc();
                self.bytecode.emit_exception(error.variable());
                self.bytecode.emit_store_true(finally_re_throw.variable());

                let no_throw = self.jump();
                self.patch_handler(catch_handler);

                self.patch_jump(no_throw);
                self.patch_jump(finally_jump);

                // Finally block: dispose resources
                let finally_start = self.next_opcode_location();
                self.jump_info
                    .last_mut()
                    .expect("there should be a jump control info")
                    .flags |= JumpControlInfoFlags::IN_FINALLY;

                // Save accumulator (disposal might modify it, similar to compile_finally_stmt)
                let value = self.register_allocator.alloc();
                self.bytecode
                    .emit_set_register_from_accumulator(value.variable());

                // Emit disposal logic
                self.bytecode.emit_dispose_resources(using_count.into());

                // Restore accumulator
                self.bytecode.emit_set_accumulator(value.variable());
                self.register_allocator.dealloc(value);

                // Re-throw if there was an exception
                let do_not_throw_exit = self.jump_if_false(&finally_re_throw);
                self.bytecode.emit_throw(error.variable());
                self.register_allocator.dealloc(error);
                self.patch_jump(do_not_throw_exit);

                // Pop jump control info (this handles break/continue/return via jump table)
                self.pop_try_with_finally_control_info(finally_start);

                self.register_allocator.dealloc(finally_re_throw);
                self.register_allocator.dealloc(finally_jump_index);
            }
        } else {
            // Normal block compilation (no using declarations)
            self.compile_statement_list(block.statement_list(), use_expr, true);
        }

        self.pop_declarative_scope(scope);
    }
}
