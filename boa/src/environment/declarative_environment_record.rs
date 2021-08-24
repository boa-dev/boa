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
    object::JsObject,
    BoaProfiler, Context, JsResult, JsValue,
};
use gc::{Gc, GcCell};
use rustc_hash::FxHashMap;

/// Declarative Bindings have a few properties for book keeping purposes, such as mutability (const vs let).
/// Can it be deleted? and strict mode.
///
/// So we need to create a struct to hold these values.
/// From this point onwards, a binding is referring to one of these structures.
#[derive(Trace, Finalize, Debug, Clone)]
pub struct DeclarativeEnvironmentRecordBinding {
    pub value: Option<JsValue>,
    pub can_delete: bool,
    pub mutable: bool,
    pub strict: bool,
}

/// A declarative Environment Record binds the set of identifiers defined by the
/// declarations contained within its scope.
#[derive(Debug, Trace, Finalize, Clone)]
pub struct DeclarativeEnvironmentRecord {
    pub env_rec: GcCell<FxHashMap<Box<str>, DeclarativeEnvironmentRecordBinding>>,
    pub outer_env: Option<Environment>,
}

impl DeclarativeEnvironmentRecord {
    pub fn new(env: Option<Environment>) -> DeclarativeEnvironmentRecord {
        let _timer = BoaProfiler::global().start_event("new_declarative_environment", "env");
        DeclarativeEnvironmentRecord {
            env_rec: GcCell::new(FxHashMap::default()),
            outer_env: env,
        }
    }
}

impl EnvironmentRecordTrait for DeclarativeEnvironmentRecord {
    fn has_binding(&self, name: &str) -> bool {
        self.env_rec.borrow().contains_key(name)
    }

    fn create_mutable_binding(
        &self,
        name: String,
        deletion: bool,
        allow_name_reuse: bool,
        _context: &mut Context,
    ) -> JsResult<()> {
        if !allow_name_reuse {
            assert!(
                !self.env_rec.borrow().contains_key(name.as_str()),
                "Identifier {} has already been declared",
                name
            );
        }

        self.env_rec.borrow_mut().insert(
            name.into_boxed_str(),
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
        &self,
        name: String,
        strict: bool,
        _context: &mut Context,
    ) -> JsResult<()> {
        assert!(
            !self.env_rec.borrow().contains_key(name.as_str()),
            "Identifier {} has already been declared",
            name
        );

        self.env_rec.borrow_mut().insert(
            name.into_boxed_str(),
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
        &self,
        name: &str,
        value: JsValue,
        _context: &mut Context,
    ) -> JsResult<()> {
        if let Some(ref mut record) = self.env_rec.borrow_mut().get_mut(name) {
            if record.value.is_none() {
                record.value = Some(value);
                return Ok(());
            }
        }
        panic!("record must have binding for {}", name);
    }

    #[allow(clippy::else_if_without_else)]
    fn set_mutable_binding(
        &self,
        name: &str,
        value: JsValue,
        mut strict: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        if self.env_rec.borrow().get(name).is_none() {
            if strict {
                return Err(context.construct_reference_error(format!("{} not found", name)));
            }

            self.create_mutable_binding(name.to_owned(), true, false, context)?;
            self.initialize_binding(name, value, context)?;
            return Ok(());
        }

        let (record_strict, record_has_no_value, record_mutable) = {
            let env_rec = self.env_rec.borrow();
            let record = env_rec.get(name).unwrap();
            (record.strict, record.value.is_none(), record.mutable)
        };
        if record_strict {
            strict = true
        }
        if record_has_no_value {
            return Err(
                context.construct_reference_error(format!("{} has not been initialized", name))
            );
        }
        if record_mutable {
            let mut env_rec = self.env_rec.borrow_mut();
            let record = env_rec.get_mut(name).unwrap();
            record.value = Some(value);
        } else if strict {
            return Err(context.construct_reference_error(format!(
                "Cannot mutate an immutable binding {}",
                name
            )));
        }

        Ok(())
    }

    fn get_binding_value(
        &self,
        name: &str,
        _strict: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let Some(binding) = self.env_rec.borrow().get(name) {
            if let Some(ref val) = binding.value {
                Ok(val.clone())
            } else {
                context.throw_reference_error(format!("{} is an uninitialized binding", name))
            }
        } else {
            panic!("Cannot get binding value for {}", name);
        }
    }

    fn delete_binding(&self, name: &str) -> bool {
        match self.env_rec.borrow().get(name) {
            Some(binding) => {
                if binding.can_delete {
                    self.env_rec.borrow_mut().remove(name);
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

    fn get_this_binding(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }

    fn has_super_binding(&self) -> bool {
        false
    }

    fn with_base_object(&self) -> Option<JsObject> {
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

impl From<DeclarativeEnvironmentRecord> for Environment {
    fn from(env: DeclarativeEnvironmentRecord) -> Environment {
        Gc::new(Box::new(env))
    }
}
