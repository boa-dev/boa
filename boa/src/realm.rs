//! Conceptually, a realm consists of a set of intrinsic objects, an ECMAScript global environment,
//! all of the ECMAScript code that is loaded within the scope of that global environment,
//! and other associated state and resources.
//!
//! A realm is represented in this implementation as a Realm struct with the fields specified from the spec.

use crate::object::Object;
use crate::{
    environment::{
        declarative_environment_record::DeclarativeEnvironmentRecord,
        global_environment_record::GlobalEnvironmentRecord,
        lexical_environment::LexicalEnvironment,
        object_environment_record::ObjectEnvironmentRecord,
    },
    BoaProfiler, Value,
};
use gc::{Gc, GcCell};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;

/// Representation of a Realm.
///
/// In the specification these are called Realm Records.
#[derive(Debug)]
pub struct Realm {
    pub global_object: Value,
    pub global_env: Gc<GcCell<GlobalEnvironmentRecord>>,
    pub environment: RefCell<LexicalEnvironment>,
}

impl Realm {
    pub fn create() -> Self {
        let _timer = BoaProfiler::global().start_event("Realm::create", "realm");
        // Create brand new global object
        // Global has no prototype to pass None to new_obj
        let global = Value::from(Object::default());

        // Allow identification of the global object easily
        global.set_data(crate::object::ObjectData::Global);

        // We need to clone the global here because its referenced from separate places (only pointer is cloned)
        let global_env = new_global_environment(global.clone(), global.clone());

        Self {
            global_object: global.clone(),
            global_env,
            environment: RefCell::new(LexicalEnvironment::new(global)),
        }
    }
}

// Similar to new_global_environment in lexical_environment, except we need to return a GlobalEnvirionment
fn new_global_environment(global: Value, this_value: Value) -> Gc<GcCell<GlobalEnvironmentRecord>> {
    let obj_rec = ObjectEnvironmentRecord {
        bindings: global,
        outer_env: None,
        /// Object Environment Records created for with statements (13.11)
        /// can provide their binding object as an implicit this value for use in function calls.
        /// The capability is controlled by a withEnvironment Boolean value that is associated
        /// with each object Environment Record. By default, the value of withEnvironment is false
        /// for any object Environment Record.
        with_environment: false,
    };

    let dcl_rec = DeclarativeEnvironmentRecord {
        env_rec: FxHashMap::default(),
        outer_env: None,
    };

    Gc::new(GcCell::new(GlobalEnvironmentRecord {
        object_record: obj_rec,
        global_this_binding: this_value,
        declarative_record: dcl_rec,
        var_names: FxHashSet::default(),
    }))
}
