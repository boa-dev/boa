//! Conceptually, a realm consists of a set of intrinsic objects, an ECMAScript global environment,
//! all of the ECMAScript code that is loaded within the scope of that global environment,
//! and other associated state and resources.
//!
//! A realm is represented in this implementation as a Realm struct with the fields specified from the spec.

use crate::{
    environment::{
        global_environment_record::GlobalEnvironmentRecord, lexical_environment::LexicalEnvironment,
    },
    gc::Gc,
    object::{JsObject, ObjectData},
    BoaProfiler,
};

/// Representation of a Realm.
///
/// In the specification these are called Realm Records.
#[derive(Debug)]
pub struct Realm {
    pub global_object: JsObject,
    pub global_env: Gc<GlobalEnvironmentRecord>,
    pub environment: LexicalEnvironment,
}

impl Realm {
    #[allow(clippy::field_reassign_with_default)]
    pub fn create() -> Self {
        let _timer = BoaProfiler::global().start_event("Realm::create", "realm");
        // Create brand new global object
        // Global has no prototype to pass None to new_obj
        // Allow identification of the global object easily
        let gc_global = JsObject::from_proto_and_data(None, ObjectData::global());

        // We need to clone the global here because its referenced from separate places (only pointer is cloned)
        let global_env = GlobalEnvironmentRecord::new(gc_global.clone(), gc_global.clone());

        Self {
            global_object: gc_global.clone(),
            global_env: Gc::new(global_env),
            environment: LexicalEnvironment::new(gc_global),
        }
    }
}
