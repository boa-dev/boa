use crate::environments::runtime::BindingLocator;
use boa_ast::expression::Identifier;
use boa_gc::{Finalize, Gc, GcRefCell, Trace};

use rustc_hash::FxHashMap;

/// A compile time binding represents a binding at bytecode compile time in a [`CompileTimeEnvironment`].
///
/// It contains the binding index and a flag to indicate if this is a mutable binding or not.
#[derive(Debug)]
struct CompileTimeBinding {
    index: u32,
    mutable: bool,
    lex: bool,
    strict: bool,
}

/// A compile time environment maps bound identifiers to their binding positions.
///
/// A compile time environment also indicates, if it is a function environment.
#[derive(Debug, Finalize, Trace)]
pub(crate) struct CompileTimeEnvironment {
    outer: Option<Gc<GcRefCell<Self>>>,
    environment_index: u32,
    #[unsafe_ignore_trace]
    bindings: FxHashMap<Identifier, CompileTimeBinding>,
    function_scope: bool,
}

impl CompileTimeEnvironment {
    /// Creates a new global compile time environment.
    pub(crate) fn new_global() -> Self {
        Self {
            outer: None,
            environment_index: 0,
            bindings: FxHashMap::default(),
            function_scope: true,
        }
    }

    /// Creates a new compile time environment.
    pub(crate) fn new(parent: Gc<GcRefCell<Self>>, function_scope: bool) -> Self {
        let index = parent.borrow().environment_index + 1;
        Self {
            outer: Some(parent),
            environment_index: index,
            bindings: FxHashMap::default(),
            function_scope,
        }
    }

    /// Check if environment has a lexical binding with the given name.
    pub(crate) fn has_lex_binding(&self, name: Identifier) -> bool {
        self.bindings
            .get(&name)
            .map_or(false, |binding| binding.lex)
    }

    #[cfg(feature = "annex-b")]
    /// Check if the environment has a binding with the given name.
    pub(crate) fn has_binding(&self, name: Identifier) -> bool {
        self.bindings.contains_key(&name)
    }

    /// Checks if `name` is a lexical binding.
    pub(crate) fn is_lex_binding(&self, name: Identifier) -> bool {
        if let Some(binding) = self.bindings.get(&name) {
            binding.lex
        } else if let Some(outer) = &self.outer {
            outer.borrow().is_lex_binding(name)
        } else {
            false
        }
    }

    /// Returns the number of bindings in this environment.
    pub(crate) fn num_bindings(&self) -> u32 {
        self.bindings.len() as u32
    }

    /// Check if the environment is a function environment.
    pub(crate) const fn is_function(&self) -> bool {
        self.function_scope
    }

    /// Get the locator for a binding name.
    pub(crate) fn get_binding(&self, name: Identifier) -> Option<BindingLocator> {
        self.bindings
            .get(&name)
            .map(|binding| BindingLocator::declarative(name, self.environment_index, binding.index))
    }

    /// Get the locator for a binding name in this and all outer environments.
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
    pub(crate) fn has_binding_recursive(&self, name: Identifier) -> bool {
        if self.bindings.contains_key(&name) {
            true
        } else if let Some(outer) = &self.outer {
            outer.borrow().has_binding_recursive(name)
        } else {
            false
        }
    }

    /// Check if a binding name exists in a environment.
    /// If strict is `false` check until a function scope is reached.
    pub(crate) fn has_binding_eval(&self, name: Identifier, strict: bool) -> bool {
        let exists = self.bindings.contains_key(&name);
        if exists || strict {
            return exists;
        }
        if self.function_scope {
            return false;
        }
        if let Some(outer) = &self.outer {
            outer.borrow().has_binding_eval(name, false)
        } else {
            false
        }
    }

    #[cfg(feature = "annex-b")]
    /// Check if a binding name exists in a environment.
    /// Stop when a function scope is reached.
    pub(crate) fn has_binding_until_var(&self, name: Identifier) -> bool {
        if self.function_scope {
            return false;
        }
        if self.bindings.contains_key(&name) {
            return true;
        }
        if let Some(outer) = &self.outer {
            outer.borrow().has_binding_until_var(name)
        } else {
            false
        }
    }

    /// Create a mutable binding.
    ///
    /// If the binding is a function scope binding and this is a declarative environment, try the outer environment.
    pub(crate) fn create_mutable_binding(
        &mut self,
        name: Identifier,
        function_scope: bool,
    ) -> bool {
        if let Some(outer) = &self.outer {
            if !function_scope || self.function_scope {
                if !self.bindings.contains_key(&name) {
                    let binding_index = self.bindings.len() as u32;
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
                let binding_index = self.bindings.len() as u32;
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
    pub(crate) fn create_immutable_binding(&mut self, name: Identifier, strict: bool) {
        let binding_index = self.bindings.len() as u32;
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
            self.bindings.get(&name).map_or_else(
                || {
                    outer
                        .borrow()
                        .initialize_mutable_binding(name, function_scope)
                },
                |binding| BindingLocator::declarative(name, self.environment_index, binding.index),
            )
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
    pub(crate) fn initialize_immutable_binding(&self, name: Identifier) -> BindingLocator {
        let binding = self.bindings.get(&name).expect("binding must exist");
        BindingLocator::declarative(name, self.environment_index, binding.index)
    }

    /// Return the binding locator for a mutable binding.
    pub(crate) fn set_mutable_binding_recursive(
        &self,
        name: Identifier,
    ) -> Result<BindingLocator, ()> {
        Ok(match self.bindings.get(&name) {
            Some(binding) if binding.mutable => {
                BindingLocator::declarative(name, self.environment_index, binding.index)
            }
            Some(binding) if binding.strict => return Err(()),
            Some(_) => BindingLocator::silent(name),
            None => self.outer.as_ref().map_or_else(
                || Ok(BindingLocator::global(name)),
                |outer| outer.borrow().set_mutable_binding_recursive(name),
            )?,
        })
    }

    #[cfg(feature = "annex-b")]
    /// Return the binding locator for a set operation on an existing var binding.
    pub(crate) fn set_mutable_binding_var_recursive(
        &self,
        name: Identifier,
    ) -> Result<BindingLocator, ()> {
        if !self.is_function() {
            return self.outer.as_ref().map_or_else(
                || Ok(BindingLocator::global(name)),
                |outer| outer.borrow().set_mutable_binding_var_recursive(name),
            );
        }

        Ok(match self.bindings.get(&name) {
            Some(binding) if binding.mutable => {
                BindingLocator::declarative(name, self.environment_index, binding.index)
            }
            Some(binding) if binding.strict => return Err(()),
            Some(_) => BindingLocator::silent(name),
            None => self.outer.as_ref().map_or_else(
                || Ok(BindingLocator::global(name)),
                |outer| outer.borrow().set_mutable_binding_var_recursive(name),
            )?,
        })
    }

    /// Gets the outer environment of this environment.
    pub(crate) fn outer(&self) -> Option<Gc<GcRefCell<Self>>> {
        self.outer.clone()
    }

    /// Gets the environment index of this environment.
    pub(crate) const fn environment_index(&self) -> u32 {
        self.environment_index
    }
}
