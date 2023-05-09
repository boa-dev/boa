mod function;
mod global;
mod lexical;

use std::cell::Cell;

use boa_gc::{Finalize, Gc, GcRefCell, Trace};
pub(crate) use function::{FunctionEnvironment, FunctionSlots, ThisBindingStatus};
pub(crate) use global::GlobalEnvironment;
pub(crate) use lexical::LexicalEnvironment;

use crate::{environments::CompileTimeEnvironment, JsObject, JsResult, JsValue};

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
    kind: DeclarativeEnvironmentKind,
    compile: Gc<GcRefCell<CompileTimeEnvironment>>,
}

impl DeclarativeEnvironment {
    /// Creates a new global `DeclarativeEnvironment`.
    pub(crate) fn global(global_this: JsObject) -> Self {
        DeclarativeEnvironment {
            kind: DeclarativeEnvironmentKind::Global(GlobalEnvironment::new(global_this)),
            compile: Gc::new(GcRefCell::new(CompileTimeEnvironment::new_global())),
        }
    }

    /// Creates a new `DeclarativeEnvironment` from its kind and compile environment.
    pub(crate) fn new(
        kind: DeclarativeEnvironmentKind,
        compile: Gc<GcRefCell<CompileTimeEnvironment>>,
    ) -> Self {
        Self { kind, compile }
    }

    /// Gets the compile time environment of this environment.
    pub(crate) fn compile_env(&self) -> Gc<GcRefCell<CompileTimeEnvironment>> {
        self.compile.clone()
    }

    /// Returns a reference to the the kind of the environment.
    pub(crate) const fn kind(&self) -> &DeclarativeEnvironmentKind {
        &self.kind
    }

    /// Gets the binding value from the environment by index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range or not initialized.
    #[track_caller]
    pub(crate) fn get(&self, index: usize) -> Option<JsValue> {
        self.kind.get(index)
    }

    /// Sets the binding value from the environment by index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set(&self, index: usize, value: JsValue) {
        self.kind.set(index, value);
    }

    /// `GetThisBinding`
    ///
    /// Returns the `this` binding of this environment.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-getthisbinding
    pub(crate) fn get_this_binding(&self) -> JsResult<Option<JsValue>> {
        self.kind.get_this_binding()
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
        self.kind.has_this_binding()
    }

    /// Returns `true` if this environment is poisoned.
    pub(crate) fn poisoned(&self) -> bool {
        self.kind.poisoned()
    }

    /// Returns `true` if this environment is inside a `with` environment.
    pub(crate) fn with(&self) -> bool {
        self.kind.with()
    }

    /// Poisons this environment for future binding searchs.
    pub(crate) fn poison(&self) {
        self.kind.poison();
    }
}

/// The kind of the declarative environment.
#[derive(Debug, Trace, Finalize)]
pub(crate) enum DeclarativeEnvironmentKind {
    /// Only stores lexical bindings.
    Lexical(LexicalEnvironment),
    /// Stores lexical bindings, global var bindings and the global this.
    Global(GlobalEnvironment),
    /// Stores lexical bindings, var bindings and the `FunctionSlots` of the function environment.
    Function(FunctionEnvironment),
}

impl DeclarativeEnvironmentKind {
    /// Unwraps the inner function environment if possible. Returns `None` otherwise.
    pub(crate) const fn as_function(&self) -> Option<&FunctionEnvironment> {
        if let Self::Function(fun) = &self {
            Some(fun)
        } else {
            None
        }
    }

    /// Unwraps the inner global environment if possible. Returns `None` otherwise.
    pub(crate) const fn as_global(&self) -> Option<&GlobalEnvironment> {
        if let Self::Global(fun) = &self {
            Some(fun)
        } else {
            None
        }
    }

    /// Get the binding value from the environment by it's index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range or not initialized.
    #[track_caller]
    pub(crate) fn get(&self, index: usize) -> Option<JsValue> {
        match self {
            DeclarativeEnvironmentKind::Lexical(inner) => inner.get(index),
            DeclarativeEnvironmentKind::Global(inner) => inner.get(index),
            DeclarativeEnvironmentKind::Function(inner) => inner.get(index),
        }
    }

    /// Sets the binding value from the environment by index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set(&self, index: usize, value: JsValue) {
        match self {
            DeclarativeEnvironmentKind::Lexical(inner) => inner.set(index, value),
            DeclarativeEnvironmentKind::Global(inner) => inner.set(index, value),
            DeclarativeEnvironmentKind::Function(inner) => inner.set(index, value),
        }
    }

    /// `GetThisBinding`
    ///
    /// Returns the `this` binding of this environment.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-getthisbinding
    pub(crate) fn get_this_binding(&self) -> JsResult<Option<JsValue>> {
        match self {
            DeclarativeEnvironmentKind::Lexical(_) => Ok(None),
            DeclarativeEnvironmentKind::Global(g) => Ok(Some(g.get_this_binding().into())),
            DeclarativeEnvironmentKind::Function(f) => f.get_this_binding(),
        }
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
        match self {
            DeclarativeEnvironmentKind::Lexical(_) => false,
            DeclarativeEnvironmentKind::Global(_) => true,
            DeclarativeEnvironmentKind::Function(f) => f.has_this_binding(),
        }
    }

    /// Returns `true` if this environment is poisoned.
    pub(crate) fn poisoned(&self) -> bool {
        match self {
            DeclarativeEnvironmentKind::Lexical(lex) => lex.poisonable_environment().poisoned(),
            DeclarativeEnvironmentKind::Global(g) => g.poisonable_environment().poisoned(),
            DeclarativeEnvironmentKind::Function(f) => f.poisonable_environment().poisoned(),
        }
    }

    /// Returns `true` if this environment is inside a `with` environment.
    pub(crate) fn with(&self) -> bool {
        match self {
            DeclarativeEnvironmentKind::Lexical(lex) => lex.poisonable_environment().with(),
            DeclarativeEnvironmentKind::Global(g) => g.poisonable_environment().with(),
            DeclarativeEnvironmentKind::Function(f) => f.poisonable_environment().with(),
        }
    }

    /// Poisons this environment for future binding searches.
    pub(crate) fn poison(&self) {
        match self {
            DeclarativeEnvironmentKind::Lexical(lex) => lex.poisonable_environment().poison(),
            DeclarativeEnvironmentKind::Global(g) => g.poisonable_environment().poison(),
            DeclarativeEnvironmentKind::Function(f) => f.poisonable_environment().poison(),
        }
    }
}

#[derive(Debug, Trace, Finalize)]
pub(crate) struct PoisonableEnvironment {
    bindings: GcRefCell<Vec<Option<JsValue>>>,
    #[unsafe_ignore_trace]
    poisoned: Cell<bool>,
    #[unsafe_ignore_trace]
    with: Cell<bool>,
}

impl PoisonableEnvironment {
    /// Creates a new `PoisonableEnvironment`.
    pub(crate) fn new(bindings_count: usize, poisoned: bool, with: bool) -> Self {
        PoisonableEnvironment {
            bindings: GcRefCell::new(vec![None; bindings_count]),
            poisoned: Cell::new(poisoned),
            with: Cell::new(with),
        }
    }

    /// Gets the bindings of this poisonable environment.
    pub(crate) const fn bindings(&self) -> &GcRefCell<Vec<Option<JsValue>>> {
        &self.bindings
    }

    /// Gets the binding value from the environment by it's index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    fn get(&self, index: usize) -> Option<JsValue> {
        self.bindings.borrow()[index].clone()
    }

    /// Sets the binding value from the environment by index.
    ///
    /// # Panics
    ///
    /// Panics if the binding value is out of range.
    #[track_caller]
    pub(crate) fn set(&self, index: usize, value: JsValue) {
        self.bindings.borrow_mut()[index] = Some(value);
    }

    /// Returns `true` if this environment is poisoned.
    fn poisoned(&self) -> bool {
        self.poisoned.get()
    }

    /// Returns `true` if this environment is inside a `with` environment.
    fn with(&self) -> bool {
        self.with.get()
    }

    /// Poisons this environment for future binding searches.
    fn poison(&self) {
        self.poisoned.set(true);
    }
}
