//! This module implements ECMAScript `Environment Records`.
//!
//! Environments contain the bindings of identifiers to their values.
//! The implementation differs from the methods defined by the specification,
//! but the resulting behavior should be the same.
//!
//! To make the runtime more performant, environment specific behavior is split
//! between bytecode compilation and the runtime.
//! While the association of identifiers to values seems like a natural fit for a hashmap,
//! lookups of the values at runtime are very expensive.
//! Environments can also have outer environments.
//! In the worst case, there are as many hashmap lookups, as there are environments.
//!
//! To avoid these costs, hashmaps are not used at runtime.
//! At runtime, environments are represented as fixed size lists of binding values.
//! The positions of the bindings in these lists is determined at compile time.
//!
//! A binding is uniquely identified by two indices:
//!  - An environment index, that identifies the environment in which the binding exists
//!  - A binding index, that identifies the binding in the environment
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-environment-records

use crate::{
    gc::{Finalize, Gc, Trace},
    property::PropertyDescriptor,
    Context, JsResult, JsString, JsValue,
};
use boa_interner::Sym;
use gc::GcCell;
use rustc_hash::FxHashMap;

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
/// Binding locators get created at compile time and are accessible at runtime via the [`CodeBlock`].
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
    fn declarative(name: Sym, environment_index: usize, binding_index: usize) -> Self {
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
    fn global(name: Sym) -> Self {
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
    fn mutate_immutable(name: Sym) -> Self {
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

/// A compile time binding represents a binding at compile time in a [`CompileTimeEnvironment`].
///
/// It contains the binding index and a flag to indicate if this is an mutable binding or not.
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

/// The compile time environment stack contains a stack of all environments at compile time.
///
/// The first environment on the stack represents the global environment.
/// This is never being deleted and is tied to the existence of the realm.
/// All other environments are being dropped once they are not needed anymore a compile time.
#[derive(Debug)]
pub(crate) struct CompileTimeEnvironmentStack {
    stack: Vec<CompileTimeEnvironment>,
}

impl CompileTimeEnvironmentStack {
    /// Creates a new compile time environment stack.
    ///
    /// This function should one be used once on realm creation.
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
    /// Note: This function only works at compile time!
    #[inline]
    pub(crate) fn push_compile_time_environment(&mut self, function_scope: bool) {
        self.realm.compile_env.stack.push(CompileTimeEnvironment {
            bindings: FxHashMap::default(),
            function_scope,
        });
    }

    /// Pop the last compile time environment from the stack.
    ///
    /// Note: This function only works at compile time!
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
        self.realm.compile_env.stack.pop().unwrap()
    }

    /// Get the number of bindings for the current compile time environment.
    ///
    /// Note: This function only works at compile time!
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

    /// Get the binding locator of the binding at compile time.
    ///
    /// Note: This function only works at compile time!
    #[inline]
    pub(crate) fn get_binding_value(&self, name: Sym) -> BindingLocator {
        for (i, env) in self.realm.compile_env.stack.iter().enumerate().rev() {
            if let Some(binding) = env.bindings.get(&name) {
                return BindingLocator::declarative(name, i, binding.index);
            }
        }
        BindingLocator::global(name)
    }

    /// Return if a declarative binding exists at compile time.
    /// This does not include bindings on the global object.
    ///
    /// Note: This function only works at compile time!
    #[inline]
    pub(crate) fn has_binding(&self, name: Sym) -> bool {
        for env in self.realm.compile_env.stack.iter().rev() {
            if env.bindings.contains_key(&name) {
                return true;
            }
        }
        false
    }

    /// Create a mutable binding at compile time.
    /// This function returns a syntax error, if the binding is a redeclaration.
    ///
    /// Note: This function only works at compile time!
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

    /// Initialize a mutable binding at compile time and return it's binding locator.
    ///
    /// Note: This function only works at compile time!
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

    /// Create an immutable binding at compile time.
    /// This function returns a syntax error, if the binding is a redeclaration.
    ///
    /// Note: This function only works at compile time!
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

    /// Initialize an immutable binding at compile time and return it's binding locator.
    ///
    /// Note: This function only works at compile time!
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
    /// Note: This function only works at compile time!
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

#[cfg(test)]
mod tests {
    use crate::exec;

    #[test]
    fn let_is_block_scoped() {
        let scenario = r#"
          {
            let bar = "bar";
          }

          try{
            bar;
          } catch (err) {
            err.message
          }
        "#;

        assert_eq!(&exec(scenario), "\"bar is not defined\"");
    }

    #[test]
    fn const_is_block_scoped() {
        let scenario = r#"
          {
            const bar = "bar";
          }

          try{
            bar;
          } catch (err) {
            err.message
          }
        "#;

        assert_eq!(&exec(scenario), "\"bar is not defined\"");
    }

    #[test]
    fn var_not_block_scoped() {
        let scenario = r#"
          {
            var bar = "bar";
          }
          bar == "bar";
        "#;

        assert_eq!(&exec(scenario), "true");
    }

    #[test]
    fn functions_use_declaration_scope() {
        let scenario = r#"
          function foo() {
            try {
                bar;
            } catch (err) {
                return err.message;
            }
          }
          {
            let bar = "bar";
            foo();
          }
        "#;

        assert_eq!(&exec(scenario), "\"bar is not defined\"");
    }

    #[test]
    fn set_outer_var_in_block_scope() {
        let scenario = r#"
          var bar;
          {
            bar = "foo";
          }
          bar == "foo";
        "#;

        assert_eq!(&exec(scenario), "true");
    }

    #[test]
    fn set_outer_let_in_block_scope() {
        let scenario = r#"
          let bar;
          {
            bar = "foo";
          }
          bar == "foo";
        "#;

        assert_eq!(&exec(scenario), "true");
    }
}
