use boa_ast::scope::Scope;

use super::ByteCompiler;

impl ByteCompiler<'_> {
    /// Push either a new declarative or function scope on the environment stack.
    #[must_use]
    pub(crate) fn push_scope(&mut self, scope: &Scope) -> u32 {
        self.current_open_environments_count += 1;

        let index = self.constants.len() as u32;
        self.constants
            .push(crate::vm::Constant::Scope(scope.clone()));

        if scope.is_function() {
            self.variable_scope = scope.clone();
        }

        self.lexical_scope = scope.clone();

        index
    }

    /// Pops the top scope.
    pub(crate) fn pop_scope(&mut self) {
        self.current_open_environments_count -= 1;
    }
}
