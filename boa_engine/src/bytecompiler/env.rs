use boa_ast::expression::Identifier;
use boa_gc::{Gc, GcRefCell};

use crate::{
    environments::{BindingLocator, CompileTimeEnvironment},
    property::PropertyDescriptor,
    JsString, JsValue,
};

use super::ByteCompiler;

/// Info returned by the [`ByteCompiler::pop_compile_environment`] method.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopEnvironmentInfo {
    /// Number of bindings declared.
    pub(crate) num_bindings: usize,
    /// Index in the compile time envs array.
    pub(crate) index: usize,
}

impl ByteCompiler<'_, '_> {
    /// Push either a new declarative or function environment on the compile time environment stack.
    pub(crate) fn push_compile_environment(&mut self, function_scope: bool) {
        self.current_environment = Gc::new(GcRefCell::new(CompileTimeEnvironment::new(
            self.current_environment.clone(),
            function_scope,
        )));
    }

    /// Pops the top compile time environment and returns its index and number of bindings.
    #[track_caller]
    pub(crate) fn pop_compile_environment(&mut self) -> PopEnvironmentInfo {
        let index = self.compile_environments.len();
        self.compile_environments
            .push(self.current_environment.clone());

        let (num_bindings, outer) = {
            let env = self.current_environment.borrow();
            (
                env.num_bindings(),
                env.outer().expect("cannot pop the global environment"),
            )
        };
        self.current_environment = outer;

        PopEnvironmentInfo {
            num_bindings,
            index,
        }
    }

    /// Get the binding locator of the binding at bytecode compile time.
    pub(crate) fn get_binding_value(&self, name: Identifier) -> BindingLocator {
        self.current_environment
            .borrow()
            .get_binding_recursive(name)
    }

    /// Return if a declarative binding exists at bytecode compile time.
    /// This does not include bindings on the global object.
    pub(crate) fn has_binding(&self, name: Identifier) -> bool {
        self.current_environment
            .borrow()
            .has_binding_recursive(name)
    }

    /// Create a mutable binding at bytecode compile time.
    /// This function returns a syntax error, if the binding is a redeclaration.
    ///
    /// # Panics
    ///
    /// Panics if the global environment is not function scoped.
    pub(crate) fn create_mutable_binding(
        &mut self,
        name: Identifier,
        function_scope: bool,
        configurable: bool,
    ) {
        if !self
            .current_environment
            .borrow_mut()
            .create_mutable_binding(name, function_scope)
        {
            let name_str = self
                .context
                .interner()
                .resolve_expect(name.sym())
                .into_common::<JsString>(false);

            let global_obj = self.context.global_object();

            // TODO: defer global initialization to execution time.
            if !global_obj
                .has_own_property(name_str.clone(), self.context)
                .unwrap_or_default()
            {
                global_obj.borrow_mut().insert(
                    name_str,
                    PropertyDescriptor::builder()
                        .value(JsValue::Undefined)
                        .writable(true)
                        .enumerable(true)
                        .configurable(configurable)
                        .build(),
                );
            }
        }
    }

    /// Initialize a mutable binding at bytecode compile time and return it's binding locator.
    pub(crate) fn initialize_mutable_binding(
        &self,
        name: Identifier,
        function_scope: bool,
    ) -> BindingLocator {
        self.current_environment
            .borrow()
            .initialize_mutable_binding(name, function_scope)
    }

    /// Create an immutable binding at bytecode compile time.
    /// This function returns a syntax error, if the binding is a redeclaration.
    ///
    /// # Panics
    ///
    /// Panics if the global environment does not exist.
    pub(crate) fn create_immutable_binding(&mut self, name: Identifier, strict: bool) {
        self.current_environment
            .borrow_mut()
            .create_immutable_binding(name, strict);
    }

    /// Initialize an immutable binding at bytecode compile time and return it's binding locator.
    ///
    /// # Panics
    ///
    /// Panics if the global environment does not exist or a the binding was not created on the current environment.
    pub(crate) fn initialize_immutable_binding(&self, name: Identifier) -> BindingLocator {
        self.current_environment
            .borrow()
            .initialize_immutable_binding(name)
    }

    /// Return the binding locator for a set operation on an existing binding.
    pub(crate) fn set_mutable_binding(&self, name: Identifier) -> BindingLocator {
        self.current_environment
            .borrow()
            .set_mutable_binding_recursive(name)
    }
}
