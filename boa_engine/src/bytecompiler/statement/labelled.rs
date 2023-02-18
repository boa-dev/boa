use crate::{
    bytecompiler::{ByteCompiler, NodeKind},
    vm::Opcode,
};
use boa_ast::{
    statement::{Labelled, LabelledItem},
    Statement,
};

impl ByteCompiler<'_, '_> {
    /// Compile a [`Labelled`] `boa_ast` node
    pub(crate) fn compile_labelled(
        &mut self,
        labelled: &Labelled,
        use_expr: bool,
        configurable_globals: bool,
    ) {
        let labelled_loc = self.next_opcode_location();
        let end_label = self.emit_opcode_with_operand(Opcode::LabelledStart);
        self.push_labelled_control_info(labelled.label(), labelled_loc);

        match labelled.item() {
            LabelledItem::Statement(stmt) => match stmt {
                Statement::ForLoop(for_loop) => {
                    self.compile_for_loop(for_loop, Some(labelled.label()), configurable_globals);
                }
                Statement::ForInLoop(for_in_loop) => {
                    self.compile_for_in_loop(
                        for_in_loop,
                        Some(labelled.label()),
                        configurable_globals,
                    );
                }
                Statement::ForOfLoop(for_of_loop) => {
                    self.compile_for_of_loop(
                        for_of_loop,
                        Some(labelled.label()),
                        configurable_globals,
                    );
                }
                Statement::WhileLoop(while_loop) => {
                    self.compile_while_loop(
                        while_loop,
                        Some(labelled.label()),
                        configurable_globals,
                    );
                }
                Statement::DoWhileLoop(do_while_loop) => {
                    self.compile_do_while_loop(
                        do_while_loop,
                        Some(labelled.label()),
                        configurable_globals,
                    );
                }
                stmt => self.compile_stmt(stmt, use_expr, configurable_globals),
            },
            LabelledItem::Function(f) => {
                self.function(f.into(), NodeKind::Declaration, false);
            }
        }

        let labelled_end = self.next_opcode_location();
        self.patch_jump_with_target(end_label, labelled_end);
        self.pop_labelled_control_info();
        self.emit_opcode(Opcode::LabelledEnd);
    }
}
