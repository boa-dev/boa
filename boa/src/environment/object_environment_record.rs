//! # Object Records
//!
//! Each object Environment Record is associated with an object called its binding object.
//! An object Environment Record binds the set of string identifier names that directly
//! correspond to the property names of its binding object.
//! Property keys that are not strings in the form of an `IdentifierName` are not included in the set of bound identifiers.
//! More info:  [Object Records](https://tc39.es/ecma262/#sec-object-environment-records)

use gc::Gc;

use crate::{
    environment::{
        environment_record_trait::EnvironmentRecordTrait,
        lexical_environment::{Environment, EnvironmentType},
    },
    gc::{Finalize, Trace},
    object::GcObject,
    property::PropertyDescriptor,
    property::{Attribute, DataDescriptor},
    Context, Result, Value,
};

#[derive(Debug, Trace, Finalize, Clone)]
pub struct ObjectEnvironmentRecord {
    // TODO: bindings should be an object.
    pub bindings: Value,
    pub with_environment: bool,
    pub outer_env: Option<Environment>,
}

impl ObjectEnvironmentRecord {
    pub fn new(object: Value, environment: Option<Environment>) -> ObjectEnvironmentRecord {
        ObjectEnvironmentRecord {
            bindings: object,
            outer_env: environment,
            /// Object Environment Records created for with statements (13.11)
            /// can provide their binding object as an implicit this value for use in function calls.
            /// The capability is controlled by a withEnvironment Boolean value that is associated
            /// with each object Environment Record. By default, the value of withEnvironment is false
            /// for any object Environment Record.
            with_environment: false,
        }
    }
}

impl EnvironmentRecordTrait for ObjectEnvironmentRecord {
    fn has_binding(&self, name: &str) -> bool {
        if self.bindings.has_field(name) {
            if self.with_environment {
                // TODO: implement unscopables
            }
            true
        } else {
            false
        }
    }

    fn create_mutable_binding(
        &self,
        name: String,
        deletion: bool,
        _allow_name_reuse: bool,
        _context: &mut Context,
    ) -> Result<()> {
        // TODO: could save time here and not bother generating a new undefined object,
        // only for it to be replace with the real value later. We could just add the name to a Vector instead
        let bindings = &self.bindings;
        let mut prop = DataDescriptor::new(
            Value::undefined(),
            Attribute::WRITABLE | Attribute::ENUMERABLE,
        );
        prop.set_configurable(deletion);

        bindings.set_property(name, prop);
        Ok(())
    }

    fn create_immutable_binding(
        &self,
        _name: String,
        _strict: bool,
        _context: &mut Context,
    ) -> Result<()> {
        Ok(())
    }

    fn initialize_binding(&self, name: &str, value: Value, context: &mut Context) -> Result<()> {
        // We should never need to check if a binding has been created,
        // As all calls to create_mutable_binding are followed by initialized binding
        // The below is just a check.
        debug_assert!(self.has_binding(name));
        self.set_mutable_binding(name, value, false, context)
    }

    fn set_mutable_binding(
        &self,
        name: &str,
        value: Value,
        strict: bool,
        _context: &mut Context,
    ) -> Result<()> {
        debug_assert!(value.is_object() || value.is_function());
        let mut property = DataDescriptor::new(value, Attribute::ENUMERABLE);
        property.set_configurable(strict);
        self.bindings
            .as_object()
            .expect("binding object")
            .insert(name, property);
        Ok(())
    }

    fn get_binding_value(&self, name: &str, strict: bool, context: &mut Context) -> Result<Value> {
        if self.bindings.has_field(name) {
            match self.bindings.get_property(name) {
                Some(PropertyDescriptor::Data(ref d)) => Ok(d.value()),
                _ => Ok(Value::undefined()),
            }
        } else if strict {
            context.throw_reference_error(format!("{} has no binding", name))
        } else {
            Ok(Value::undefined())
        }
    }

    fn delete_binding(&self, name: &str) -> bool {
        self.bindings.remove_property(name);
        true
    }

    fn has_this_binding(&self) -> bool {
        false
    }

    fn get_this_binding(&self, _context: &mut Context) -> Result<Value> {
        Ok(Value::undefined())
    }

    fn has_super_binding(&self) -> bool {
        false
    }

    fn with_base_object(&self) -> Option<GcObject> {
        // Object Environment Records return undefined as their
        // WithBaseObject unless their withEnvironment flag is true.
        if self.with_environment {
            return Some(self.bindings.as_object().unwrap());
        }

        None
    }

    fn get_outer_environment_ref(&self) -> Option<&Environment> {
        self.outer_env.as_ref()
    }

    fn set_outer_environment(&mut self, env: Environment) {
        self.outer_env = Some(env);
    }

    fn get_environment_type(&self) -> EnvironmentType {
        EnvironmentType::Function
    }
}

impl From<ObjectEnvironmentRecord> for Environment {
    fn from(env: ObjectEnvironmentRecord) -> Environment {
        Gc::new(Box::new(env))
    }
}
