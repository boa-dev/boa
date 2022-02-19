use crate::{
    environments::runtime::BindingLocator, property::PropertyDescriptor, Context, JsResult,
    JsString, JsValue,
};
use boa_interner::Sym;
use rustc_hash::FxHashMap;

/// A compile time binding represents a binding at bytecode compile time in a [`CompileTimeEnvironment`].
///
/// It contains the binding index and a flag to indicate if this is a mutable binding or not.
#[derive(Debug)]
struct CompileTimeBinding {
    index: usize,
    mutable: bool,
}

/// A compile time environment maps bound identifiers to their binding positions.
///
/// A compile time environment also indicates, if it is a function environment.
#[derive(Debug)]
pub(crate) struct CompileTimeEnvironment {
    bindings: FxHashMap<Sym, CompileTimeBinding>,
    function_scope: bool,
}

impl CompileTimeEnvironment {
    /// Returns the number of bindings in this environment.
    #[inline]
    pub(crate) fn num_bindings(&self) -> usize {
        self.bindings.len()
    }
}

/// The compile time environment stack contains a stack of all environments at bytecode compile time.
///
/// The first environment on the stack represents the global environment.
/// This is never being deleted and is tied to the existence of the realm.
/// All other environments are being dropped once they are not needed anymore.
#[derive(Debug)]
pub(crate) struct CompileTimeEnvironmentStack {
    stack: Vec<CompileTimeEnvironment>,
}

impl CompileTimeEnvironmentStack {
    /// Creates a new compile time environment stack.
    ///
    /// This function should only be used once, on realm creation.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            stack: vec![CompileTimeEnvironment {
                bindings: FxHashMap::default(),
                function_scope: true,
            }],
        }
    }

    /// Get the number of bindings for the current last environment.
    ///
    /// # Panics
    ///
    /// Panics if there are no environments on the stack.
    #[inline]
    pub(crate) fn get_binding_number(&self) -> usize {
        self.stack
            .last()
            .expect("global environment must always exist")
            .num_bindings()
    }
}

impl Context {
    /// Push either a new declarative or function environment on the compile time environment stack.
    ///
    /// Note: This function only works at bytecode compile time!
    #[inline]
    pub(crate) fn push_compile_time_environment(&mut self, function_scope: bool) {
        self.realm.compile_env.stack.push(CompileTimeEnvironment {
            bindings: FxHashMap::default(),
            function_scope,
        });
    }

    /// Pop the last compile time environment from the stack.
    ///
    /// Note: This function only works at bytecode compile time!
    ///
    /// # Panics
    ///
    /// Panics if there are no more environments that can be pop'ed.
    #[inline]
    pub(crate) fn pop_compile_time_environment(&mut self) -> CompileTimeEnvironment {
        assert!(
            self.realm.compile_env.stack.len() > 1,
            "cannot pop global environment"
        );
        self.realm
            .compile_env
            .stack
            .pop()
            .expect("len > 1 already checked")
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
        self.realm
            .compile_env
            .stack
            .last()
            .expect("global environment must always exist")
            .num_bindings()
    }

    /// Get the binding locator of the binding at bytecode compile time.
    ///
    /// Note: This function only works at bytecode compile time!
    #[inline]
    pub(crate) fn get_binding_value(&self, name: Sym) -> BindingLocator {
        for (i, env) in self.realm.compile_env.stack.iter().enumerate().rev() {
            if let Some(binding) = env.bindings.get(&name) {
                return BindingLocator::declarative(name, i, binding.index);
            }
        }
        BindingLocator::global(name)
    }

    /// Return if a declarative binding exists at bytecode compile time.
    /// This does not include bindings on the global object.
    ///
    /// Note: This function only works at bytecode compile time!
    #[inline]
    pub(crate) fn has_binding(&self, name: Sym) -> bool {
        for env in self.realm.compile_env.stack.iter().rev() {
            if env.bindings.contains_key(&name) {
                return true;
            }
        }
        false
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
        name: Sym,
        function_scope: bool,
        allow_name_reuse: bool,
    ) -> JsResult<()> {
        let name_str = JsString::from(self.interner().resolve_expect(name));

        for (i, env) in self.realm.compile_env.stack.iter_mut().enumerate().rev() {
            if !function_scope || env.function_scope {
                if env.bindings.contains_key(&name) {
                    if allow_name_reuse {
                        return Ok(());
                    }
                    return self
                        .throw_syntax_error(format!("Redeclaration of variable {}", name_str));
                }

                if i == 0 {
                    let desc = self
                        .realm
                        .global_property_map
                        .string_property_map()
                        .get(&name_str);
                    let non_configurable_binding_exists = match desc {
                        Some(desc) => !matches!(desc.configurable(), Some(true)),
                        None => false,
                    };
                    if function_scope && desc.is_none() {
                        self.global_bindings_mut().insert(
                            name_str,
                            PropertyDescriptor::builder()
                                .value(JsValue::Undefined)
                                .writable(true)
                                .enumerable(true)
                                .configurable(true)
                                .build(),
                        );
                        return Ok(());
                    } else if function_scope {
                        return Ok(());
                    } else if !function_scope
                        && !allow_name_reuse
                        && non_configurable_binding_exists
                    {
                        return self
                            .throw_syntax_error(format!("Redeclaration of variable {}", name_str));
                    }
                }

                let binding_index = env.bindings.len();
                env.bindings.insert(
                    name,
                    CompileTimeBinding {
                        index: binding_index,
                        mutable: true,
                    },
                );
                return Ok(());
            }
            continue;
        }
        panic!("global environment must be function scoped")
    }

    /// Initialize a mutable binding at bytecode compile time and return it's binding locator.
    ///
    /// Note: This function only works at bytecode compile time!
    #[inline]
    pub(crate) fn initialize_mutable_binding(
        &self,
        name: Sym,
        function_scope: bool,
    ) -> BindingLocator {
        for (i, env) in self.realm.compile_env.stack.iter().enumerate().rev() {
            if function_scope && !env.function_scope {
                continue;
            }
            if let Some(binding) = env.bindings.get(&name) {
                return BindingLocator::declarative(name, i, binding.index);
            }
            return BindingLocator::global(name);
        }
        BindingLocator::global(name)
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
    pub(crate) fn create_immutable_binding(&mut self, name: Sym) -> JsResult<()> {
        let name_str = JsString::from(self.interner().resolve_expect(name));
        let exists_global = self.realm.compile_env.stack.len() == 1
            && self.global_bindings().contains_key(&name_str);

        let env = self
            .realm
            .compile_env
            .stack
            .last_mut()
            .expect("global environment must always exist");

        if env.bindings.contains_key(&name) || exists_global {
            self.throw_syntax_error(format!("Redeclaration of variable {}", name_str))
        } else {
            let binding_index = env.bindings.len();
            env.bindings.insert(
                name,
                CompileTimeBinding {
                    index: binding_index,
                    mutable: false,
                },
            );
            Ok(())
        }
    }

    /// Initialize an immutable binding at bytecode compile time and return it's binding locator.
    ///
    /// Note: This function only works at bytecode compile time!
    ///
    /// # Panics
    ///
    /// Panics if the global environment does not exist or a the binding was not created on the current environment.
    #[inline]
    pub(crate) fn initialize_immutable_binding(&self, name: Sym) -> BindingLocator {
        let environment_index = self.realm.compile_env.stack.len() - 1;
        let env = self
            .realm
            .compile_env
            .stack
            .last()
            .expect("global environment must always exist");

        let binding = env.bindings.get(&name).expect("binding must exist");
        BindingLocator::declarative(name, environment_index, binding.index)
    }

    /// Return the binding locator for a set operation on an existing binding.
    ///
    /// Note: This function only works at bytecode compile time!
    #[inline]
    pub(crate) fn set_mutable_binding(&self, name: Sym) -> BindingLocator {
        for (i, env) in self.realm.compile_env.stack.iter().enumerate().rev() {
            if let Some(binding) = env.bindings.get(&name) {
                if binding.mutable {
                    return BindingLocator::declarative(name, i, binding.index);
                }
                return BindingLocator::mutate_immutable(name);
            }
        }
        BindingLocator::global(name)
    }
}
