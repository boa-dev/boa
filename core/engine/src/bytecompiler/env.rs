use super::ByteCompiler;
use crate::vm::Constant;
use crate::vm::opcode::{PopEnvironment, PushScope};
use boa_ast::scope::Scope;

impl ByteCompiler<'_> {
    /// Push either a new declarative or function scope on the environment stack.
    #[must_use]
    pub(crate) fn push_scope(&mut self, scope: &Scope) -> u32 {
        self.current_open_environments_count += 1;

        let index = self.constants.len() as u32;
        self.constants.push(Constant::Scope(scope.clone()));

        if scope.is_function() {
            self.variable_scope = scope.clone();
        }

        self.lexical_scope = scope.clone();

        index
    }

    /// Push a declarative scope.
    ///
    /// Returns the outer scope.
    #[must_use]
    pub(crate) fn push_declarative_scope(&mut self, scope: Option<&Scope>) -> Option<Scope> {
        let mut scope = scope?.clone();
        if !scope.all_bindings_local() {
            self.current_open_environments_count += 1;
            let index = self.constants.len() as u32;
            self.constants.push(Constant::Scope(scope.clone()));
            PushScope::emit(self, index.into());
        }
        std::mem::swap(&mut self.lexical_scope, &mut scope);
        Some(scope)
    }

    /// Pop a declarative scope.
    pub(crate) fn pop_declarative_scope(&mut self, scope: Option<Scope>) {
        if let Some(mut scope) = scope {
            std::mem::swap(&mut self.lexical_scope, &mut scope);
            if !scope.all_bindings_local() {
                self.current_open_environments_count -= 1;
                PopEnvironment::emit(self);
            }
        }
    }

    /// Pops the top scope.
    pub(crate) fn pop_scope(&mut self) {
        self.current_open_environments_count -= 1;
    }
}
