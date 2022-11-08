use std::cell::Cell;

use crate::{
    environments::CompileTimeEnvironment, error::JsNativeError, object::JsObject, Context, JsValue,
};
use boa_gc::{Cell as GcCell, Finalize, Gc, Trace};

use boa_ast::expression::Identifier;
use rustc_hash::FxHashSet;

/// A declarative environment holds binding values at runtime.
///
/// Bindings are stored in a fixed size list of optional values.
/// If a binding is not initialized, the value is `None`.
///
/// Optionally, an environment can hold a `this` value.
/// The `this` value is present only if the environment is a function environment.
///
/// Code evaluation at runtime (e.g. the `eval` built-in function) can add
/// bindings to existing, compiled function environments.
/// This makes it impossible to determine the location of all bindings at compile time.
/// To dynamically check for added bindings at runtime, a reference to the
/// corresponding compile time environment is needed.
///
/// Checking all environments for potential added bindings at runtime on every get/set
/// would offset the performance improvement of determining binding locations at compile time.
/// To minimize this, each environment holds a `poisoned` flag.
/// If bindings where added at runtime, the current environment and all inner environments
/// are marked as poisoned.
/// All poisoned environments have to be checked for added bindings.
#[derive(Debug, Trace, Finalize)]
pub(crate) struct DeclarativeEnvironment {
    bindings: GcCell<Vec<Option<JsValue>>>,
    compile: Gc<GcCell<CompileTimeEnvironment>>,
    #[unsafe_ignore_trace]
    poisoned: Cell<bool>,
    slots: Option<EnvironmentSlots>,
}

/// Describes the different types of internal slot data that an environment can hold.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) enum EnvironmentSlots {
    Function(GcCell<FunctionSlots>),
    Global,
}

impl EnvironmentSlots {
    /// Return the slots if they are part of a function environment.
    pub(crate) fn as_function_slots(&self) -> Option<&GcCell<FunctionSlots>> {
        if let Self::Function(env) = &self {
            Some(env)
        } else {
            None
        }
    }
}

/// Holds the internal slots of a function environment.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct FunctionSlots {
    /// The `[[ThisValue]]` internal slot.
    this: JsValue,

    /// The `[[ThisBindingStatus]]` internal slot.
    #[unsafe_ignore_trace]
    this_binding_status: ThisBindingStatus,

    /// The `[[FunctionObject]]` internal slot.
    function_object: JsObject,

    /// The `[[NewTarget]]` internal slot.
    new_target: Option<JsObject>,
}

impl FunctionSlots {
    /// Returns the value of the `[[FunctionObject]]` internal slot.
    pub(crate) fn function_object(&self) -> &JsObject {
        &self.function_object
    }

    /// Returns the value of the `[[NewTarget]]` internal slot.
    pub(crate) fn new_target(&self) -> Option<&JsObject> {
        self.new_target.as_ref()
    }

    /// `BindThisValue`
    ///
    /// Sets the given value as the `this` binding of the environment.
    /// Returns `false` if the `this` binding has already been initialized.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bindthisvalue
    pub(crate) fn bind_this_value(&mut self, this: &JsObject) -> bool {
        // 1. Assert: envRec.[[ThisBindingStatus]] is not lexical.
        debug_assert!(self.this_binding_status != ThisBindingStatus::Lexical);

        // 2. If envRec.[[ThisBindingStatus]] is initialized, throw a ReferenceError exception.
        if self.this_binding_status == ThisBindingStatus::Initialized {
            return false;
        }

        // 3. Set envRec.[[ThisValue]] to V.
        self.this = this.clone().into();

        // 4. Set envRec.[[ThisBindingStatus]] to initialized.
        self.this_binding_status = ThisBindingStatus::Initialized;

        // 5. Return V.
        true
    }

    /// `HasThisBinding`
    ///
    /// Returns if the environment has a `this` binding.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-hasthisbinding
    pub(crate) fn has_this_binding(&self) -> bool {
        // 1. If envRec.[[ThisBindingStatus]] is lexical, return false; otherwise, return true.
        self.this_binding_status != ThisBindingStatus::Lexical
    }

    /// `HasSuperBinding`
    ///
    /// Returns if the environment has a `super` binding.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-hassuperbinding
    ///
    /// # Panics
    ///
    /// Panics if the function object of the environment is not a function.
    pub(crate) fn has_super_binding(&self) -> bool {
        // 1.If envRec.[[ThisBindingStatus]] is lexical, return false.
        if self.this_binding_status == ThisBindingStatus::Lexical {
            return false;
        }

        // 2. If envRec.[[FunctionObject]].[[HomeObject]] is undefined, return false; otherwise, return true.
        self.function_object
            .borrow()
            .as_function()
            .expect("function object must be function")
            .get_home_object()
            .is_some()
    }

    /// `GetThisBinding`
    ///
    /// Returns the `this` binding on the function environment.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-getthisbinding
    pub(crate) fn get_this_binding(&self) -> Result<&JsValue, JsNativeError> {
        // 1. Assert: envRec.[[ThisBindingStatus]] is not lexical.
        debug_assert!(self.this_binding_status != ThisBindingStatus::Lexical);

        // 2. If envRec.[[ThisBindingStatus]] is uninitialized, throw a ReferenceError exception.
        if self.this_binding_status == ThisBindingStatus::Uninitialized {
            Err(JsNativeError::reference().with_message("Must call super constructor in derived class before accessing 'this' or returning from derived constructor"))
        } else {
            // 3. Return envRec.[[ThisValue]].
            Ok(&self.this)
        }
    }
}

/// Describes the status of a `this` binding in function environments.
#[derive(Clone, Copy, Debug, PartialEq)]
enum ThisBindingStatus {
    Lexical,
    Initialized,
    Uninitialized,
}

impl DeclarativeEnvironment {
    /// Returns the internal slot data of the current environment.
    pub(crate) fn slots(&self) -> Option<&EnvironmentSlots> {
        self.slots.as_ref()
    }

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
    pub(crate) fn new(global_compile_environment: Gc<GcCell<CompileTimeEnvironment>>) -> Self {
        Self {
            stack: vec![Gc::new(DeclarativeEnvironment {
                bindings: GcCell::new(Vec::new()),
                compile: global_compile_environment,
                poisoned: Cell::new(false),
                slots: Some(EnvironmentSlots::Global),
            })],
        }
    }

    /// Extends the length of the next outer function environment to the number of compiled bindings.
    ///
    /// This is only useful when compiled bindings are added after the initial compilation (eval).
    pub(crate) fn extend_outer_function_environment(&mut self) {
        for env in self.stack.iter().rev() {
            if let Some(EnvironmentSlots::Function(_)) = env.slots {
                let compile_bindings_number = env.compile.borrow().num_bindings();
                let mut bindings_mut = env.bindings.borrow_mut();

                if compile_bindings_number > bindings_mut.len() {
                    let diff = compile_bindings_number - bindings_mut.len();
                    bindings_mut.extend(vec![None; diff]);
                }
                break;
            }
        }
    }

    /// Check if any of the provided binding names are defined as lexical bindings.
    ///
    /// Start at the current environment.
    /// Stop at the next outer function environment.
    pub(crate) fn has_lex_binding_until_function_environment(
        &self,
        names: &FxHashSet<Identifier>,
    ) -> Option<Identifier> {
        for env in self.stack.iter().rev() {
            let compile = env.compile.borrow();
            for name in names {
                if compile.has_lex_binding(*name) {
                    return Some(*name);
                }
            }
            if compile.is_function() {
                break;
            }
        }
        None
    }

    /// Pop all current environments except the global environment.
    pub(crate) fn pop_to_global(&mut self) -> Vec<Gc<DeclarativeEnvironment>> {
        self.stack.split_off(1)
    }

    /// Get the number of current environments.
    pub(crate) fn len(&self) -> usize {
        self.stack.len()
    }

    /// Truncate current environments to the given number.
    pub(crate) fn truncate(&mut self, len: usize) {
        self.stack.truncate(len);
    }

    /// Extend the current environment stack with the given environments.
    pub(crate) fn extend(&mut self, other: Vec<Gc<DeclarativeEnvironment>>) {
        self.stack.extend(other);
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

    /// `GetThisEnvironment`
    ///
    /// Returns the environment that currently provides a `this` biding.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getthisenvironment
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[inline]
    pub(crate) fn get_this_environment(&self) -> &EnvironmentSlots {
        for env in self.stack.iter().rev() {
            if let Some(slots) = &env.slots {
                match slots {
                    EnvironmentSlots::Function(function_env) => {
                        if function_env.borrow().has_this_binding() {
                            return slots;
                        }
                    }
                    EnvironmentSlots::Global => return slots,
                }
            }
        }

        panic!("global environment must exist")
    }

    /// Push a declarative environment on the environments stack and return it's index.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[inline]
    pub(crate) fn push_declarative(
        &mut self,
        num_bindings: usize,
        compile_environment: Gc<GcCell<CompileTimeEnvironment>>,
    ) -> usize {
        let poisoned = self
            .stack
            .last()
            .expect("global environment must always exist")
            .poisoned
            .get();

        let index = self.stack.len();

        self.stack.push(Gc::new(DeclarativeEnvironment {
            bindings: GcCell::new(vec![None; num_bindings]),
            compile: compile_environment,
            poisoned: Cell::new(poisoned),
            slots: None,
        }));

        index
    }

    /// Push a function environment on the environments stack.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[inline]
    pub(crate) fn push_function(
        &mut self,
        num_bindings: usize,
        compile_environment: Gc<GcCell<CompileTimeEnvironment>>,
        this: Option<JsValue>,
        function_object: JsObject,
        new_target: Option<JsObject>,
        lexical: bool,
    ) {
        let outer = self
            .stack
            .last()
            .expect("global environment must always exist");

        let poisoned = outer.poisoned.get();

        let this_binding_status = if lexical {
            ThisBindingStatus::Lexical
        } else if this.is_some() {
            ThisBindingStatus::Initialized
        } else {
            ThisBindingStatus::Uninitialized
        };

        let this = if let Some(this) = this {
            this
        } else {
            JsValue::Null
        };

        self.stack.push(Gc::new(DeclarativeEnvironment {
            bindings: GcCell::new(vec![None; num_bindings]),
            compile: compile_environment,
            poisoned: Cell::new(poisoned),
            slots: Some(EnvironmentSlots::Function(GcCell::new(FunctionSlots {
                this,
                this_binding_status,
                function_object,
                new_target,
            }))),
        }));
    }

    /// Push a function environment that inherits it's internal slots from the outer environment.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    pub(crate) fn push_function_inherit(
        &mut self,
        num_bindings: usize,
        compile_environment: Gc<GcCell<CompileTimeEnvironment>>,
    ) {
        let outer = self
            .stack
            .last()
            .expect("global environment must always exist");

        let poisoned = outer.poisoned.get();
        let slots = outer.slots.clone();

        self.stack.push(Gc::new(DeclarativeEnvironment {
            bindings: GcCell::new(vec![None; num_bindings]),
            compile: compile_environment,
            poisoned: Cell::new(poisoned),
            slots,
        }));
    }

    /// Pop environment from the environments stack.
    #[inline]
    pub(crate) fn pop(&mut self) -> Gc<DeclarativeEnvironment> {
        debug_assert!(self.stack.len() > 1);
        self.stack
            .pop()
            .expect("environment stack is cannot be empty")
    }

    /// Get the most outer function environment slots.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[inline]
    pub(crate) fn current_function_slots(&self) -> &EnvironmentSlots {
        for env in self.stack.iter().rev() {
            if let Some(slots) = &env.slots {
                return slots;
            }
        }

        panic!("global environment must exist")
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

    /// Get the compile environment for the current runtime environment.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    pub(crate) fn current_compile_environment(&self) -> Gc<GcCell<CompileTimeEnvironment>> {
        self.stack
            .last()
            .expect("global environment must always exist")
            .compile
            .clone()
    }

    /// Mark that there may be added bindings in the current environment.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[inline]
    pub(crate) fn poison_current(&mut self) {
        self.stack
            .last()
            .expect("global environment must always exist")
            .poisoned
            .set(true);
    }

    /// Mark that there may be added binding in all environments.
    #[inline]
    pub(crate) fn poison_all(&mut self) {
        for env in &mut self.stack {
            if env.poisoned.get() {
                return;
            }
            env.poisoned.set(true);
        }
    }

    /// Get the value of a binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[inline]
    pub(crate) fn get_value_optional(
        &self,
        mut environment_index: usize,
        mut binding_index: usize,
        name: Identifier,
    ) -> Option<JsValue> {
        if environment_index != self.stack.len() - 1 {
            for env_index in (environment_index + 1..self.stack.len()).rev() {
                let env = self
                    .stack
                    .get(env_index)
                    .expect("environment index must be in range");
                if !env.poisoned.get() {
                    break;
                }
                let compile = env.compile.borrow();
                if compile.is_function() {
                    if let Some(b) = compile.get_binding(name) {
                        environment_index = b.environment_index;
                        binding_index = b.binding_index;
                        break;
                    }
                }
            }
        }

        self.stack
            .get(environment_index)
            .expect("environment index must be in range")
            .bindings
            .borrow()
            .get(binding_index)
            .expect("binding index must be in range")
            .clone()
    }

    /// Get the value of a binding by it's name.
    ///
    /// This only considers function environments that are poisoned.
    /// All other bindings are accessed via indices.
    #[inline]
    pub(crate) fn get_value_if_global_poisoned(&self, name: Identifier) -> Option<JsValue> {
        for env in self.stack.iter().rev() {
            if !env.poisoned.get() {
                return None;
            }
            let compile = env.compile.borrow();
            if compile.is_function() {
                if let Some(b) = compile.get_binding(name) {
                    return self
                        .stack
                        .get(b.environment_index)
                        .expect("environment index must be in range")
                        .bindings
                        .borrow()
                        .get(b.binding_index)
                        .expect("binding index must be in range")
                        .clone();
                }
            }
        }
        None
    }

    /// Set the value of a binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
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
    /// Panics if the environment or binding index are out of range.
    #[inline]
    pub(crate) fn put_value_if_initialized(
        &mut self,
        mut environment_index: usize,
        mut binding_index: usize,
        name: Identifier,
        value: JsValue,
    ) -> bool {
        if environment_index != self.stack.len() - 1 {
            for env_index in (environment_index + 1..self.stack.len()).rev() {
                let env = self
                    .stack
                    .get(env_index)
                    .expect("environment index must be in range");
                if !env.poisoned.get() {
                    break;
                }
                let compile = env.compile.borrow();
                if compile.is_function() {
                    if let Some(b) = compile.get_binding(name) {
                        environment_index = b.environment_index;
                        binding_index = b.binding_index;
                        break;
                    }
                }
            }
        }

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
    /// Panics if the environment or binding index are out of range.
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

    /// Set the value of a binding by it's name.
    ///
    /// This only considers function environments that are poisoned.
    /// All other bindings are set via indices.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[inline]
    pub(crate) fn put_value_if_global_poisoned(
        &mut self,
        name: Identifier,
        value: &JsValue,
    ) -> bool {
        for env in self.stack.iter().rev() {
            if !env.poisoned.get() {
                return false;
            }
            let compile = env.compile.borrow();
            if compile.is_function() {
                if let Some(b) = compile.get_binding(name) {
                    let mut bindings = self
                        .stack
                        .get(b.environment_index)
                        .expect("environment index must be in range")
                        .bindings
                        .borrow_mut();
                    let binding = bindings
                        .get_mut(b.binding_index)
                        .expect("binding index must be in range");
                    *binding = Some(value.clone());
                    return true;
                }
            }
        }
        false
    }

    /// Checks if the name only exists as a global property.
    ///
    /// A binding could be marked as `global`, and at the same time, exist in a deeper environment
    /// context; if the global context is poisoned, an `eval` call could have added a binding that is
    /// not global with the same name as the global binding. This double checks that the binding name
    /// is truly a global property.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[inline]
    pub(crate) fn is_only_global_property(&mut self, name: Identifier) -> bool {
        for env in self
            .stack
            .split_first()
            .expect("global environment must exist")
            .1
            .iter()
            .rev()
        {
            if !env.poisoned.get() {
                return true;
            }
            let compile = env.compile.borrow();
            if compile.is_function() && compile.get_binding(name).is_some() {
                return false;
            }
        }
        true
    }
}

/// A binding locator contains all information about a binding that is needed to resolve it at runtime.
///
/// Binding locators get created at bytecode compile time and are accessible at runtime via the [`crate::vm::CodeBlock`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BindingLocator {
    name: Identifier,
    environment_index: usize,
    binding_index: usize,
    global: bool,
    mutate_immutable: bool,
    silent: bool,
}

impl BindingLocator {
    /// Creates a new declarative binding locator that has knows indices.
    #[inline]
    pub(in crate::environments) fn declarative(
        name: Identifier,
        environment_index: usize,
        binding_index: usize,
    ) -> Self {
        Self {
            name,
            environment_index,
            binding_index,
            global: false,
            mutate_immutable: false,
            silent: false,
        }
    }

    /// Creates a binding locator that indicates that the binding is on the global object.
    #[inline]
    pub(in crate::environments) fn global(name: Identifier) -> Self {
        Self {
            name,
            environment_index: 0,
            binding_index: 0,
            global: true,
            mutate_immutable: false,
            silent: false,
        }
    }

    /// Creates a binding locator that indicates that it was attempted to mutate an immutable binding.
    /// At runtime this should always produce a type error.
    #[inline]
    pub(in crate::environments) fn mutate_immutable(name: Identifier) -> Self {
        Self {
            name,
            environment_index: 0,
            binding_index: 0,
            global: false,
            mutate_immutable: true,
            silent: false,
        }
    }

    /// Creates a binding locator that indicates that any action is silently ignored.
    #[inline]
    pub(in crate::environments) fn silent(name: Identifier) -> Self {
        Self {
            name,
            environment_index: 0,
            binding_index: 0,
            global: false,
            mutate_immutable: false,
            silent: true,
        }
    }

    /// Returns the name of the binding.
    #[inline]
    pub(crate) fn name(&self) -> Identifier {
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

    /// Returns if the binding is a silent operation.
    #[inline]
    pub(crate) fn is_silent(&self) -> bool {
        self.silent
    }

    /// Helper method to throws an error if the binding access is illegal.
    #[inline]
    pub(crate) fn throw_mutate_immutable(
        &self,
        context: &mut Context,
    ) -> Result<(), JsNativeError> {
        if self.mutate_immutable {
            Err(JsNativeError::typ().with_message(format!(
                "cannot mutate an immutable binding '{}'",
                context.interner().resolve_expect(self.name.sym())
            )))
        } else {
            Ok(())
        }
    }
}
