//! Conceptually, a realm consists of a set of intrinsic objects, an ECMAScript global environment,
//! all of the ECMAScript code that is loaded within the scope of that global environment,
//! and other associated state and resources.
//!
//!A realm is represented in this implementation as a Realm struct with the fields specified from the spec
use crate::{
    builtins::{
        array, boolean, console, function,
        function::NativeFunctionData,
        json, math, number, object, regexp, string, symbol,
        value::{ToValue, Value, ValueData},
    },
    environment::{
        declarative_environment_record::DeclarativeEnvironmentRecord,
        global_environment_record::GlobalEnvironmentRecord,
        lexical_environment::LexicalEnvironment,
        object_environment_record::ObjectEnvironmentRecord,
    },
};
use gc::{Gc, GcCell};
use std::collections::{hash_map::HashMap, hash_set::HashSet};

/// Representation of a Realm.   
/// In the specification these are called Realm Records.
#[derive(Debug)]
pub struct Realm {
    pub global_obj: Value,
    pub global_env: Gc<GcCell<Box<GlobalEnvironmentRecord>>>,
    pub environment: LexicalEnvironment,
}

impl Realm {
    pub fn create() -> Self {
        // Create brand new global object
        // Global has no prototype to pass None to new_obj
        let global = ValueData::new_obj(None);
        // We need to clone the global here because its referenced from separate places (only pointer is cloned)
        let global_env = new_global_environment(global.clone(), global.clone());

        let new_realm = Self {
            global_obj: global.clone(),
            global_env,
            environment: LexicalEnvironment::new(global),
        };

        // Add new builtIns to Realm
        // At a later date this can be removed from here and called explicity, but for now we almost always want these default builtins
        new_realm.create_instrinsics();

        new_realm
    }

    // Sets up the default global objects within Global
    fn create_instrinsics(&self) {
        let global = &self.global_obj;
        // Create intrinsics, add global objects here
        function::init(global);

        global.set_field_slice("Array", array::create_constructor(global));
        global.set_field_slice("Boolean", boolean::create_constructor(global));
        global.set_field_slice("JSON", json::create_constructor(global));
        global.set_field_slice("Math", math::create_constructor(global));
        global.set_field_slice("Number", number::create_constructor(global));
        global.set_field_slice("Object", object::create_constructor(global));
        global.set_field_slice("RegExp", regexp::create_constructor(global));
        global.set_field_slice("String", string::create_constructor(global));
        global.set_field_slice("Symbol", symbol::create_constructor(global));
        global.set_field_slice("console", console::create_constructor(global));
    }

    /// Utility to add a function to the global object
    pub fn register_global_func(self, func_name: &str, func: NativeFunctionData) -> Self {
        self.global_obj
            .set_field(func_name.to_value(), func.to_value());

        self
    }
}

// Similar to new_global_environment in lexical_environment, except we need to return a GlobalEnvirionment
fn new_global_environment(
    global: Value,
    this_value: Value,
) -> Gc<GcCell<Box<GlobalEnvironmentRecord>>> {
    let obj_rec = Box::new(ObjectEnvironmentRecord {
        bindings: global,
        outer_env: None,
        /// Object Environment Records created for with statements (13.11)
        /// can provide their binding object as an implicit this value for use in function calls.
        /// The capability is controlled by a withEnvironment Boolean value that is associated
        /// with each object Environment Record. By default, the value of withEnvironment is false
        /// for any object Environment Record.
        with_environment: false,
    });

    let dcl_rec = Box::new(DeclarativeEnvironmentRecord {
        env_rec: HashMap::new(),
        outer_env: None,
    });

    Gc::new(GcCell::new(Box::new(GlobalEnvironmentRecord {
        object_record: obj_rec,
        global_this_binding: this_value,
        declarative_record: dcl_rec,
        var_names: HashSet::new(),
    })))
}
