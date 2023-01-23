//! Boa's implementation of ECMAScript's `Realm Records`
//!
//! Conceptually, a realm consists of a set of intrinsic objects, an ECMAScript global environment,
//! all of the ECMAScript code that is loaded within the scope of that global environment,
//! and other associated state and resources.
//!
//! A realm is represented in this implementation as a Realm struct with the fields specified from the spec.

use crate::{
    context::{intrinsics::Intrinsics, HostHooks},
    environments::{CompileTimeEnvironment, DeclarativeEnvironmentStack},
    object::{GlobalPropertyMap, JsObject, PropertyMap},
};
use boa_gc::{Gc, GcRefCell};
use boa_profiler::Profiler;

/// Representation of a Realm.
///
/// In the specification these are called Realm Records.
#[derive(Debug)]
pub struct Realm {
    pub(crate) intrinsics: Intrinsics,
    pub(crate) global_property_map: PropertyMap,
    pub(crate) environments: DeclarativeEnvironmentStack,
    pub(crate) compile_env: Gc<GcRefCell<CompileTimeEnvironment>>,
    global_object: JsObject,
    global_this: JsObject,
}

impl Realm {
    /// Create a new Realm.
    #[inline]
    pub fn create(hooks: &dyn HostHooks) -> Self {
        let _timer = Profiler::global().start_event("Realm::create", "realm");

        let intrinsics = Intrinsics::new();

        let global_object = hooks.create_global_object(&intrinsics);
        let global_this = hooks
            .create_global_this(&intrinsics)
            .unwrap_or_else(|| global_object.clone());

        let global_compile_environment =
            Gc::new(GcRefCell::new(CompileTimeEnvironment::new_global()));

        #[allow(unreachable_code)]
        Self {
            intrinsics,
            global_object,
            global_this,
            global_property_map: PropertyMap::default(),
            environments: DeclarativeEnvironmentStack::new(global_compile_environment.clone()),
            compile_env: global_compile_environment,
        }
    }

    pub(crate) const fn global_object(&self) -> &JsObject {
        &self.global_object
    }

    pub(crate) const fn global_this(&self) -> &JsObject {
        &self.global_this
    }

    pub(crate) fn global_bindings_mut(&mut self) -> &mut GlobalPropertyMap {
        self.global_property_map.string_property_map_mut()
    }

    /// Set the number of bindings on the global environment.
    pub(crate) fn set_global_binding_number(&mut self) {
        let binding_number = self.compile_env.borrow().num_bindings();
        self.environments.set_global_binding_number(binding_number);
    }
}
