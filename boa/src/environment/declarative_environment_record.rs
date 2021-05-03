//! # Declarative Records
//!
//! Each declarative Environment Record is associated with an ECMAScript program scope containing variable,
//! `constant`, `let`, `class`, `module`, `import`, and/or function declarations.
//! A declarative Environment Record binds the set of identifiers defined by the declarations contained within its scope.
//! More info:  [ECMA-262 sec-declarative-environment-records](https://tc39.es/ecma262/#sec-declarative-environment-records)

use crate::{
    environment::{
        environment_record_trait::EnvironmentRecordTrait,
        lexical_environment::{Environment, EnvironmentType},
    },
    gc::{Finalize, Trace},
    object::GcObject,
    Context, Result, Value,
};
use rustc_hash::FxHashMap;

/// Declarative Bindings have a few properties for book keeping purposes, such as mutability (const vs let).
/// Can it be deleted? and strict mode.
///
/// So we need to create a struct to hold these values.
/// From this point onwards, a binding is referring to one of these structures.
#[derive(Trace, Finalize, Debug, Clone)]
pub struct DeclarativeEnvironmentRecordBinding {
    pub value: Option<Value>,
    pub can_delete: bool,
    pub mutable: bool,
    pub strict: bool,
}

/// A declarative Environment Record binds the set of identifiers defined by the
/// declarations contained within its scope.
#[derive(Debug, Trace, Finalize, Clone)]
pub struct DeclarativeEnvironmentRecord {
    pub env_rec: FxHashMap<String, DeclarativeEnvironmentRecordBinding>,
    pub outer_env: Option<Environment>,
}

impl EnvironmentRecordTrait for DeclarativeEnvironmentRecord {
    fn has_binding(&self, name: &str) -> bool {
        self.env_rec.contains_key(name)
    }

    fn create_mutable_binding(
        &mut self,
        name: String,
        deletion: bool,
        allow_name_reuse: bool,
        _context: &mut Context,
    ) -> Result<()> {
        if !allow_name_reuse {
            assert!(
                !self.env_rec.contains_key(&name),
                "Identifier {} has already been declared",
                name
            );
        }

        self.env_rec.insert(
            name,
            DeclarativeEnvironmentRecordBinding {
                value: None,
                can_delete: deletion,
                mutable: true,
                strict: false,
            },
        );
        Ok(())
    }

    fn create_immutable_binding(
        &mut self,
        name: String,
        strict: bool,
        _context: &mut Context,
    ) -> Result<()> {
        assert!(
            !self.env_rec.contains_key(&name),
            "Identifier {} has already been declared",
            name
        );

        self.env_rec.insert(
            name,
            DeclarativeEnvironmentRecordBinding {
                value: None,
                can_delete: true,
                mutable: false,
                strict,
            },
        );
        Ok(())
    }

    fn initialize_binding(
        &mut self,
        name: &str,
        value: Value,
        _context: &mut Context,
    ) -> Result<()> {
        if let Some(ref mut record) = self.env_rec.get_mut(name) {
            if record.value.is_none() {
                record.value = Some(value);
                return Ok(());
            }
        }
        panic!("record must have binding for {}", name);
    }

    #[allow(clippy::else_if_without_else)]
    fn set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        mut strict: bool,
        context: &mut Context,
    ) -> Result<()> {
        if self.env_rec.get(name).is_none() {
            if strict {
                return Err(context.construct_reference_error(format!("{} not found", name)));
            }

            self.create_mutable_binding(name.to_owned(), true, false, context)?;
            self.initialize_binding(name, value, context)?;
            return Ok(());
        }

        let record: &mut DeclarativeEnvironmentRecordBinding = self.env_rec.get_mut(name).unwrap();
        if record.strict {
            strict = true
        }
        if record.value.is_none() {
            return Err(
                context.construct_reference_error(format!("{} has not been initialized", name))
            );
        }
        if record.mutable {
            record.value = Some(value);
        } else if strict {
            return Err(context.construct_reference_error(format!(
                "Cannot mutate an immutable binding {}",
                name
            )));
        }

        Ok(())
    }

    fn get_binding_value(&self, name: &str, _strict: bool, context: &mut Context) -> Result<Value> {
        if let Some(binding) = self.env_rec.get(name) {
            if let Some(ref val) = binding.value {
                Ok(val.clone())
            } else {
                context.throw_reference_error(format!("{} is an uninitialized binding", name))
            }
        } else {
            panic!("Cannot get binding value for {}", name);
        }
    }

    fn delete_binding(&mut self, name: &str) -> bool {
        match self.env_rec.get(name) {
            Some(binding) => {
                if binding.can_delete {
                    self.env_rec.remove(name);
                    true
                } else {
                    false
                }
            }
            None => panic!("env_rec has no binding for {}", name),
        }
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
        None
    }

    fn get_outer_environment_ref(&self) -> Option<&Environment> {
        self.outer_env.as_ref()
    }

    fn set_outer_environment(&mut self, env: Environment) {
        self.outer_env = Some(env);
    }

    fn get_environment_type(&self) -> EnvironmentType {
        EnvironmentType::Declarative
    }
}
