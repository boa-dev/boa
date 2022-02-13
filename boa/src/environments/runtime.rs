use crate::{
    gc::{Finalize, Gc, Trace},
    Context, JsResult, JsValue,
};
use boa_interner::Sym;
use gc::GcCell;

/// A declarative environment holds the bindings values at runtime.
///
/// Bindings are stored in a fixed size list of optional values.
/// If a binding is not initialized, the value is `None`.
///
/// Optionally, an environment can hold a `this` value.
/// The `this` value is present only if the environment is a function environment.
#[derive(Debug, Trace, Finalize)]
pub(crate) struct DeclarativeEnvironment {
    bindings: GcCell<Vec<Option<JsValue>>>,
    this: Option<JsValue>,
}

impl DeclarativeEnvironment {
    /// Get the binding value from the environment by it's index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range or not initialized.
    #[inline]
    pub(crate) fn get(&self, index: usize) -> JsValue {
        self.bindings
            .borrow()
            .get(index)
            .expect("binding index must be in range")
            .clone()
            .expect("binding must be initialized")
    }

    /// Set the binding value at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range or not initialized.
    #[inline]
    pub(crate) fn set(&self, index: usize, value: JsValue) {
        let mut bindings = self.bindings.borrow_mut();
        let binding = bindings
            .get_mut(index)
            .expect("binding index must be in range");
        assert!(!binding.is_none(), "binding must be initialized");
        *binding = Some(value);
    }
}

/// A declarative environment stack holds all declarative environments at runtime.
///
/// Environments themselves are garbage collected,
/// because they must be preserved for function calls.
#[derive(Clone, Debug, Trace, Finalize)]
pub struct DeclarativeEnvironmentStack {
    stack: Vec<Gc<DeclarativeEnvironment>>,
}

impl DeclarativeEnvironmentStack {
    /// Create a new environment stack with the most outer declarative environment.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            stack: vec![Gc::new(DeclarativeEnvironment {
                bindings: GcCell::new(Vec::new()),
                this: None,
            })],
        }
    }

    /// Set the number of bindings on the global environment.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[inline]
    pub(crate) fn set_global_binding_number(&mut self, binding_number: usize) {
        let environment = self
            .stack
            .get(0)
            .expect("global environment must always exist");
        let mut bindings = environment.bindings.borrow_mut();
        if bindings.len() < binding_number {
            bindings.resize(binding_number, None);
        }
    }

    /// Get the `this` value of the most outer function environment.
    #[inline]
    pub(crate) fn get_last_this(&self) -> Option<JsValue> {
        for env in self.stack.iter().rev() {
            if let Some(this) = &env.this {
                return Some(this.clone());
            }
        }
        None
    }

    /// Push a declarative environment on the environments stack.
    #[inline]
    pub(crate) fn push_declarative(&mut self, num_bindings: usize) {
        self.stack.push(Gc::new(DeclarativeEnvironment {
            bindings: GcCell::new(vec![None; num_bindings]),
            this: None,
        }));
    }

    /// Push a function environment on the environments stack.
    #[inline]
    pub(crate) fn push_function(&mut self, num_bindings: usize, this: JsValue) {
        self.stack.push(Gc::new(DeclarativeEnvironment {
            bindings: GcCell::new(vec![None; num_bindings]),
            this: Some(this),
        }));
    }

    /// Pop environment from the environments stack.
    #[inline]
    pub(crate) fn pop(&mut self) {
        debug_assert!(self.stack.len() > 1);
        self.stack.pop();
    }

    /// Get the most outer environment.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[inline]
    pub(crate) fn current(&mut self) -> Gc<DeclarativeEnvironment> {
        self.stack
            .last()
            .expect("global environment must always exist")
            .clone()
    }

    /// Get the value of a binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment of binding index are out of range.
    #[inline]
    pub(crate) fn get_value_optional(
        &self,
        environment_index: usize,
        binding_index: usize,
    ) -> Option<JsValue> {
        self.stack
            .get(environment_index)
            .expect("environment index must be in range")
            .bindings
            .borrow()
            .get(binding_index)
            .expect("binding index must be in range")
            .clone()
    }

    /// Set the value of a binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment of binding index are out of range.
    #[inline]
    pub(crate) fn put_value(
        &mut self,
        environment_index: usize,
        binding_index: usize,
        value: JsValue,
    ) {
        let mut bindings = self
            .stack
            .get(environment_index)
            .expect("environment index must be in range")
            .bindings
            .borrow_mut();
        let binding = bindings
            .get_mut(binding_index)
            .expect("binding index must be in range");
        *binding = Some(value);
    }

    /// Set the value of a binding if it is initialized.
    /// Return `true` if the value has been set.
    ///
    /// # Panics
    ///
    /// Panics if the environment of binding index are out of range.
    #[inline]
    pub(crate) fn put_value_if_initialized(
        &mut self,
        environment_index: usize,
        binding_index: usize,
        value: JsValue,
    ) -> bool {
        let mut bindings = self
            .stack
            .get(environment_index)
            .expect("environment index must be in range")
            .bindings
            .borrow_mut();
        let binding = bindings
            .get_mut(binding_index)
            .expect("binding index must be in range");
        if binding.is_none() {
            false
        } else {
            *binding = Some(value);
            true
        }
    }

    /// Set the value of a binding if it is uninitialized.
    ///
    /// # Panics
    ///
    /// Panics if the environment of binding index are out of range.
    #[inline]
    pub(crate) fn put_value_if_uninitialized(
        &mut self,
        environment_index: usize,
        binding_index: usize,
        value: JsValue,
    ) {
        let mut bindings = self
            .stack
            .get(environment_index)
            .expect("environment index must be in range")
            .bindings
            .borrow_mut();
        let binding = bindings
            .get_mut(binding_index)
            .expect("binding index must be in range");
        if binding.is_none() {
            *binding = Some(value);
        }
    }
}

/// A binding locator contains all information about a binding that is needed to resolve it at runtime.
///
/// Binding locators get created at compile time and are accessible at runtime via the [`crate::vm::CodeBlock`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BindingLocator {
    name: Sym,
    environment_index: usize,
    binding_index: usize,
    global: bool,
    mutate_immutable: bool,
}

impl BindingLocator {
    /// Creates a new declarative binding locator that has knows indices.
    #[inline]
    pub(in crate::environments) fn declarative(
        name: Sym,
        environment_index: usize,
        binding_index: usize,
    ) -> Self {
        Self {
            name,
            environment_index,
            binding_index,
            global: false,
            mutate_immutable: false,
        }
    }

    /// Creates a binding locator that indicates that the binding is on the global object.
    #[inline]
    pub(in crate::environments) fn global(name: Sym) -> Self {
        Self {
            name,
            environment_index: 0,
            binding_index: 0,
            global: true,
            mutate_immutable: false,
        }
    }

    /// Creates a binding locator that indicates that it was attempted to mutate an immutable binding.
    /// At runtime this should always produce a type error.
    #[inline]
    pub(in crate::environments) fn mutate_immutable(name: Sym) -> Self {
        Self {
            name,
            environment_index: 0,
            binding_index: 0,
            global: false,
            mutate_immutable: true,
        }
    }

    /// Returns the name of the binding.
    #[inline]
    pub(crate) fn name(&self) -> Sym {
        self.name
    }

    /// Returns if the binding is located on the global object.
    #[inline]
    pub(crate) fn is_global(&self) -> bool {
        self.global
    }

    /// Returns the environment index of the binding.
    #[inline]
    pub(crate) fn environment_index(&self) -> usize {
        self.environment_index
    }

    /// Returns the binding index of the binding.
    #[inline]
    pub(crate) fn binding_index(&self) -> usize {
        self.binding_index
    }

    /// Helper method to throws an error if the binding access is illegal.
    #[inline]
    pub(crate) fn throw_mutate_immutable(&self, context: &mut Context) -> JsResult<()> {
        if self.mutate_immutable {
            context.throw_type_error(format!(
                "cannot mutate an immutable binding '{}'",
                context.interner().resolve_expect(self.name)
            ))
        } else {
            Ok(())
        }
    }
}
