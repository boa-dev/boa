//! Boa's implementation of ECMAScript's `Realm Records`
//!
//! Conceptually, a realm consists of a set of intrinsic objects, an ECMAScript global environment,
//! all of the ECMAScript code that is loaded within the scope of that global environment,
//! and other associated state and resources.
//!
//! A realm is represented in this implementation as a Realm struct with the fields specified from the spec.

use crate::{
    environments::{CompileTimeEnvironment, DeclarativeEnvironmentStack},
    object::{GlobalPropertyMap, JsObject, JsPrototype, ObjectData, PropertyMap},
};
use boa_gc::{Gc, GcCell};
use boa_profiler::Profiler;

/// Representation of a Realm.
///
/// In the specification these are called Realm Records.
#[derive(Debug)]
pub struct Realm {
    global_object: JsObject,
    pub(crate) global_extensible: bool,
    pub(crate) global_property_map: PropertyMap,
    pub(crate) global_prototype: JsPrototype,
    pub(crate) environments: DeclarativeEnvironmentStack,
    pub(crate) compile_env: Gc<GcCell<CompileTimeEnvironment>>,
}

impl Realm {
    /// Create a new Realm.
    #[inline]
    pub fn create(global_prototype: JsPrototype) -> Self {
        let _timer = Profiler::global().start_event("Realm::create", "realm");
        // Create brand new global object
        // Global has no prototype to pass None to new_obj
        // Allow identification of the global object easily
        let global_object = JsObject::from_proto_and_data(None, ObjectData::global());

        let global_compile_environment = Gc::new(GcCell::new(CompileTimeEnvironment::new_global()));

        Self {
            global_object,
            global_extensible: true,
            global_property_map: PropertyMap::default(),
            global_prototype,
            environments: DeclarativeEnvironmentStack::new(global_compile_environment.clone()),
            compile_env: global_compile_environment,
        }
    }

    pub(crate) const fn global_object(&self) -> &JsObject {
        &self.global_object
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
