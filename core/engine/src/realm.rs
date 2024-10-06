//! Boa's implementation of ECMAScript's `Realm Records`
//!
//! Conceptually, a realm consists of a set of intrinsic objects, an ECMAScript global environment,
//! all of the ECMAScript code that is loaded within the scope of that global environment,
//! and other associated state and resources.
//!
//! A realm is represented in this implementation as a Realm struct with the fields specified from the spec.

use std::any::TypeId;

use boa_ast::scope::Scope;
use rustc_hash::FxHashMap;

use crate::{
    class::Class,
    context::{
        intrinsics::{Intrinsics, StandardConstructor},
        HostHooks,
    },
    environments::DeclarativeEnvironment,
    module::Module,
    object::shape::RootShape,
    HostDefined, JsObject, JsResult, JsString,
};
use boa_gc::{Finalize, Gc, GcRef, GcRefCell, GcRefMut, Trace};
use boa_profiler::Profiler;

/// Representation of a Realm.
///
/// In the specification these are called Realm Records.
#[derive(Clone, Trace, Finalize)]
pub struct Realm {
    /// The inner data of the realm, which includes the intrinsics, environment,
    /// global object, and other realm-specific information.
    pub inner: Gc<RealmInner>,
}

impl Eq for Realm {}

impl PartialEq for Realm {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(&self.inner, &other.inner)
    }
}

impl std::fmt::Debug for Realm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Realm")
            .field("intrinsics", &self.inner.intrinsics)
            .field("environment", &self.inner.environment)
            .field("global_object", &self.inner.global_object)
            .field("global_this", &self.inner.global_this)
            .finish()
    }
}

#[derive(Trace, Finalize)]

/// The inner data of a Realm.
///
/// This struct contains all the realm-specific information, including the intrinsics,
/// environment, global object, and other necessary data for the execution context.
pub struct RealmInner {
    intrinsics: Intrinsics,

    /// The global declarative environment of this realm.
    environment: Gc<DeclarativeEnvironment>,

    /// The global scope of this realm.
    /// This is directly related to the global declarative environment.
    // Safety: Nothing in `Scope` needs tracing.
    #[unsafe_ignore_trace]
    scope: Scope,

    global_object: JsObject,
    global_this: JsObject,
    template_map: GcRefCell<FxHashMap<u64, JsObject>>,
    loaded_modules: GcRefCell<FxHashMap<JsString, Module>>,
    host_classes: GcRefCell<FxHashMap<TypeId, StandardConstructor>>,
    host_defined: GcRefCell<HostDefined>,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for RealmInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealmInner")
            .field("intrinsics", &self.intrinsics)
            .field("environment", &self.environment)
            .field("global_object", &self.global_object)
            .field("global_this", &self.global_this)
            .field("template_map", &self.template_map)
            .field("loaded_modules", &self.loaded_modules)
            .field("host_classes", &self.host_classes)
            .finish()
    }
}
impl Realm {
    /// Create a new [`Realm`].
    #[inline]
    pub fn create(hooks: &dyn HostHooks, root_shape: &RootShape) -> JsResult<Self> {
        let _timer = Profiler::global().start_event("Realm::create", "realm");

        // Use Gc::new_cyclic to create the Realm with a cyclic reference
        let inner = Gc::new_cyclic(|weak_realm| {
            // Initialize intrinsics with a reference to the weak_realm
            let intrinsics = Intrinsics::uninit(root_shape, weak_realm)
                .expect("failed to create the realm intrinsics");

            let global_object = hooks.create_global_object(&intrinsics);
            let global_this = hooks
                .create_global_this(&intrinsics)
                .unwrap_or_else(|| global_object.clone());
            let environment = Gc::new(DeclarativeEnvironment::global());
            let scope = Scope::new_global();

            RealmInner {
                intrinsics,
                environment,
                scope,
                global_object,
                global_this,
                template_map: GcRefCell::default(),
                loaded_modules: GcRefCell::default(),
                host_classes: GcRefCell::default(),
                host_defined: GcRefCell::default(),
            }
        });

        let realm = Self { inner };

        realm.initialize();

        Ok(realm)
    }

    /// Gets the intrinsics of this `Realm`.
    #[inline]
    #[must_use]
    pub fn intrinsics(&self) -> &Intrinsics {
        &self.inner.intrinsics
    }

    /// Returns an immutable reference to the [`ECMAScript specification`][spec] defined
    /// [`\[\[\HostDefined]\]`][`HostDefined`] field of the [`Realm`].
    ///
    /// [spec]: https://tc39.es/ecma262/#table-realm-record-fields
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is mutably borrowed.
    #[inline]
    #[must_use]
    pub fn host_defined(&self) -> GcRef<'_, HostDefined> {
        self.inner.host_defined.borrow()
    }

    /// Returns a mutable reference to [`ECMAScript specification`][spec] defined
    /// [`\[\[\HostDefined]\]`][`HostDefined`] field of the [`Realm`].
    ///
    /// [spec]: https://tc39.es/ecma262/#table-realm-record-fields
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is borrowed.
    #[inline]
    #[must_use]
    pub fn host_defined_mut(&self) -> GcRefMut<'_, HostDefined> {
        self.inner.host_defined.borrow_mut()
    }

    /// Checks if this `Realm` has the class `C` registered into its class map.
    #[must_use]
    pub fn has_class<C: Class>(&self) -> bool {
        self.inner
            .host_classes
            .borrow()
            .contains_key(&TypeId::of::<C>())
    }

    /// Gets the constructor and prototype of the class `C` if it is registered in the class map.
    #[must_use]
    pub fn get_class<C: Class>(&self) -> Option<StandardConstructor> {
        self.inner
            .host_classes
            .borrow()
            .get(&TypeId::of::<C>())
            .cloned()
    }

    pub(crate) fn environment(&self) -> &Gc<DeclarativeEnvironment> {
        &self.inner.environment
    }

    /// Returns the scope of this realm.
    #[must_use]
    pub fn scope(&self) -> &Scope {
        &self.inner.scope
    }

    pub(crate) fn global_object(&self) -> &JsObject {
        &self.inner.global_object
    }

    pub(crate) fn global_this(&self) -> &JsObject {
        &self.inner.global_this
    }

    pub(crate) fn loaded_modules(&self) -> &GcRefCell<FxHashMap<JsString, Module>> {
        &self.inner.loaded_modules
    }

    /// Resizes the number of bindings on the global environment.
    pub(crate) fn resize_global_env(&self) {
        let binding_number = self.scope().num_bindings();
        let env = self
            .environment()
            .kind()
            .as_global()
            .expect("Realm should only store global environments")
            .poisonable_environment();
        let mut bindings = env.bindings().borrow_mut();

        if bindings.len() < binding_number as usize {
            bindings.resize(binding_number as usize, None);
        }
    }

    pub(crate) fn push_template(&self, site: u64, template: JsObject) {
        self.inner.template_map.borrow_mut().insert(site, template);
    }

    pub(crate) fn lookup_template(&self, site: u64) -> Option<JsObject> {
        self.inner.template_map.borrow().get(&site).cloned()
    }

    pub(crate) fn register_class<C: Class>(&self, spec: StandardConstructor) {
        self.inner
            .host_classes
            .borrow_mut()
            .insert(TypeId::of::<C>(), spec);
    }

    pub(crate) fn unregister_class<C: Class>(&self) -> Option<StandardConstructor> {
        self.inner
            .host_classes
            .borrow_mut()
            .remove(&TypeId::of::<C>())
    }

    pub(crate) fn addr(&self) -> *const () {
        let ptr: *const _ = &*self.inner;
        ptr.cast()
    }
}
