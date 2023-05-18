use crate::{
    environments::CompileTimeEnvironment,
    error::JsNativeError,
    object::{JsObject, PrivateName},
    Context, JsResult, JsString, JsSymbol, JsValue,
};
use boa_ast::expression::Identifier;
use boa_gc::{empty_trace, Finalize, Gc, GcRefCell, Trace};
use boa_interner::Sym;
use rustc_hash::FxHashSet;

mod declarative;
mod private;

use self::declarative::ModuleEnvironment;
pub(crate) use self::{
    declarative::{
        DeclarativeEnvironment, DeclarativeEnvironmentKind, FunctionEnvironment, FunctionSlots,
        LexicalEnvironment, ThisBindingStatus,
    },
    private::PrivateEnvironment,
};

/// The environment stack holds all environments at runtime.
///
/// Environments themselves are garbage collected,
/// because they must be preserved for function calls.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct EnvironmentStack {
    stack: Vec<Environment>,

    private_stack: Vec<Gc<PrivateEnvironment>>,
}

/// A runtime environment.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) enum Environment {
    Declarative(Gc<DeclarativeEnvironment>),
    Object(JsObject),
}

impl Environment {
    /// Returns the declarative environment if it is one.
    pub(crate) const fn as_declarative(&self) -> Option<&Gc<DeclarativeEnvironment>> {
        match self {
            Self::Declarative(env) => Some(env),
            Self::Object(_) => None,
        }
    }

    /// Returns the declarative environment and panic if it is not one.
    #[track_caller]
    pub(crate) fn declarative_expect(&self) -> &Gc<DeclarativeEnvironment> {
        self.as_declarative()
            .expect("environment must be declarative")
    }
}

impl EnvironmentStack {
    /// Create a new environment stack.
    pub(crate) fn new(global: Gc<DeclarativeEnvironment>) -> Self {
        assert!(matches!(
            global.kind(),
            DeclarativeEnvironmentKind::Global(_)
        ));
        Self {
            stack: vec![Environment::Declarative(global)],
            private_stack: Vec::new(),
        }
    }

    /// Replaces the current global with a new global environment.
    pub(crate) fn replace_global(&mut self, global: Gc<DeclarativeEnvironment>) {
        assert!(matches!(
            global.kind(),
            DeclarativeEnvironmentKind::Global(_)
        ));
        self.stack[0] = Environment::Declarative(global);
    }

    /// Extends the length of the next outer function environment to the number of compiled bindings.
    ///
    /// This is only useful when compiled bindings are added after the initial compilation (eval).
    pub(crate) fn extend_outer_function_environment(&mut self) {
        for env in self
            .stack
            .iter()
            .filter_map(Environment::as_declarative)
            .rev()
        {
            if let DeclarativeEnvironmentKind::Function(fun) = &env.kind() {
                let compile_bindings_number = env.compile_env().borrow().num_bindings();
                let mut bindings_mut = fun.poisonable_environment().bindings().borrow_mut();

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
        for env in self
            .stack
            .iter()
            .filter_map(Environment::as_declarative)
            .rev()
        {
            let compile = env.compile_env();
            let compile = compile.borrow();
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

    /// Check if the next outer function environment is the global environment.
    pub(crate) fn is_next_outer_function_environment_global(&self) -> bool {
        for env in self
            .stack
            .iter()
            .rev()
            .filter_map(Environment::as_declarative)
        {
            let compile = env.compile_env();
            let compile = compile.borrow();
            if compile.is_function() {
                return compile.outer().is_none();
            }
        }
        true
    }

    /// Pop all current environments except the global environment.
    pub(crate) fn pop_to_global(&mut self) -> Vec<Environment> {
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
    pub(crate) fn extend(&mut self, other: Vec<Environment>) {
        self.stack.extend(other);
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
    pub(crate) fn get_this_environment(&self) -> &DeclarativeEnvironmentKind {
        for env in self.stack.iter().rev() {
            if let Some(decl) = env.as_declarative().filter(|decl| decl.has_this_binding()) {
                return decl.kind();
            }
        }

        panic!("global environment must exist");
    }

    /// `GetThisBinding`
    ///
    /// Returns the current `this` binding of the environment.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-environment-records-getthisbinding
    pub(crate) fn get_this_binding(&self) -> JsResult<JsValue> {
        for env in self.stack.iter().rev() {
            if let Environment::Declarative(decl) = env {
                if let Some(this) = decl.get_this_binding()? {
                    return Ok(this);
                }
            }
        }

        panic!("global environment must exist");
    }

    /// Push a new object environment on the environments stack and return it's index.
    pub(crate) fn push_object(&mut self, object: JsObject) -> usize {
        let index = self.stack.len();
        self.stack.push(Environment::Object(object));
        index
    }

    /// Push a lexical environment on the environments stack and return it's index.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[track_caller]
    pub(crate) fn push_lexical(
        &mut self,
        num_bindings: usize,
        compile_environment: Gc<GcRefCell<CompileTimeEnvironment>>,
    ) -> usize {
        let (poisoned, with) = {
            let with = self
                .stack
                .last()
                .expect("global environment must always exist")
                .as_declarative()
                .is_none();

            let environment = self
                .stack
                .iter()
                .rev()
                .find_map(Environment::as_declarative)
                .expect("global environment must always exist");
            (environment.poisoned(), with || environment.with())
        };

        let index = self.stack.len();

        self.stack.push(Environment::Declarative(Gc::new(
            DeclarativeEnvironment::new(
                DeclarativeEnvironmentKind::Lexical(LexicalEnvironment::new(
                    num_bindings,
                    poisoned,
                    with,
                )),
                compile_environment,
            ),
        )));

        index
    }

    /// Push a function environment on the environments stack.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[track_caller]
    pub(crate) fn push_function(
        &mut self,
        num_bindings: usize,
        compile_environment: Gc<GcRefCell<CompileTimeEnvironment>>,
        function_slots: FunctionSlots,
    ) {
        let (poisoned, with) = {
            let with = self
                .stack
                .last()
                .expect("global environment must always exist")
                .as_declarative()
                .is_none();

            let environment = self
                .stack
                .iter()
                .rev()
                .find_map(Environment::as_declarative)
                .expect("global environment must always exist");
            (environment.poisoned(), with || environment.with())
        };

        self.stack.push(Environment::Declarative(Gc::new(
            DeclarativeEnvironment::new(
                DeclarativeEnvironmentKind::Function(FunctionEnvironment::new(
                    num_bindings,
                    poisoned,
                    with,
                    function_slots,
                )),
                compile_environment,
            ),
        )));
    }

    /// Push a function environment that inherits it's internal slots from the outer function
    /// environment.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[track_caller]
    pub(crate) fn push_function_inherit(
        &mut self,
        num_bindings: usize,
        compile_environment: Gc<GcRefCell<CompileTimeEnvironment>>,
    ) {
        debug_assert!(
            self.stack.len() == compile_environment.borrow().environment_index(),
            "tried to push an invalid compile environment"
        );

        let (poisoned, with, slots) = {
            let with = self
                .stack
                .last()
                .expect("can only be called inside a function")
                .as_declarative()
                .is_none();

            let (environment, slots) = self
                .stack
                .iter()
                .rev()
                .find_map(|env| {
                    if let Some(env) = env.as_declarative() {
                        if let DeclarativeEnvironmentKind::Function(fun) = env.kind() {
                            return Some((env, fun.slots().clone()));
                        }
                    }
                    None
                })
                .expect("can only be called inside a function");
            (environment.poisoned(), with || environment.with(), slots)
        };

        self.stack.push(Environment::Declarative(Gc::new(
            DeclarativeEnvironment::new(
                DeclarativeEnvironmentKind::Function(FunctionEnvironment::new(
                    num_bindings,
                    poisoned,
                    with,
                    slots,
                )),
                compile_environment,
            ),
        )));
    }

    /// Push a module environment on the environments stack.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[track_caller]
    pub(crate) fn push_module(
        &mut self,
        compile_environment: Gc<GcRefCell<CompileTimeEnvironment>>,
    ) {
        let num_bindings = compile_environment.borrow().num_bindings();
        self.stack.push(Environment::Declarative(Gc::new(
            DeclarativeEnvironment::new(
                DeclarativeEnvironmentKind::Module(ModuleEnvironment::new(num_bindings)),
                compile_environment,
            ),
        )));
    }

    /// Pop environment from the environments stack.
    #[track_caller]
    pub(crate) fn pop(&mut self) -> Environment {
        debug_assert!(self.stack.len() > 1);
        self.stack
            .pop()
            .expect("environment stack is cannot be empty")
    }

    /// Get the most outer environment.
    ///
    /// # Panics
    ///
    /// Panics if no environment exists on the stack.
    #[track_caller]
    pub(crate) fn current(&self) -> Environment {
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
    pub(crate) fn current_compile_environment(&self) -> Gc<GcRefCell<CompileTimeEnvironment>> {
        self.stack
            .iter()
            .filter_map(Environment::as_declarative)
            .last()
            .expect("global environment must always exist")
            .compile_env()
    }

    /// Mark that there may be added bindings from the current environment to the next function
    /// environment.
    pub(crate) fn poison_until_last_function(&mut self) {
        for env in self
            .stack
            .iter()
            .rev()
            .filter_map(Environment::as_declarative)
        {
            env.poison();
            if env.compile_env().borrow().is_function() {
                return;
            }
        }
    }

    /// Set the value of a lexical binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[track_caller]
    pub(crate) fn put_lexical_value(
        &mut self,
        environment_index: usize,
        binding_index: usize,
        value: JsValue,
    ) {
        let env = self
            .stack
            .get(environment_index)
            .expect("environment index must be in range")
            .declarative_expect();
        env.set(binding_index, value);
    }

    /// Set the value of a binding if it is uninitialized.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[track_caller]
    pub(crate) fn put_value_if_uninitialized(
        &mut self,
        environment_index: usize,
        binding_index: usize,
        value: JsValue,
    ) {
        let env = self
            .stack
            .get(environment_index)
            .expect("environment index must be in range")
            .declarative_expect();
        if env.get(binding_index).is_none() {
            env.set(binding_index, value);
        }
    }

    /// Push a private environment to the private environment stack.
    pub(crate) fn push_private(&mut self, environment: Gc<PrivateEnvironment>) {
        self.private_stack.push(environment);
    }

    /// Pop a private environment from the private environment stack.
    pub(crate) fn pop_private(&mut self) {
        self.private_stack.pop();
    }

    /// `ResolvePrivateIdentifier ( privEnv, identifier )`
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-resolve-private-identifier
    pub(crate) fn resolve_private_identifier(&self, identifier: Sym) -> Option<PrivateName> {
        // 1. Let names be privEnv.[[Names]].
        // 2. For each Private Name pn of names, do
        //     a. If pn.[[Description]] is identifier, then
        //         i. Return pn.
        // 3. Let outerPrivEnv be privEnv.[[OuterPrivateEnvironment]].
        // 4. Assert: outerPrivEnv is not null.
        // 5. Return ResolvePrivateIdentifier(outerPrivEnv, identifier).
        for environment in self.private_stack.iter().rev() {
            if environment.descriptions().contains(&identifier) {
                return Some(PrivateName::new(identifier, environment.id()));
            }
        }
        None
    }

    /// Return all private name descriptions in all private environments.
    pub(crate) fn private_name_descriptions(&self) -> Vec<Sym> {
        let mut names = Vec::new();
        for environment in self.private_stack.iter().rev() {
            for name in environment.descriptions() {
                if !names.contains(name) {
                    names.push(*name);
                }
            }
        }
        names
    }
}

/// A binding locator contains all information about a binding that is needed to resolve it at runtime.
///
/// Binding locators get created at bytecode compile time and are accessible at runtime via the [`crate::vm::CodeBlock`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Finalize)]
pub(crate) struct BindingLocator {
    name: Identifier,
    environment_index: usize,
    binding_index: usize,
    global: bool,
    mutate_immutable: bool,
    silent: bool,
}

unsafe impl Trace for BindingLocator {
    empty_trace!();
}

impl BindingLocator {
    /// Creates a new declarative binding locator that has knows indices.
    pub(crate) const fn declarative(
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
    pub(super) const fn global(name: Identifier) -> Self {
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
    pub(in crate::environments) const fn mutate_immutable(name: Identifier) -> Self {
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
    pub(in crate::environments) const fn silent(name: Identifier) -> Self {
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
    pub(crate) const fn name(&self) -> Identifier {
        self.name
    }

    /// Returns if the binding is located on the global object.
    pub(crate) const fn is_global(&self) -> bool {
        self.global
    }

    /// Returns the environment index of the binding.
    pub(crate) const fn environment_index(&self) -> usize {
        self.environment_index
    }

    /// Returns the binding index of the binding.
    pub(crate) const fn binding_index(&self) -> usize {
        self.binding_index
    }

    /// Returns if the binding is a silent operation.
    pub(crate) const fn is_silent(&self) -> bool {
        self.silent
    }

    /// Helper method to throws an error if the binding access is illegal.
    pub(crate) fn throw_mutate_immutable(
        &self,
        context: &mut Context<'_>,
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

impl Context<'_> {
    /// Gets the corresponding runtime binding of the provided `BindingLocator`, modifying
    /// its indexes in place.
    ///
    /// This readjusts a `BindingLocator` to the correct binding if a `with` environment or
    /// `eval` call modified the compile-time bindings.
    ///
    /// Only use if the binding origin is unknown or comes from a `var` declaration. Lexical bindings
    /// are completely removed of runtime checks because the specification guarantees that runtime
    /// semantics cannot add or remove lexical bindings.
    pub(crate) fn find_runtime_binding(&mut self, locator: &mut BindingLocator) -> JsResult<()> {
        let current = self.vm.environments.current();
        if let Some(env) = current.as_declarative() {
            if !env.with() && !env.poisoned() {
                return Ok(());
            }
        }

        for env_index in (locator.environment_index..self.vm.environments.stack.len()).rev() {
            match self.environment_expect(env_index) {
                Environment::Declarative(env) => {
                    if env.poisoned() {
                        let compile = env.compile_env();
                        let compile = compile.borrow();
                        if compile.is_function() {
                            if let Some(b) = compile.get_binding(locator.name) {
                                locator.environment_index = b.environment_index;
                                locator.binding_index = b.binding_index;
                                locator.global = false;
                                break;
                            }
                        }
                    } else if !env.with() {
                        break;
                    }
                }
                Environment::Object(o) => {
                    let o = o.clone();
                    let key: JsString = self
                        .interner()
                        .resolve_expect(locator.name.sym())
                        .into_common(false);
                    if o.has_property(key.clone(), self)? {
                        if let Some(unscopables) = o.get(JsSymbol::unscopables(), self)?.as_object()
                        {
                            if unscopables.get(key.clone(), self)?.to_boolean() {
                                continue;
                            }
                        }
                        locator.environment_index = env_index;
                        locator.global = false;
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Checks if the binding pointed by `locator` is initialized.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    pub(crate) fn is_initialized_binding(&mut self, locator: &BindingLocator) -> JsResult<bool> {
        if locator.global {
            let key: JsString = self
                .interner()
                .resolve_expect(locator.name.sym())
                .into_common(false);
            self.global_object().has_property(key, self)
        } else {
            match self.environment_expect(locator.environment_index) {
                Environment::Declarative(env) => Ok(env.get(locator.binding_index).is_some()),
                Environment::Object(obj) => {
                    let key: JsString = self
                        .interner()
                        .resolve_expect(locator.name.sym())
                        .into_common(false);
                    obj.clone().has_property(key, self)
                }
            }
        }
    }

    /// Get the value of a binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    pub(crate) fn get_binding(&mut self, locator: BindingLocator) -> JsResult<Option<JsValue>> {
        if locator.global {
            let global = self.global_object();
            let key: JsString = self
                .interner()
                .resolve_expect(locator.name.sym())
                .into_common(false);
            if global.has_property(key.clone(), self)? {
                global.get(key, self).map(Some)
            } else {
                Ok(None)
            }
        } else {
            match self.environment_expect(locator.environment_index) {
                Environment::Declarative(env) => Ok(env.get(locator.binding_index)),
                Environment::Object(obj) => {
                    let obj = obj.clone();
                    let key: JsString = self
                        .interner()
                        .resolve_expect(locator.name.sym())
                        .into_common(false);
                    obj.get(key, self).map(Some)
                }
            }
        }
    }

    /// Sets the value of a binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[track_caller]
    pub(crate) fn set_binding(
        &mut self,
        locator: BindingLocator,
        value: JsValue,
        strict: bool,
    ) -> JsResult<()> {
        if locator.global {
            let key = self
                .interner()
                .resolve_expect(locator.name().sym())
                .into_common::<JsString>(false);

            self.global_object().set(key, value, strict, self)?;
        } else {
            match self.environment_expect(locator.environment_index) {
                Environment::Declarative(decl) => {
                    decl.set(locator.binding_index, value);
                }
                Environment::Object(obj) => {
                    let obj = obj.clone();
                    let key: JsString = self
                        .interner()
                        .resolve_expect(locator.name.sym())
                        .into_common(false);

                    obj.set(key, value, strict, self)?;
                }
            }
        }

        Ok(())
    }

    /// Deletes a binding if it exists.
    ///
    /// Returns `true` if the binding was deleted.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    pub(crate) fn delete_binding(&mut self, locator: BindingLocator) -> JsResult<bool> {
        if locator.is_global() {
            let key: JsString = self
                .interner()
                .resolve_expect(locator.name().sym())
                .into_common::<JsString>(false);
            self.global_object().__delete__(&key.into(), self)
        } else {
            match self.environment_expect(locator.environment_index) {
                Environment::Declarative(_) => Ok(false),
                Environment::Object(obj) => {
                    let obj = obj.clone();
                    let key: JsString = self
                        .interner()
                        .resolve_expect(locator.name.sym())
                        .into_common(false);

                    obj.__delete__(&key.into(), self)
                }
            }
        }
    }

    /// Return the environment at the given index. Panics if the index is out of range.
    pub(crate) fn environment_expect(&self, index: usize) -> &Environment {
        self.vm
            .environments
            .stack
            .get(index)
            .expect("environment index must be in range")
    }
}
