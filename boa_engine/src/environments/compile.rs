use std::{cell::RefCell, rc::Rc};

use crate::environments::runtime::BindingLocator;
use boa_ast::expression::Identifier;
use boa_gc::{empty_trace, Finalize, Trace};

use rustc_hash::FxHashMap;

use super::runtime::BindingLocatorError;

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
#[derive(Debug, Finalize)]
pub(crate) struct CompileTimeEnvironment {
    outer: Option<Rc<Self>>,
    environment_index: u32,
    bindings: RefCell<FxHashMap<Identifier, CompileTimeBinding>>,
    function_scope: bool,
}

// Safety: Nothing in this struct needs tracing, so this is safe.
unsafe impl Trace for CompileTimeEnvironment {
    empty_trace!();
}

impl CompileTimeEnvironment {
    /// Creates a new global compile time environment.
    pub(crate) fn new_global() -> Self {
        Self {
            outer: None,
            environment_index: 0,
            bindings: RefCell::default(),
            function_scope: true,
        }
    }

    /// Creates a new compile time environment.
    pub(crate) fn new(parent: Rc<Self>, function_scope: bool) -> Self {
        let index = parent.environment_index + 1;
        Self {
            outer: Some(parent),
            environment_index: index,
            bindings: RefCell::default(),
            function_scope,
        }
    }

    /// Check if environment has a lexical binding with the given name.
    pub(crate) fn has_lex_binding(&self, name: Identifier) -> bool {
        self.bindings
            .borrow()
            .get(&name)
            .map_or(false, |binding| binding.lex)
    }

    /// Check if the environment has a binding with the given name.
    pub(crate) fn has_binding(&self, name: Identifier) -> bool {
        self.bindings.borrow().contains_key(&name)
    }

    /// Get the binding locator for a binding with the given name.
    /// Fall back to the global environment if the binding is not found.
    pub(crate) fn get_identifier_reference(&self, name: Identifier) -> IdentifierReference {
        if let Some(binding) = self.bindings.borrow().get(&name) {
            IdentifierReference::new(
                BindingLocator::declarative(name, self.environment_index, binding.index),
                binding.lex,
            )
        } else if let Some(outer) = &self.outer {
            outer.get_identifier_reference(name)
        } else {
            IdentifierReference::new(BindingLocator::global(name), false)
        }
    }

    /// Returns the number of bindings in this environment.
    pub(crate) fn num_bindings(&self) -> u32 {
        self.bindings.borrow().len() as u32
    }

    /// Returns the index of this environment.
    pub(crate) fn environment_index(&self) -> u32 {
        self.environment_index
    }

    /// Check if the environment is a function environment.
    pub(crate) const fn is_function(&self) -> bool {
        self.function_scope
    }

    /// Check if the environment is a global environment.
    pub(crate) const fn is_global(&self) -> bool {
        self.outer.is_none()
    }

    /// Get the locator for a binding name.
    pub(crate) fn get_binding(&self, name: Identifier) -> Option<BindingLocator> {
        self.bindings
            .borrow()
            .get(&name)
            .map(|binding| BindingLocator::declarative(name, self.environment_index, binding.index))
    }

    /// Create a mutable binding.
    pub(crate) fn create_mutable_binding(
        &self,
        name: Identifier,
        function_scope: bool,
    ) -> BindingLocator {
        let binding_index = self.bindings.borrow().len() as u32;
        self.bindings.borrow_mut().insert(
            name,
            CompileTimeBinding {
                index: binding_index,
                mutable: true,
                lex: !function_scope,
                strict: false,
            },
        );
        BindingLocator::declarative(name, self.environment_index, binding_index)
    }

    /// Crate an immutable binding.
    pub(crate) fn create_immutable_binding(
        &self,
        name: Identifier,
        strict: bool,
    ) -> BindingLocator {
        let binding_index = self.bindings.borrow().len() as u32;
        self.bindings.borrow_mut().insert(
            name,
            CompileTimeBinding {
                index: binding_index,
                mutable: false,
                lex: true,
                strict,
            },
        );
        BindingLocator::declarative(name, self.environment_index, binding_index)
    }

    /// Return the binding locator for a mutable binding.
    pub(crate) fn set_mutable_binding(
        &self,
        name: Identifier,
    ) -> Result<BindingLocator, BindingLocatorError> {
        Ok(match self.bindings.borrow().get(&name) {
            Some(binding) if binding.mutable => {
                BindingLocator::declarative(name, self.environment_index, binding.index)
            }
            Some(binding) if binding.strict => return Err(BindingLocatorError::MutateImmutable),
            Some(_) => return Err(BindingLocatorError::Silent),
            None => self.outer.as_ref().map_or_else(
                || Ok(BindingLocator::global(name)),
                |outer| outer.set_mutable_binding(name),
            )?,
        })
    }

    #[cfg(feature = "annex-b")]
    /// Return the binding locator for a set operation on an existing var binding.
    pub(crate) fn set_mutable_binding_var(
        &self,
        name: Identifier,
    ) -> Result<BindingLocator, BindingLocatorError> {
        if !self.is_function() {
            return self.outer.as_ref().map_or_else(
                || Ok(BindingLocator::global(name)),
                |outer| outer.set_mutable_binding_var(name),
            );
        }

        Ok(match self.bindings.borrow().get(&name) {
            Some(binding) if binding.mutable => {
                BindingLocator::declarative(name, self.environment_index, binding.index)
            }
            Some(binding) if binding.strict => return Err(BindingLocatorError::MutateImmutable),
            Some(_) => return Err(BindingLocatorError::Silent),
            None => self.outer.as_ref().map_or_else(
                || Ok(BindingLocator::global(name)),
                |outer| outer.set_mutable_binding_var(name),
            )?,
        })
    }

    /// Gets the outer environment of this environment.
    pub(crate) fn outer(&self) -> Option<Rc<Self>> {
        self.outer.clone()
    }
}

/// A reference to an identifier in a compile time environment.
pub(crate) struct IdentifierReference {
    locator: BindingLocator,
    lexical: bool,
}

impl IdentifierReference {
    /// Create a new identifier reference.
    pub(crate) fn new(locator: BindingLocator, lexical: bool) -> Self {
        Self { locator, lexical }
    }

    /// Get the binding locator for this identifier reference.
    pub(crate) fn locator(&self) -> BindingLocator {
        self.locator
    }

    /// Check if this identifier reference is lexical.
    pub(crate) fn is_lexical(&self) -> bool {
        self.lexical
    }
}
