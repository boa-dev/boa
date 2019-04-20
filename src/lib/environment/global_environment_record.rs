//! # Global Environment Records
//!
//! A global Environment Record is used to represent the outer most scope that is shared by all
//! of the ECMAScript Script elements that are processed in a common realm.
//! A global Environment Record provides the bindings for built-in globals (clause 18),
//! properties of the global object, and for all top-level declarations (13.2.8, 13.2.10)
//! that occur within a Script.
//! More info:  https://tc39.github.io/ecma262/#sec-global-environment-records

use crate::environment::declerative_environment_record::{
    DeclerativeEnvironmentRecord, DeclerativeEnvironmentRecordBinding,
};
use crate::environment::environment_record::EnvironmentRecordTrait;
use crate::environment::object_environment_record::ObjectEnvironmentRecord;
use crate::js::value::Value;
use std::collections::HashMap;

pub struct GlobalEnvironmentRecord {
    pub object_record: ObjectEnvironmentRecord,
    pub global_this_binding: Value,
    pub declerative_record: DeclerativeEnvironmentRecord,
    pub var_names: Vec<String>,
}

impl GlobalEnvironmentRecord {
    fn get_this_binding(&self) -> Value {
        return self.global_this_binding.clone();
    }

    fn has_var_decleration(&self, name: &String) -> bool {
        return self.var_names.contains(name);
    }
}

impl EnvironmentRecordTrait for GlobalEnvironmentRecord {
    fn has_binding(&self, name: &String) -> bool {
        let dclRec = self.declerative_record;
        if dclRec.has_binding(name) {
            return true;
        }
        self.object_record.has_binding(name)
    }
}
