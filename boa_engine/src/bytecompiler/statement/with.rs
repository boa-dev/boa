use crate::{bytecompiler::ByteCompiler, vm::Opcode};
use boa_ast::statement::With;

impl ByteCompiler<'_, '_> {
    /// Compile a [`With`] `boa_ast` node
    pub(crate) fn compile_with(&mut self, with: &With, use_expr: bool) {
        self.compile_expr(with.expression(), true);
        self.push_compile_environment(false);
        self.emit_opcode(Opcode::PushObjectEnvironment);

        if !with.statement().returns_value() {
            self.emit_opcode(Opcode::PushUndefined);
        }
        self.compile_stmt(with.statement(), true);

        self.pop_compile_environment();
        self.emit_opcode(Opcode::PopEnvironment);

        if !use_expr {
            self.emit_opcode(Opcode::Pop);
        }
    }
}
