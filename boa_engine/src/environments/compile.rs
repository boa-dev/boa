use crate::{
    environments::runtime::BindingLocator, property::PropertyDescriptor, Context, JsString, JsValue,
};
use boa_ast::expression::Identifier;
use boa_gc::{Cell, Finalize, Gc, Trace};

use rustc_hash::FxHashMap;

/// A compile time binding represents a binding at bytecode compile time in a [`CompileTimeEnvironment`].
///
/// It contains the binding index and a flag to indicate if this is a mutable binding or not.
#[derive(Debug)]
struct CompileTimeBinding {
    index: usize,
    mutable: bool,
    lex: bool,
    strict: bool,
}

/// A compile time environment maps bound identifiers to their binding positions.
///
/// A compile time environment also indicates, if it is a function environment.
#[derive(Debug, Finalize, Trace)]
pub(crate) struct CompileTimeEnvironment {
    outer: Option<Gc<Cell<Self>>>,
    environment_index: usize,
    #[unsafe_ignore_trace]
    bindings: FxHashMap<Identifier, CompileTimeBinding>,
    function_scope: bool,
}

impl CompileTimeEnvironment {
    /// Crate a new global compile time environment.
    #[inline]
    pub(crate) fn new_global() -> Self {
        Self {
            outer: None,
            environment_index: 0,
            bindings: FxHashMap::default(),
            function_scope: true,
        }
    }

    /// Check if environment has a lexical binding with the given name.
    #[inline]
    pub(crate) fn has_lex_binding(&self, name: Identifier) -> bool {
        self.bindings
            .get(&name)
            .map_or(false, |binding| binding.lex)
    }

    /// Returns the number of bindings in this environment.
    #[inline]
    pub(crate) fn num_bindings(&self) -> usize {
        self.bindings.len()
    }

    /// Check if the environment is a function environment.
    #[inline]
    pub(crate) fn is_function(&self) -> bool {
        self.function_scope
    }

    /// Get the locator for a binding name.
    #[inline]
    pub(crate) fn get_binding(&self, name: Identifier) -> Option<BindingLocator> {
        self.bindings
            .get(&name)
            .map(|binding| BindingLocator::declarative(name, self.environment_index, binding.index))
    }

    /// Get the locator for a binding name in this and all outer environments.
    #[inline]
    pub(crate) fn get_binding_recursive(&self, name: Identifier) -> BindingLocator {
        if let Some(binding) = self.bindings.get(&name) {
            BindingLocator::declarative(name, self.environment_index, binding.index)
        } else if let Some(outer) = &self.outer {
            outer.borrow().get_binding_recursive(name)
        } else {
            BindingLocator::global(name)
        }
    }

    /// Check if a binding name exists in this and all outer environments.
    #[inline]
    pub(crate) fn has_binding_recursive(&self, name: Identifier) -> bool {
        if self.bindings.contains_key(&name) {
            true
        } else if let Some(outer) = &self.outer {
            outer.borrow().has_binding_recursive(name)
        } else {
            false
        }
    }

    /// Create a mutable binding.
    ///
    /// If the binding is a function scope binding and this is a declarative environment, try the outer environment.
    #[inline]
    pub(crate) fn create_mutable_binding(
        &mut self,
        name: Identifier,
        function_scope: bool,
    ) -> bool {
        if let Some(outer) = &self.outer {
            if !function_scope || self.function_scope {
                if !self.bindings.contains_key(&name) {
                    let binding_index = self.bindings.len();
                    self.bindings.insert(
                        name,
                        CompileTimeBinding {
                            index: binding_index,
                            mutable: true,
                            lex: !function_scope,
                            strict: false,
                        },
                    );
                }
                true
            } else {
                return outer
                    .borrow_mut()
                    .create_mutable_binding(name, function_scope);
            }
        } else if function_scope {
            false
        } else {
            if !self.bindings.contains_key(&name) {
                let binding_index = self.bindings.len();
                self.bindings.insert(
                    name,
                    CompileTimeBinding {
                        index: binding_index,
                        mutable: true,
                        lex: !function_scope,
                        strict: false,
                    },
                );
            }
            true
        }
    }

    /// Crate an immutable binding.
    #[inline]
    pub(crate) fn create_immutable_binding(&mut self, name: Identifier, strict: bool) {
        let binding_index = self.bindings.len();
        self.bindings.insert(
            name,
            CompileTimeBinding {
                index: binding_index,
                mutable: false,
                lex: true,
                strict,
            },
        );
    }

    /// Return the binding locator for a mutable binding with the given binding name and scope.
    #[inline]
    pub(crate) fn initialize_mutable_binding(
        &self,
        name: Identifier,
        function_scope: bool,
    ) -> BindingLocator {
        if let Some(outer) = &self.outer {
            if function_scope && !self.function_scope {
                return outer
                    .borrow()
                    .initialize_mutable_binding(name, function_scope);
            }
            if let Some(binding) = self.bindings.get(&name) {
                BindingLocator::declarative(name, self.environment_index, binding.index)
            } else {
                outer
                    .borrow()
                    .initialize_mutable_binding(name, function_scope)
            }
        } else if let Some(binding) = self.bindings.get(&name) {
            BindingLocator::declarative(name, self.environment_index, binding.index)
        } else {
            BindingLocator::global(name)
        }
    }

    /// Return the binding locator for an immutable binding.
    ///
    /// # Panics
    ///
    /// Panics if the binding is not in the current environment.
    #[inline]
    pub(crate) fn initialize_immutable_binding(&self, name: Identifier) -> BindingLocator {
        let binding = self.bindings.get(&name).expect("binding must exist");
        BindingLocator::declarative(name, self.environment_index, binding.index)
    }

    /// Return the binding locator for a mutable binding.
    #[inline]
    pub(crate) fn set_mutable_binding_recursive(&self, name: Identifier) -> BindingLocator {
        match self.bindings.get(&name) {
            Some(binding) if binding.mutable => {
                BindingLocator::declarative(name, self.environment_index, binding.index)
            }
            Some(binding) if binding.strict => BindingLocator::mutate_immutable(name),
            Some(_) => BindingLocator::silent(name),
            None => {
                if let Some(outer) = &self.outer {
                    outer.borrow().set_mutable_binding_recursive(name)
                } else {
                    BindingLocator::global(name)
                }
            }
        }
    }
}

impl Context {
    /// Push either a new declarative or function environment on the compile time environment stack.
    ///
    /// Note: This function only works at bytecode compile time!
    #[inline]
    pub(crate) fn push_compile_time_environment(&mut self, function_scope: bool) {
        let environment_index = self.realm.compile_env.borrow().environment_index + 1;
        let outer = self.realm.compile_env.clone();

        self.realm.compile_env = Gc::new(Cell::new(CompileTimeEnvironment {
            outer: Some(outer),
            environment_index,
            bindings: FxHashMap::default(),
            function_scope,
        }));
    }

    /// Pop the last compile time environment from the stack.
    ///
    /// Note: This function only works at bytecode compile time!
    ///
    /// # Panics
    ///
    /// Panics if there are no more environments that can be pop'ed.
    #[inline]
    pub(crate) fn pop_compile_time_environment(
        &mut self,
    ) -> (usize, Gc<Cell<CompileTimeEnvironment>>) {
        let current_env_borrow = self.realm.compile_env.borrow();
        if let Some(outer) = &current_env_borrow.outer {
            let outer_clone = outer.clone();
            let num_bindings = current_env_borrow.num_bindings();
            drop(current_env_borrow);
            let current = self.realm.compile_env.clone();
            self.realm.compile_env = outer_clone;
            (num_bindings, current)
        } else {
            panic!("cannot pop global environment")
        }
    }

    /// Get the number of bindings for the current compile time environment.
    ///
    /// Note: This function only works at bytecode compile time!
    ///
    /// # Panics
    ///
    /// Panics if there are no environments on the compile time environment stack.
    #[inline]
    pub(crate) fn get_binding_number(&self) -> usize {
        self.realm.compile_env.borrow().num_bindings()
    }

    /// Get the binding locator of the binding at bytecode compile time.
    ///
    /// Note: This function only works at bytecode compile time!
    #[inline]
    pub(crate) fn get_binding_value(&self, name: Identifier) -> BindingLocator {
        self.realm.compile_env.borrow().get_binding_recursive(name)
    }

    /// Return if a declarative binding exists at bytecode compile time.
    /// This does not include bindings on the global object.
    ///
    /// Note: This function only works at bytecode compile time!
    #[inline]
    pub(crate) fn has_binding(&self, name: Identifier) -> bool {
        self.realm.compile_env.borrow().has_binding_recursive(name)
    }

    /// Create a mutable binding at bytecode compile time.
    /// This function returns a syntax error, if the binding is a redeclaration.
    ///
    /// Note: This function only works at bytecode compile time!
    ///
    /// # Panics
    ///
    /// Panics if the global environment is not function scoped.
    #[inline]
    pub(crate) fn create_mutable_binding(
        &mut self,
        name: Identifier,
        function_scope: bool,
        configurable: bool,
    ) {
        if !self
            .realm
            .compile_env
            .borrow_mut()
            .create_mutable_binding(name, function_scope)
        {
            let name_str = self
                .interner()
                .resolve_expect(name.sym())
                .into_common::<JsString>(false);
            let desc = self
                .realm
                .global_property_map
                .string_property_map()
                .get(&name_str);
            if desc.is_none() {
                self.global_bindings_mut().insert(
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
    ///
    /// Note: This function only works at bytecode compile time!
    #[inline]
    pub(crate) fn initialize_mutable_binding(
        &self,
        name: Identifier,
        function_scope: bool,
    ) -> BindingLocator {
        self.realm
            .compile_env
            .borrow()
            .initialize_mutable_binding(name, function_scope)
    }

    /// Create an immutable binding at bytecode compile time.
    /// This function returns a syntax error, if the binding is a redeclaration.
    ///
    /// Note: This function only works at bytecode compile time!
    ///
    /// # Panics
    ///
    /// Panics if the global environment does not exist.
    #[inline]
    pub(crate) fn create_immutable_binding(&mut self, name: Identifier, strict: bool) {
        self.realm
            .compile_env
            .borrow_mut()
            .create_immutable_binding(name, strict);
    }

    /// Initialize an immutable binding at bytecode compile time and return it's binding locator.
    ///
    /// Note: This function only works at bytecode compile time!
    ///
    /// # Panics
    ///
    /// Panics if the global environment does not exist or a the binding was not created on the current environment.
    #[inline]
    pub(crate) fn initialize_immutable_binding(&self, name: Identifier) -> BindingLocator {
        self.realm
            .compile_env
            .borrow()
            .initialize_immutable_binding(name)
    }

    /// Return the binding locator for a set operation on an existing binding.
    ///
    /// Note: This function only works at bytecode compile time!
    #[inline]
    pub(crate) fn set_mutable_binding(&self, name: Identifier) -> BindingLocator {
        self.realm
            .compile_env
            .borrow()
            .set_mutable_binding_recursive(name)
    }
}
