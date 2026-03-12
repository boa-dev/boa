use crate::{
    Context, JsResult, JsString, JsSymbol, JsValue,
    object::{JsObject, PrivateName},
};
use boa_ast::scope::{BindingLocator, BindingLocatorScope, Scope};
use boa_gc::{Finalize, Gc, Trace, custom_trace};
use std::cell::Cell;
use thin_vec::ThinVec;

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

/// A single node in the captured environment chain.
///
/// Used only for environments inherited from closures (the "captured" region).
/// Locally-pushed environments live in a flat `Vec` instead.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct EnvironmentNode {
    env: Environment,
    parent: Option<Gc<EnvironmentNode>>,
}

/// A locally-pushed environment that has not yet been promoted to the GC heap.
///
/// Environments start as `Inline` when pushed during function execution. They are
/// promoted to `Promoted` (Gc-managed) only when a closure captures them via
/// [`EnvironmentStack::snapshot_for_closure`]. After promotion, both the outer scope
/// and the closure share the same `Gc<DeclarativeEnvironment>`.
#[derive(Debug)]
pub(crate) enum LocalEnvironment {
    /// Bindings stored inline — no `Gc` allocation.
    Inline {
        kind: DeclarativeEnvironmentKind,
        poisoned: Cell<bool>,
        with: bool,
    },
    /// Promoted to GC heap after closure capture.
    Promoted(Gc<DeclarativeEnvironment>),
    /// Object environment (for `with` statements).
    Object(JsObject),
    /// Temporary sentinel used during promotion. Never visible externally.
    _Vacant,
}

impl Finalize for LocalEnvironment {}

// SAFETY: We trace all GC-managed fields in each variant.
unsafe impl Trace for LocalEnvironment {
    custom_trace!(this, mark, {
        match this {
            Self::Inline { kind, .. } => mark(kind),
            Self::Promoted(gc) => mark(gc),
            Self::Object(obj) => mark(obj),
            Self::_Vacant => {}
        }
    });
}

impl Clone for LocalEnvironment {
    fn clone(&self) -> Self {
        match self {
            Self::Promoted(gc) => Self::Promoted(gc.clone()),
            Self::Object(obj) => Self::Object(obj.clone()),
            Self::Inline { .. } => {
                panic!("Cannot clone inline local environment; promote first")
            }
            Self::_Vacant => panic!("cannot clone vacant environment"),
        }
    }
}

impl LocalEnvironment {
    /// Returns the `DeclarativeEnvironmentKind` if this is a declarative environment.
    fn as_declarative_kind(&self) -> Option<&DeclarativeEnvironmentKind> {
        match self {
            Self::Inline { kind, .. } => Some(kind),
            Self::Promoted(gc) => Some(gc.kind()),
            Self::Object(_) | Self::_Vacant => None,
        }
    }

    /// Returns `true` if this is a declarative environment (not object).
    fn is_declarative(&self) -> bool {
        !matches!(self, Self::Object(_) | Self::_Vacant)
    }

    /// Returns the `poisoned` flag.
    fn poisoned(&self) -> bool {
        match self {
            Self::Inline { poisoned, .. } => poisoned.get(),
            Self::Promoted(gc) => gc.poisoned(),
            Self::Object(_) | Self::_Vacant => false,
        }
    }

    /// Returns the `with` flag.
    fn with(&self) -> bool {
        match self {
            Self::Inline { with, .. } => *with,
            Self::Promoted(gc) => gc.with(),
            Self::Object(_) | Self::_Vacant => false,
        }
    }

    /// Sets the `poisoned` flag.
    fn poison(&self) {
        match self {
            Self::Inline { poisoned, .. } => poisoned.set(true),
            Self::Promoted(gc) => gc.poison(),
            Self::Object(_) | Self::_Vacant => {}
        }
    }

    /// Returns `true` if this is a function environment.
    fn is_function(&self) -> bool {
        self.as_declarative_kind()
            .is_some_and(|k| matches!(k, DeclarativeEnvironmentKind::Function(_)))
    }

    /// Gets a binding value.
    fn get_binding(&self, index: u32) -> Option<JsValue> {
        match self {
            Self::Inline { kind, .. } => kind.get(index),
            Self::Promoted(gc) => gc.get(index),
            Self::Object(_) | Self::_Vacant => panic!("not a declarative environment"),
        }
    }

    /// Sets a binding value.
    fn set_binding(&self, index: u32, value: JsValue) {
        match self {
            Self::Inline { kind, .. } => kind.set(index, value),
            Self::Promoted(gc) => gc.set(index, value),
            Self::Object(_) | Self::_Vacant => panic!("not a declarative environment"),
        }
    }

    /// Promote this inline environment to a `Gc<DeclarativeEnvironment>`.
    ///
    /// If already promoted, returns the existing `Gc`. Panics for object environments.
    fn promote_to_gc(&mut self) -> Gc<DeclarativeEnvironment> {
        if let Self::Promoted(gc) = self {
            return gc.clone();
        }

        let old = std::mem::replace(self, Self::_Vacant);
        match old {
            Self::Inline {
                kind,
                poisoned,
                with,
            } => {
                let gc = Gc::new(DeclarativeEnvironment::new(kind, poisoned.get(), with));
                *self = Self::Promoted(gc.clone());
                gc
            }
            other => {
                *self = other;
                panic!("cannot promote non-declarative local environment");
            }
        }
    }
}

/// The environment stack holds all environments at runtime.
///
/// Split into two regions for performance:
/// - **Captured**: A linked list of `Gc<EnvironmentNode>` inherited from the closure
///   chain. These are already heap-allocated. Accessed via linked-list traversal.
/// - **Local**: A flat `Vec<LocalEnvironment>` for environments pushed during the
///   current function's execution. No `Gc` allocation on push. Accessed via O(1)
///   Vec indexing.
///
/// When a closure captures the environment ([`snapshot_for_closure`]), all local
/// inline environments are promoted to `Gc<DeclarativeEnvironment>` and linked into
/// the captured chain. After promotion, both the outer scope and the closure share
/// the same `Gc` pointers.
///
/// The global declarative environment is NOT stored here — it lives in the
/// [`crate::realm::Realm`] and is accessed via `frame.realm.environment()`.
#[derive(Debug)]
pub(crate) struct EnvironmentStack {
    /// Environments inherited from the closure chain (already Gc-managed).
    captured_tip: Option<Gc<EnvironmentNode>>,

    /// Number of environments in the captured chain.
    #[allow(dead_code)]
    captured_depth: u32,

    /// Environments pushed during this function's execution (flat, no Gc on push).
    local: Vec<LocalEnvironment>,

    private_stack: ThinVec<Gc<PrivateEnvironment>>,
}

impl Finalize for EnvironmentStack {}

// SAFETY: We trace all GC-managed fields.
unsafe impl Trace for EnvironmentStack {
    custom_trace!(this, mark, {
        mark(&this.captured_tip);
        for env in &this.local {
            mark(env);
        }
        mark(&this.private_stack);
    });
}

impl Clone for EnvironmentStack {
    fn clone(&self) -> Self {
        // Clone is used by:
        // - OrdinaryFunction::environments.clone() in function_call (local is always empty)
        // - Generator frame cloning (should promote_all first)
        // For safety, all Inline entries must have been promoted before cloning.
        Self {
            captured_tip: self.captured_tip.clone(),
            captured_depth: self.captured_depth,
            local: self.local.clone(),
            private_stack: self.private_stack.clone(),
        }
    }
}

/// Saved environment state for `pop_to_global` / `restore_from_saved`.
/// Used by indirect `eval` and `Function.prototype.toString` recompilation.
pub(crate) struct SavedEnvironments {
    captured_tip: Option<Gc<EnvironmentNode>>,
    captured_depth: u32,
    local: Vec<LocalEnvironment>,
}

/// A runtime environment (used in the captured linked-list chain).
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
}

impl EnvironmentStack {
    /// Create a new environment stack.
    pub(crate) fn new() -> Self {
        Self {
            captured_tip: None,
            captured_depth: 0,
            local: Vec::new(),
            private_stack: ThinVec::new(),
        }
    }

    /// Get the total number of environments (captured + local), not counting global.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.captured_depth as usize + self.local.len()
    }

    // ---- Push operations (allocation-free for Inline) ----

    /// Push a lexical environment and return its absolute index.
    pub(crate) fn push_lexical(
        &mut self,
        bindings_count: u32,
        global: &Gc<DeclarativeEnvironment>,
    ) -> u32 {
        let (poisoned, with) = self.compute_poisoned_with(global);
        let index = self.len() as u32;
        self.local.push(LocalEnvironment::Inline {
            kind: DeclarativeEnvironmentKind::Lexical(LexicalEnvironment::new(bindings_count)),
            poisoned: Cell::new(poisoned),
            with,
        });
        index
    }

    /// Push a function environment on the environments stack.
    pub(crate) fn push_function(
        &mut self,
        scope: Scope,
        function_slots: FunctionSlots,
        global: &Gc<DeclarativeEnvironment>,
    ) {
        let num_bindings = scope.num_bindings_non_local();
        let (poisoned, with) = self.compute_poisoned_with(global);
        self.local.push(LocalEnvironment::Inline {
            kind: DeclarativeEnvironmentKind::Function(FunctionEnvironment::new(
                num_bindings,
                function_slots,
                scope,
            )),
            poisoned: Cell::new(poisoned),
            with,
        });
    }

    /// Push an object environment (for `with` statements).
    pub(crate) fn push_object(&mut self, object: JsObject) {
        self.local.push(LocalEnvironment::Object(object));
    }

    /// Push a module environment on the environments stack.
    ///
    /// Module environments are immediately promoted to Gc because they are
    /// global singletons that need to be shared across module boundaries.
    pub(crate) fn push_module(&mut self, scope: Scope) {
        let num_bindings = scope.num_bindings_non_local();
        let gc = Gc::new(DeclarativeEnvironment::new(
            DeclarativeEnvironmentKind::Module(ModuleEnvironment::new(num_bindings, scope)),
            false,
            false,
        ));
        self.local.push(LocalEnvironment::Promoted(gc));
    }

    // ---- Pop / Truncate ----

    /// Pop the most recently pushed environment.
    #[track_caller]
    pub(crate) fn pop(&mut self) {
        if self.local.pop().is_some() {
            return;
        }
        // Fall back to popping from captured chain (shouldn't normally happen
        // within a single frame's execution).
        let node = self
            .captured_tip
            .as_ref()
            .expect("cannot pop empty environment chain");
        self.captured_tip = node.parent.clone();
        self.captured_depth -= 1;
    }

    /// Truncate current environments to the given total depth.
    pub(crate) fn truncate(&mut self, len: usize) {
        let captured = self.captured_depth as usize;
        if len >= captured {
            // Only truncate local environments.
            self.local.truncate(len - captured);
        } else {
            // Truncate all locals and some captured.
            self.local.clear();
            while (self.captured_depth as usize) > len {
                let node = self
                    .captured_tip
                    .as_ref()
                    .expect("depth > 0 implies tip is Some");
                self.captured_tip = node.parent.clone();
                self.captured_depth -= 1;
            }
        }
    }

    /// Save all current environments and clear the stack.
    /// Used by indirect eval and `Function.prototype.toString` recompilation.
    pub(crate) fn pop_to_global(&mut self) -> SavedEnvironments {
        SavedEnvironments {
            captured_tip: self.captured_tip.take(),
            captured_depth: std::mem::replace(&mut self.captured_depth, 0),
            local: std::mem::take(&mut self.local),
        }
    }

    /// Restore environments from a previous `pop_to_global` call.
    pub(crate) fn restore_from_saved(&mut self, saved: SavedEnvironments) {
        self.captured_tip = saved.captured_tip;
        self.captured_depth = saved.captured_depth;
        self.local = saved.local;
    }

    // ---- Access methods ----

    /// Get a binding value from the environment at `env_index`.
    #[track_caller]
    pub(crate) fn get_binding_value(&self, env_index: u32, binding_index: u32) -> Option<JsValue> {
        let captured = self.captured_depth as usize;
        let idx = env_index as usize;
        if idx >= captured {
            // Local environment — O(1) access.
            self.local[idx - captured].get_binding(binding_index)
        } else {
            // Captured environment — linked-list traversal.
            let env = self.get_captured(idx).expect("index in range");
            env.as_declarative()
                .expect("must be declarative")
                .get(binding_index)
        }
    }

    /// Set a binding value in the environment at `env_index`.
    #[track_caller]
    pub(crate) fn set_binding_value(&self, env_index: u32, binding_index: u32, value: JsValue) {
        let captured = self.captured_depth as usize;
        let idx = env_index as usize;
        if idx >= captured {
            self.local[idx - captured].set_binding(binding_index, value);
        } else {
            let env = self.get_captured(idx).expect("index in range");
            env.as_declarative()
                .expect("must be declarative")
                .set(binding_index, value);
        }
    }

    /// Check if the environment at `env_index` is an object environment.
    pub(crate) fn is_object_env(&self, env_index: u32) -> bool {
        let captured = self.captured_depth as usize;
        let idx = env_index as usize;
        if idx >= captured {
            matches!(self.local[idx - captured], LocalEnvironment::Object(_))
        } else {
            matches!(self.get_captured(idx), Some(Environment::Object(_)))
        }
    }

    /// Get the object from an object environment at `env_index`.
    pub(crate) fn get_object_env(&self, env_index: u32) -> Option<&JsObject> {
        let captured = self.captured_depth as usize;
        let idx = env_index as usize;
        if idx >= captured {
            match &self.local[idx - captured] {
                LocalEnvironment::Object(obj) => Some(obj),
                _ => None,
            }
        } else {
            match self.get_captured(idx)? {
                Environment::Object(obj) => Some(obj),
                Environment::Declarative(_) => None,
            }
        }
    }

    /// Get the `DeclarativeEnvironmentKind` at the given absolute index.
    #[allow(dead_code)]
    pub(crate) fn get_declarative_kind(
        &self,
        env_index: u32,
    ) -> Option<&DeclarativeEnvironmentKind> {
        let captured = self.captured_depth as usize;
        let idx = env_index as usize;
        if idx >= captured {
            self.local[idx - captured].as_declarative_kind()
        } else {
            self.get_captured(idx)?.as_declarative().map(|gc| gc.kind())
        }
    }

    /// Get the `Gc<DeclarativeEnvironment>` at the given index, promoting if needed.
    #[allow(dead_code)]
    pub(crate) fn get_declarative_gc(
        &mut self,
        env_index: u32,
    ) -> Option<Gc<DeclarativeEnvironment>> {
        let captured = self.captured_depth as usize;
        let idx = env_index as usize;
        if idx >= captured {
            let local = &mut self.local[idx - captured];
            if local.is_declarative() {
                Some(local.promote_to_gc())
            } else {
                None
            }
        } else {
            self.get_captured(idx)?.as_declarative().cloned()
        }
    }

    /// Get the environment at the given absolute index in the captured chain.
    fn get_captured(&self, index: usize) -> Option<&Environment> {
        let depth = self.captured_depth as usize;
        if index >= depth {
            return None;
        }
        let steps = depth - 1 - index;
        let mut current = self.captured_tip.as_deref()?;
        for _ in 0..steps {
            current = current.parent.as_deref()?;
        }
        Some(&current.env)
    }

    // ---- This environment ----

    /// `GetThisEnvironment`
    ///
    /// Returns the environment that currently provides a `this` binding.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getthisenvironment
    pub(crate) fn get_this_environment<'a>(
        &'a self,
        global: &'a Gc<DeclarativeEnvironment>,
    ) -> &'a DeclarativeEnvironmentKind {
        // Search local environments first (tip to base).
        for local in self.local.iter().rev() {
            if let Some(kind) = local.as_declarative_kind()
                && kind.has_this_binding()
            {
                return kind;
            }
        }
        // Then search captured chain.
        for (env, _) in self.iter_captured() {
            if let Some(decl) = env.as_declarative().filter(|decl| decl.has_this_binding()) {
                return decl.kind();
            }
        }
        global.kind()
    }

    /// `GetThisBinding`
    ///
    /// Returns the current `this` binding of the environment.
    pub(crate) fn get_this_binding(&self) -> JsResult<Option<JsValue>> {
        // Search local environments first.
        for local in self.local.iter().rev() {
            match local {
                LocalEnvironment::Inline { kind, .. } => {
                    if let Some(this) = kind.get_this_binding()? {
                        return Ok(Some(this));
                    }
                }
                LocalEnvironment::Promoted(gc) => {
                    if let Some(this) = gc.get_this_binding()? {
                        return Ok(Some(this));
                    }
                }
                LocalEnvironment::Object(_) | LocalEnvironment::_Vacant => {}
            }
        }
        // Then captured chain.
        for (env, _) in self.iter_captured() {
            if let Environment::Declarative(decl) = env
                && let Some(this) = decl.get_this_binding()?
            {
                return Ok(Some(this));
            }
        }
        Ok(None)
    }

    // ---- Outer function environment (for eval) ----

    /// Gets the next outer function environment.
    pub(crate) fn outer_function_environment(
        &mut self,
    ) -> Option<(Gc<DeclarativeEnvironment>, Scope)> {
        // Search local environments first. Force-promote if found.
        for (i, local) in self.local.iter().enumerate().rev() {
            if local.is_function() {
                let gc = self.local[i].promote_to_gc();
                let scope = gc
                    .kind()
                    .as_function()
                    .expect("must be function")
                    .compile()
                    .clone();
                return Some((gc, scope));
            }
        }
        // Then captured chain.
        let mut current = self.captured_tip.as_deref();
        while let Some(node) = current {
            if let Some(decl) = node.env.as_declarative()
                && let Some(function_env) = decl.kind().as_function()
            {
                return Some((decl.clone(), function_env.compile().clone()));
            }
            current = node.parent.as_deref();
        }
        None
    }

    // ---- Current tip checks ----

    /// Check if the tip (most recently pushed) environment is a declarative
    /// environment that is not poisoned and not inside a `with`.
    ///
    /// Used as a fast-path check in `find_runtime_binding` and similar.
    pub(crate) fn current_is_clean_declarative(&self, global: &Gc<DeclarativeEnvironment>) -> bool {
        if let Some(local) = self.local.last() {
            local.is_declarative() && !local.poisoned() && !local.with()
        } else if let Some(node) = self.captured_tip.as_deref() {
            node.env
                .as_declarative()
                .is_some_and(|d| !d.poisoned() && !d.with())
        } else {
            // Stack is empty, check global.
            !global.poisoned() && !global.with()
        }
    }

    /// Check if the tip is a declarative environment that is not inside a `with`.
    pub(crate) fn current_is_not_with(&self, global: &Gc<DeclarativeEnvironment>) -> bool {
        if let Some(local) = self.local.last() {
            local.is_declarative() && !local.with()
        } else if let Some(node) = self.captured_tip.as_deref() {
            node.env.as_declarative().is_some_and(|d| !d.with())
        } else {
            !global.with()
        }
    }

    /// Get the `Gc<DeclarativeEnvironment>` of the most recently pushed declarative
    /// environment. Promotes if needed.
    pub(crate) fn current_declarative_gc(
        &mut self,
        global: &Gc<DeclarativeEnvironment>,
    ) -> Option<Gc<DeclarativeEnvironment>> {
        if let Some(local) = self.local.last_mut() {
            if local.is_declarative() {
                return Some(local.promote_to_gc());
            }
            return None;
        }
        if let Some(node) = self.captured_tip.as_deref() {
            return node.env.as_declarative().cloned();
        }
        Some(global.clone())
    }

    /// Get the `DeclarativeEnvironmentKind` of the most recently pushed environment.
    pub(crate) fn current_declarative_kind<'a>(
        &'a self,
        global: &'a Gc<DeclarativeEnvironment>,
    ) -> Option<&'a DeclarativeEnvironmentKind> {
        if let Some(local) = self.local.last() {
            return local.as_declarative_kind();
        }
        if let Some(node) = self.captured_tip.as_deref() {
            return node.env.as_declarative().map(|gc| gc.kind());
        }
        Some(global.kind())
    }

    // ---- Poison ----

    /// Mark that there may be added bindings from the current environment to the next function
    /// environment.
    pub(crate) fn poison_until_last_function(&mut self, global: &Gc<DeclarativeEnvironment>) {
        // Poison local environments from tip toward base.
        for local in self.local.iter().rev() {
            if local.is_declarative() {
                local.poison();
                if local.is_function() {
                    return;
                }
            }
        }
        // Then captured chain.
        let mut current = self.captured_tip.as_deref();
        while let Some(node) = current {
            if let Some(decl) = node.env.as_declarative() {
                decl.poison();
                if decl.is_function() {
                    return;
                }
            }
            current = node.parent.as_deref();
        }
        global.poison();
    }

    // ---- Binding value helpers ----

    /// Set the value of a lexical binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[track_caller]
    pub(crate) fn put_lexical_value(
        &mut self,
        environment: BindingLocatorScope,
        binding_index: u32,
        value: JsValue,
        global: &Gc<DeclarativeEnvironment>,
    ) {
        match environment {
            BindingLocatorScope::GlobalObject | BindingLocatorScope::GlobalDeclarative => {
                global.set(binding_index, value);
            }
            BindingLocatorScope::Stack(index) => {
                self.set_binding_value(index, binding_index, value);
            }
        }
    }

    /// Set the value of a binding if it is uninitialized.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[track_caller]
    pub(crate) fn put_value_if_uninitialized(
        &mut self,
        environment: BindingLocatorScope,
        binding_index: u32,
        value: JsValue,
        global: &Gc<DeclarativeEnvironment>,
    ) {
        match environment {
            BindingLocatorScope::GlobalObject | BindingLocatorScope::GlobalDeclarative => {
                if global.get(binding_index).is_none() {
                    global.set(binding_index, value);
                }
            }
            BindingLocatorScope::Stack(index) => {
                if self.get_binding_value(index, binding_index).is_none() {
                    self.set_binding_value(index, binding_index, value);
                }
            }
        }
    }

    // ---- Object environment checks ----

    /// Indicate if the current environment stack has an object environment.
    pub(crate) fn has_object_environment(&self) -> bool {
        for local in self.local.iter().rev() {
            if matches!(local, LocalEnvironment::Object(_)) {
                return true;
            }
        }
        for (env, _) in self.iter_captured() {
            if matches!(env, Environment::Object(_)) {
                return true;
            }
        }
        false
    }

    // ---- Closure capture ----

    /// Create an `EnvironmentStack` snapshot suitable for storing in a closure.
    ///
    /// Promotes all inline local environments to `Gc<DeclarativeEnvironment>` and
    /// builds a linked-list chain. Both the outer scope and the closure share the
    /// same `Gc` pointers after promotion.
    pub(crate) fn snapshot_for_closure(&mut self) -> EnvironmentStack {
        // Build a linked list from captured_tip + all locals.
        let mut tip = self.captured_tip.clone();
        let mut depth = self.captured_depth;

        for local in &mut self.local {
            match local {
                LocalEnvironment::Inline {
                    kind: _,
                    poisoned: _,
                    with: _,
                } => {
                    let gc = local.promote_to_gc();
                    tip = Some(Gc::new(EnvironmentNode {
                        env: Environment::Declarative(gc),
                        parent: tip,
                    }));
                    depth += 1;
                }
                LocalEnvironment::Promoted(gc) => {
                    tip = Some(Gc::new(EnvironmentNode {
                        env: Environment::Declarative(gc.clone()),
                        parent: tip,
                    }));
                    depth += 1;
                }
                LocalEnvironment::Object(obj) => {
                    tip = Some(Gc::new(EnvironmentNode {
                        env: Environment::Object(obj.clone()),
                        parent: tip,
                    }));
                    depth += 1;
                }
                LocalEnvironment::_Vacant => panic!("vacant environment in stack"),
            }
        }

        EnvironmentStack {
            captured_tip: tip,
            captured_depth: depth,
            local: Vec::new(),
            private_stack: self.private_stack.clone(),
        }
    }

    /// Promote all inline local environments to Gc.
    ///
    /// Call this before cloning the `EnvironmentStack` (e.g., for generators).
    #[allow(dead_code)]
    pub(crate) fn promote_all(&mut self) {
        for local in &mut self.local {
            if matches!(local, LocalEnvironment::Inline { .. }) {
                local.promote_to_gc();
            }
        }
    }

    // ---- Private environments ----

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
    pub(crate) fn resolve_private_identifier(&self, identifier: JsString) -> Option<PrivateName> {
        for environment in self.private_stack.iter().rev() {
            if environment.descriptions().contains(&identifier) {
                return Some(PrivateName::new(identifier, environment.id()));
            }
        }
        None
    }

    /// Return all private name descriptions in all private environments.
    pub(crate) fn private_name_descriptions(&self) -> Vec<&JsString> {
        let mut names = Vec::new();
        for environment in self.private_stack.iter().rev() {
            for name in environment.descriptions() {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        names
    }

    // ---- Private helpers ----

    /// Iterate captured chain from tip toward root.
    fn iter_captured(&self) -> CapturedChainIter<'_> {
        CapturedChainIter {
            current: self.captured_tip.as_deref(),
            index: self.captured_depth,
        }
    }

    /// Compute the `(poisoned, with)` flags for a new environment.
    fn compute_poisoned_with(&self, global: &Gc<DeclarativeEnvironment>) -> (bool, bool) {
        // Check if the tip is an object environment (for `with`).
        let with = if let Some(local) = self.local.last() {
            matches!(local, LocalEnvironment::Object(_))
        } else if let Some(node) = self.captured_tip.as_deref() {
            node.env.as_declarative().is_none()
        } else {
            false
        };

        // Find the nearest declarative environment to check poisoned/with.
        // Search local first, then captured.
        for local in self.local.iter().rev() {
            if local.is_declarative() {
                return (local.poisoned(), with || local.with());
            }
        }
        for (env, _) in self.iter_captured() {
            if let Some(decl) = env.as_declarative() {
                return (decl.poisoned(), with || decl.with());
            }
        }
        (global.poisoned(), with || global.with())
    }
}

/// Iterator over the captured linked-list chain from tip toward root.
struct CapturedChainIter<'a> {
    current: Option<&'a EnvironmentNode>,
    index: u32,
}

impl<'a> Iterator for CapturedChainIter<'a> {
    type Item = (&'a Environment, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.index = self
            .index
            .checked_sub(1)
            .expect("iterator advanced past root");
        self.current = node.parent.as_deref();
        Some((&node.env, self.index))
    }
}

impl Context {
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
        let global = self.vm.frame().realm.environment();
        if self
            .vm
            .frame()
            .environments
            .current_is_clean_declarative(global)
        {
            return Ok(());
        }

        let (global_scope, min_index) = match locator.scope() {
            BindingLocatorScope::GlobalObject | BindingLocatorScope::GlobalDeclarative => (true, 0),
            BindingLocatorScope::Stack(index) => (false, index),
        };
        let max_index = self.vm.frame().environments.len() as u32;

        for index in (min_index..max_index).rev() {
            if self.vm.frame().environments.is_object_env(index) {
                let obj = self
                    .vm
                    .frame()
                    .environments
                    .get_object_env(index)
                    .expect("must be object env")
                    .clone();
                let key = locator.name().clone();
                if obj.has_property(key.clone(), self)? {
                    if let Some(unscopables) = obj.get(JsSymbol::unscopables(), self)?.as_object()
                        && unscopables.get(key.clone(), self)?.to_boolean()
                    {
                        continue;
                    }
                    locator.set_scope(BindingLocatorScope::Stack(index));
                    return Ok(());
                }
            } else {
                // Declarative environment.
                let poisoned = {
                    let captured = self.vm.frame().environments.captured_depth as usize;
                    let idx = index as usize;
                    if idx >= captured {
                        let local = &self.vm.frame().environments.local[idx - captured];
                        (
                            local.poisoned(),
                            local.with(),
                            local.is_function(),
                            local.as_declarative_kind(),
                        )
                    } else {
                        let env = self
                            .vm
                            .frame()
                            .environments
                            .get_captured(idx)
                            .and_then(Environment::as_declarative);
                        if let Some(env) = env {
                            (
                                env.poisoned(),
                                env.with(),
                                env.is_function(),
                                Some(env.kind()),
                            )
                        } else {
                            continue;
                        }
                    }
                };
                let (is_poisoned, is_with, is_function, kind) = poisoned;
                if is_poisoned {
                    if let Some(kind) = kind
                        && let Some(func_env) = kind.as_function()
                        && let Some(b) = func_env.compile().get_binding(locator.name())
                    {
                        locator.set_scope(b.scope());
                        locator.set_binding_index(b.binding_index());
                        return Ok(());
                    }
                } else if !is_with {
                    return Ok(());
                }
                let _ = is_function;
            }
        }

        if global_scope
            && self.realm().environment().poisoned()
            && let Some(b) = self.realm().scope().get_binding(locator.name())
        {
            locator.set_scope(b.scope());
            locator.set_binding_index(b.binding_index());
        }

        Ok(())
    }

    /// Finds the object environment that contains the binding and returns the `this` value of the object environment.
    pub(crate) fn this_from_object_environment_binding(
        &mut self,
        locator: &BindingLocator,
    ) -> JsResult<Option<JsObject>> {
        let global = self.vm.frame().realm.environment();
        if self.vm.frame().environments.current_is_not_with(global) {
            return Ok(None);
        }

        let min_index = match locator.scope() {
            BindingLocatorScope::GlobalObject | BindingLocatorScope::GlobalDeclarative => 0,
            BindingLocatorScope::Stack(index) => index,
        };
        let max_index = self.vm.frame().environments.len() as u32;

        for index in (min_index..max_index).rev() {
            if self.vm.frame().environments.is_object_env(index) {
                let o = self
                    .vm
                    .frame()
                    .environments
                    .get_object_env(index)
                    .expect("must be object env")
                    .clone();
                let key = locator.name().clone();
                if o.has_property(key.clone(), self)? {
                    if let Some(unscopables) = o.get(JsSymbol::unscopables(), self)?.as_object()
                        && unscopables.get(key.clone(), self)?.to_boolean()
                    {
                        continue;
                    }
                    return Ok(Some(o));
                }
            } else {
                // Declarative environment.
                let captured = self.vm.frame().environments.captured_depth as usize;
                let idx = index as usize;
                if idx >= captured {
                    let local = &self.vm.frame().environments.local[idx - captured];
                    if local.poisoned() {
                        if let Some(kind) = local.as_declarative_kind()
                            && let Some(func_env) = kind.as_function()
                            && func_env.compile().get_binding(locator.name()).is_some()
                        {
                            break;
                        }
                    } else if !local.with() {
                        break;
                    }
                } else {
                    let env = self
                        .vm
                        .frame()
                        .environments
                        .get_captured(idx)
                        .and_then(Environment::as_declarative);
                    if let Some(env) = env {
                        if env.poisoned() {
                            if let Some(func_env) = env.kind().as_function()
                                && func_env.compile().get_binding(locator.name()).is_some()
                            {
                                break;
                            }
                        } else if !env.with() {
                            break;
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Checks if the binding pointed by `locator` is initialized.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    pub(crate) fn is_initialized_binding(&mut self, locator: &BindingLocator) -> JsResult<bool> {
        match locator.scope() {
            BindingLocatorScope::GlobalObject => {
                let key = locator.name().clone();
                let obj = self.global_object();
                obj.has_property(key, self)
            }
            BindingLocatorScope::GlobalDeclarative => {
                let env = self.vm.frame().realm.environment();
                Ok(env.get(locator.binding_index()).is_some())
            }
            BindingLocatorScope::Stack(index) => {
                if self.vm.frame().environments.is_object_env(index) {
                    let obj = self
                        .vm
                        .frame()
                        .environments
                        .get_object_env(index)
                        .expect("must be object env")
                        .clone();
                    let key = locator.name().clone();
                    obj.has_property(key, self)
                } else {
                    Ok(self
                        .vm
                        .frame()
                        .environments
                        .get_binding_value(index, locator.binding_index())
                        .is_some())
                }
            }
        }
    }

    /// Get the value of a binding.
    ///
    /// # Panics
    ///
    /// Panics if the environment or binding index are out of range.
    #[track_caller]
    pub(crate) fn get_binding(&mut self, locator: &BindingLocator) -> JsResult<Option<JsValue>> {
        match locator.scope() {
            BindingLocatorScope::GlobalObject => {
                let key = locator.name().clone();
                let obj = self.global_object();
                obj.try_get(key, self)
            }
            BindingLocatorScope::GlobalDeclarative => {
                let env = self.vm.frame().realm.environment();
                Ok(env.get(locator.binding_index()))
            }
            BindingLocatorScope::Stack(index) => {
                if self.vm.frame().environments.is_object_env(index) {
                    let obj = self
                        .vm
                        .frame()
                        .environments
                        .get_object_env(index)
                        .expect("must be object env")
                        .clone();
                    let key = locator.name().clone();
                    obj.get(key, self).map(Some)
                } else {
                    Ok(self
                        .vm
                        .frame()
                        .environments
                        .get_binding_value(index, locator.binding_index()))
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
        locator: &BindingLocator,
        value: JsValue,
        strict: bool,
    ) -> JsResult<()> {
        match locator.scope() {
            BindingLocatorScope::GlobalObject => {
                let key = locator.name().clone();
                let obj = self.global_object();
                obj.set(key, value, strict, self)?;
            }
            BindingLocatorScope::GlobalDeclarative => {
                let env = self.vm.frame().realm.environment();
                env.set(locator.binding_index(), value);
            }
            BindingLocatorScope::Stack(index) => {
                if self.vm.frame().environments.is_object_env(index) {
                    let obj = self
                        .vm
                        .frame()
                        .environments
                        .get_object_env(index)
                        .expect("must be object env")
                        .clone();
                    let key = locator.name().clone();
                    obj.set(key, value, strict, self)?;
                } else {
                    self.vm.frame().environments.set_binding_value(
                        index,
                        locator.binding_index(),
                        value,
                    );
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
    pub(crate) fn delete_binding(&mut self, locator: &BindingLocator) -> JsResult<bool> {
        match locator.scope() {
            BindingLocatorScope::GlobalObject => {
                let key = locator.name().clone();
                let obj = self.global_object();
                obj.__delete__(&key.into(), &mut self.into())
            }
            BindingLocatorScope::GlobalDeclarative => Ok(false),
            BindingLocatorScope::Stack(index) => {
                if self.vm.frame().environments.is_object_env(index) {
                    let obj = self
                        .vm
                        .frame()
                        .environments
                        .get_object_env(index)
                        .expect("must be object env")
                        .clone();
                    let key = locator.name().clone();
                    obj.__delete__(&key.into(), &mut self.into())
                } else {
                    Ok(false)
                }
            }
        }
    }
}
