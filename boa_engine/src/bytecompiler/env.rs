use super::ByteCompiler;
use crate::environments::CompileTimeEnvironment;
use std::rc::Rc;

impl ByteCompiler<'_, '_> {
    /// Push either a new declarative or function environment on the compile time environment stack.
    #[must_use]
    pub(crate) fn push_compile_environment(&mut self, function_scope: bool) -> u32 {
        self.current_open_environments_count += 1;

        let env = Rc::new(CompileTimeEnvironment::new(
            self.lexical_environment.clone(),
            function_scope,
        ));

        let index = self.compile_environments.len() as u32;
        self.compile_environments.push(env.clone());

        if function_scope {
            self.variable_environment = env.clone();
        }

        self.lexical_environment = env;

        index
    }

    /// Pops the top compile time environment and returns its index in the compile time environments array.
    pub(crate) fn pop_compile_environment(&mut self) {
        self.current_open_environments_count -= 1;
    }
}
