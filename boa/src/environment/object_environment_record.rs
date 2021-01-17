//! # Object Records
//!
//! Each object Environment Record is associated with an object called its binding object.
//! An object Environment Record binds the set of string identifier names that directly
//! correspond to the property names of its binding object.
//! Property keys that are not strings in the form of an `IdentifierName` are not included in the set of bound identifiers.
//! More info:  [Object Records](https://tc39.es/ecma262/#sec-object-environment-records)

use crate::property::PropertyDescriptor;
use crate::{
    environment::{
        environment_record_trait::EnvironmentRecordTrait,
        lexical_environment::{Environment, EnvironmentType},
    },
    gc::{Finalize, Trace},
    property::{Attribute, DataDescriptor},
    Context, Result, Value,
};

#[derive(Debug, Trace, Finalize, Clone)]
pub struct ObjectEnvironmentRecord {
    pub bindings: Value,
    pub with_environment: bool,
    pub outer_env: Option<Environment>,
}

impl EnvironmentRecordTrait for ObjectEnvironmentRecord {
    fn has_binding(&self, name: &str, context: &Context) -> Result<bool> {
        if self.bindings.has_field(name, context)? {
            if self.with_environment {
                // TODO: implement unscopables
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn create_mutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        _allow_name_reuse: bool,
        _context: &Context,
    ) -> Result<()> {
        // TODO: could save time here and not bother generating a new undefined object,
        // only for it to be replace with the real value later. We could just add the name to a Vector instead
        let bindings = &mut self.bindings;
        let mut prop = DataDescriptor::new(
            Value::undefined(),
            Attribute::WRITABLE | Attribute::ENUMERABLE,
        );
        prop.set_configurable(deletion);

        bindings.set_property(name, prop);
        Ok(())
    }

    fn create_immutable_binding(
        &mut self,
        _name: String,
        _strict: bool,
        _context: &Context,
    ) -> Result<()> {
        Ok(())
    }

    fn initialize_binding(&mut self, name: &str, value: Value, context: &Context) -> Result<()> {
        // We should never need to check if a binding has been created,
        // As all calls to create_mutable_binding are followed by initialized binding
        // The below is just a check.
        debug_assert!(self.has_binding(&name, context)?);
        self.set_mutable_binding(name, value, false, context)
    }

    fn set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        strict: bool,
        _context: &Context,
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

    fn get_binding_value(&self, name: &str, strict: bool, context: &Context) -> Result<Value> {
        if self.bindings.has_field(name, context)? {
            match self.bindings.get_property(name, context)? {
                Some(PropertyDescriptor::Data(ref d)) => Ok(d.value()),
                _ => Ok(Value::undefined()),
            }
        } else if strict {
            context.throw_reference_error(format!("{} has no binding", name))
        } else {
            Ok(Value::undefined())
        }
    }

    fn delete_binding(&mut self, name: &str, _context: &Context) -> Result<bool> {
        self.bindings.remove_property(name);
        Ok(true)
    }

    fn has_this_binding(&self) -> bool {
        false
    }

    fn get_this_binding(&self, _context: &Context) -> Result<Value> {
        Ok(Value::undefined())
    }

    fn has_super_binding(&self) -> bool {
        false
    }

    fn with_base_object(&self, _context: &Context) -> Result<Value> {
        // Object Environment Records return undefined as their
        // WithBaseObject unless their withEnvironment flag is true.
        if self.with_environment {
            return Ok(self.bindings.clone());
        }

        Ok(Value::undefined())
    }

    fn get_outer_environment(&self) -> Option<Environment> {
        match &self.outer_env {
            Some(outer) => Some(outer.clone()),
            None => None,
        }
    }

    fn set_outer_environment(&mut self, env: Environment) {
        self.outer_env = Some(env);
    }

    fn get_environment_type(&self) -> EnvironmentType {
        EnvironmentType::Function
    }

    fn get_global_object(&self) -> Option<Value> {
        if let Some(outer) = &self.outer_env {
            outer.borrow().get_global_object()
        } else {
            None
        }
    }
}
