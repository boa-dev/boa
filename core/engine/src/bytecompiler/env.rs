use boa_ast::scope::Scope;

use crate::vm::{Constant, Opcode};

use super::ByteCompiler;

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
            self.emit_with_varying_operand(Opcode::PushScope, index);
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
                self.emit_opcode(Opcode::PopEnvironment);
            }
        }
    }

    /// Pops the top scope.
    pub(crate) fn pop_scope(&mut self) {
        self.current_open_environments_count -= 1;
    }
}
