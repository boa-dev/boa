use crate::{bytecompiler::ByteCompiler, vm::Opcode};
use boa_ast::statement::With;

impl ByteCompiler<'_> {
    /// Compile a [`With`] `boa_ast` node
    pub(crate) fn compile_with(&mut self, with: &With, use_expr: bool) {
        self.compile_expr(with.expression(), true);

        let outer_scope = self.lexical_scope.clone();
        let _ = self.push_scope(with.scope());
        self.emit_opcode(Opcode::PushObjectEnvironment);

        let in_with = self.in_with;
        self.in_with = true;
        self.compile_stmt(with.statement(), use_expr, true);
        self.in_with = in_with;

        self.pop_scope();
        self.lexical_scope = outer_scope;
        self.emit_opcode(Opcode::PopEnvironment);
    }
}
