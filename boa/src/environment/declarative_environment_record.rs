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
    gc::{self, Finalize, Gc, Trace},
    object::JsObject,
    BoaProfiler, Context, JsResult, JsValue,
};
use boa_interner::Sym;
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
    pub env_rec: gc::Cell<FxHashMap<Sym, DeclarativeEnvironmentRecordBinding>>,
    pub outer_env: Option<Environment>,
}

impl DeclarativeEnvironmentRecord {
    pub fn new(env: Option<Environment>) -> DeclarativeEnvironmentRecord {
        let _timer = BoaProfiler::global().start_event("new_declarative_environment", "env");
        DeclarativeEnvironmentRecord {
            env_rec: gc::Cell::new(FxHashMap::default()),
            outer_env: env,
        }
    }
}

impl EnvironmentRecordTrait for DeclarativeEnvironmentRecord {
    /// `9.1.1.1.1 HasBinding ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-declarative-environment-records-hasbinding-n
    fn has_binding(&self, name: Sym, _context: &mut Context) -> JsResult<bool> {
        // 1. If envRec has a binding for the name that is the value of N, return true.
        // 2. Return false.
        Ok(self.env_rec.borrow().contains_key(&name))
    }

    /// `9.1.1.1.2 CreateMutableBinding ( N, D )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-declarative-environment-records-createmutablebinding-n-d
    fn create_mutable_binding(
        &self,
        name: Sym,
        deletion: bool,
        allow_name_reuse: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Assert: envRec does not already have a binding for N.
        if !allow_name_reuse {
            assert!(
                !self.env_rec.borrow().contains_key(&name),
                "Identifier {} has already been declared",
                context
                    .interner()
                    .resolve(name)
                    .expect("string disappeared")
            );
        }

        // 2. Create a mutable binding in envRec for N and record that it is uninitialized.
        //    If D is true, record that the newly created binding may be deleted by a subsequent DeleteBinding call.
        self.env_rec.borrow_mut().insert(
            name,
            DeclarativeEnvironmentRecordBinding {
                value: None,
                can_delete: deletion,
                mutable: true,
                strict: false,
            },
        );

        // 3. Return NormalCompletion(empty).
        Ok(())
    }

    /// `9.1.1.1.3 CreateImmutableBinding ( N, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-declarative-environment-records-createimmutablebinding-n-s
    fn create_immutable_binding(
        &self,
        name: Sym,
        strict: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Assert: envRec does not already have a binding for N.
        assert!(
            !self.env_rec.borrow().contains_key(&name),
            "Identifier {} has already been declared",
            context
                .interner()
                .resolve(name)
                .expect("string disappeared")
        );

        // 2. Create an immutable binding in envRec for N and record that it is uninitialized.
        //    If S is true, record that the newly created binding is a strict binding.
        self.env_rec.borrow_mut().insert(
            name,
            DeclarativeEnvironmentRecordBinding {
                value: None,
                can_delete: true,
                mutable: false,
                strict,
            },
        );

        // 3. Return NormalCompletion(empty).
        Ok(())
    }

    /// `9.1.1.1.4 InitializeBinding ( N, V )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-declarative-environment-records-initializebinding-n-v
    fn initialize_binding(&self, name: Sym, value: JsValue, context: &mut Context) -> JsResult<()> {
        if let Some(ref mut record) = self.env_rec.borrow_mut().get_mut(&name) {
            if record.value.is_none() {
                // 2. Set the bound value for N in envRec to V.
                // 3. Record that the binding for N in envRec has been initialized.
                record.value = Some(value);

                // 4. Return NormalCompletion(empty).
                return Ok(());
            }
        }

        // 1. Assert: envRec must have an uninitialized binding for N.
        panic!(
            "record must have binding for {}",
            context
                .interner()
                .resolve(name)
                .expect("string disappeared")
        );
    }

    /// `9.1.1.1.5 SetMutableBinding ( N, V, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-declarative-environment-records-setmutablebinding-n-v-s
    #[allow(clippy::else_if_without_else)]
    fn set_mutable_binding(
        &self,
        name: Sym,
        value: JsValue,
        mut strict: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. If envRec does not have a binding for N, then
        if self.env_rec.borrow().get(&name).is_none() {
            // a. If S is true, throw a ReferenceError exception.
            if strict {
                return context.throw_reference_error(format!(
                    "{} not found",
                    context
                        .interner()
                        .resolve(name)
                        .expect("string disappeared")
                ));
            }

            // b. Perform envRec.CreateMutableBinding(N, true).
            self.create_mutable_binding(name, true, false, context)?;
            // c. Perform envRec.InitializeBinding(N, V).
            self.initialize_binding(name, value, context)?;

            // d. Return NormalCompletion(empty).
            return Ok(());
        }

        let (binding_strict, binding_value_is_none, binding_mutable) = {
            let env_rec = self.env_rec.borrow();
            let binding = env_rec.get(&name).unwrap();
            (binding.strict, binding.value.is_none(), binding.mutable)
        };

        // 2. If the binding for N in envRec is a strict binding, set S to true.
        if binding_strict {
            strict = true;
        }

        // 3. If the binding for N in envRec has not yet been initialized, throw a ReferenceError exception.
        if binding_value_is_none {
            return context.throw_reference_error(format!(
                "{} has not been initialized",
                context
                    .interner()
                    .resolve(name)
                    .expect("string disappeared")
            ));
        // 4. Else if the binding for N in envRec is a mutable binding, change its bound value to V.
        } else if binding_mutable {
            let mut env_rec = self.env_rec.borrow_mut();
            let binding = env_rec.get_mut(&name).unwrap();
            binding.value = Some(value);
        // 5. Else,
        // a. Assert: This is an attempt to change the value of an immutable binding.
        // b. If S is true, throw a TypeError exception.
        } else if strict {
            return context.throw_type_error(format!(
                "Cannot mutate an immutable binding {}",
                context
                    .interner()
                    .resolve(name)
                    .expect("string disappeared")
            ));
        }

        // 6. Return NormalCompletion(empty).
        Ok(())
    }

    /// `9.1.1.1.6 GetBindingValue ( N, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-declarative-environment-records-getbindingvalue-n-s
    fn get_binding_value(
        &self,
        name: Sym,
        _strict: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Assert: envRec has a binding for N.
        // 2. If the binding for N in envRec is an uninitialized binding, throw a ReferenceError exception.
        // 3. Return the value currently bound to N in envRec.
        if let Some(binding) = self.env_rec.borrow().get(&name) {
            if let Some(ref val) = binding.value {
                Ok(val.clone())
            } else {
                context.throw_reference_error(format!(
                    "{} is an uninitialized binding",
                    context
                        .interner()
                        .resolve(name)
                        .expect("string disappeared")
                ))
            }
        } else {
            panic!(
                "Cannot get binding value for {}",
                context
                    .interner()
                    .resolve(name)
                    .expect("string disappeared")
            );
        }
    }

    /// `9.1.1.1.7 DeleteBinding ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-declarative-environment-records-deletebinding-n
    fn delete_binding(&self, name: Sym, context: &mut Context) -> JsResult<bool> {
        // 1. Assert: envRec has a binding for the name that is the value of N.
        // 2. If the binding for N in envRec cannot be deleted, return false.
        // 3. Remove the binding for N from envRec.
        // 4. Return true.
        match self.env_rec.borrow().get(&name) {
            Some(binding) => {
                if binding.can_delete {
                    self.env_rec.borrow_mut().remove(&name);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => panic!(
                "env_rec has no binding for {}",
                context
                    .interner()
                    .resolve(name)
                    .expect("string disappeared")
            ),
        }
    }

    /// `9.1.1.1.8 HasThisBinding ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-declarative-environment-records-hasthisbinding
    fn has_this_binding(&self) -> bool {
        // 1. Return false.
        false
    }

    fn get_this_binding(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }

    /// `9.1.1.1.9 HasSuperBinding ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-declarative-environment-records-hassuperbinding
    fn has_super_binding(&self) -> bool {
        // 1. Return false.
        false
    }

    /// `9.1.1.1.10 WithBaseObject ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-declarative-environment-records-withbaseobject
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
