//! # Object Records
//!
//! Each object Environment Record is associated with an object called its binding object.
//! An object Environment Record binds the set of string identifier names that directly
//! correspond to the property names of its binding object.
//! Property keys that are not strings in the form of an `IdentifierName` are not included in the set of bound identifiers.
//! More info:  [Object Records](https://tc39.es/ecma262/#sec-object-environment-records)

use crate::{
    environment::{
        environment_record_trait::EnvironmentRecordTrait,
        lexical_environment::{Environment, EnvironmentType},
    },
    gc::{Finalize, Gc, Trace},
    object::JsObject,
    property::PropertyDescriptor,
    symbol::WellKnownSymbols,
    Context, JsResult, JsValue,
};
use boa_interner::Sym;

#[derive(Debug, Trace, Finalize, Clone)]
pub struct ObjectEnvironmentRecord {
    pub bindings: JsObject,
    pub with_environment: bool,
    pub outer_env: Option<Environment>,
}

impl ObjectEnvironmentRecord {
    pub fn new(object: JsObject, environment: Option<Environment>) -> ObjectEnvironmentRecord {
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
    /// `9.1.1.2.1 HasBinding ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-environment-records-hasbinding-n
    fn has_binding(&self, name: Sym, context: &mut Context) -> JsResult<bool> {
        // 1. Let bindingObject be envRec.[[BindingObject]].
        // 2. Let foundBinding be ? HasProperty(bindingObject, N).
        // 3. If foundBinding is false, return false.
        if !self
            .bindings
            .has_property(context.interner().resolve_expect(name).to_owned(), context)?
        {
            return Ok(false);
        }

        // 4. If envRec.[[IsWithEnvironment]] is false, return true.
        if !self.with_environment {
            return Ok(true);
        }

        // 5. Let unscopables be ? Get(bindingObject, @@unscopables).
        // 6. If Type(unscopables) is Object, then
        if let Some(unscopables) = self
            .bindings
            .get(WellKnownSymbols::unscopables(), context)?
            .as_object()
        {
            // a. Let blocked be ! ToBoolean(? Get(unscopables, N)).
            // b. If blocked is true, return false.
            if unscopables
                .get(context.interner().resolve_expect(name).to_owned(), context)?
                .to_boolean()
            {
                return Ok(false);
            }
        }

        // 7. Return true.
        Ok(true)
    }

    /// `9.1.1.2.2 CreateMutableBinding ( N, D )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-environment-records-createmutablebinding-n-d
    fn create_mutable_binding(
        &self,
        name: Sym,
        deletion: bool,
        _allow_name_reuse: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let bindingObject be envRec.[[BindingObject]].
        // 2. Return ? DefinePropertyOrThrow(bindingObject, N, PropertyDescriptor { [[Value]]: undefined, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: D }).
        self.bindings.define_property_or_throw(
            context.interner().resolve_expect(name).to_owned(),
            PropertyDescriptor::builder()
                .value(JsValue::undefined())
                .writable(true)
                .enumerable(true)
                .configurable(deletion),
            context,
        )?;
        Ok(())
    }

    /// `9.1.1.2.3 CreateImmutableBinding ( N, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-environment-records-createimmutablebinding-n-s
    fn create_immutable_binding(
        &self,
        _name: Sym,
        _strict: bool,
        _context: &mut Context,
    ) -> JsResult<()> {
        Ok(())
    }

    /// `9.1.1.2.4 InitializeBinding ( N, V )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-environment-records-initializebinding-n-v
    fn initialize_binding(&self, name: Sym, value: JsValue, context: &mut Context) -> JsResult<()> {
        // 1. Return ? envRec.SetMutableBinding(N, V, false).
        self.set_mutable_binding(name, value, false, context)
    }

    /// `9.1.1.2.5 SetMutableBinding ( N, V, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-environment-records-setmutablebinding-n-v-s
    fn set_mutable_binding(
        &self,
        name: Sym,
        value: JsValue,
        strict: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let bindingObject be envRec.[[BindingObject]].
        // 2. Let stillExists be ? HasProperty(bindingObject, N).
        let still_exists = self
            .bindings
            .has_property(context.interner().resolve_expect(name).to_owned(), context)?;

        // 3. If stillExists is false and S is true, throw a ReferenceError exception.
        if !still_exists && strict {
            return context.throw_reference_error("Binding already exists");
        }

        // 4. Return ? Set(bindingObject, N, V, S).
        self.bindings.set(
            context.interner().resolve_expect(name).to_owned(),
            value,
            strict,
            context,
        )?;
        Ok(())
    }

    /// `9.1.1.2.6 GetBindingValue ( N, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-environment-records-getbindingvalue-n-s
    fn get_binding_value(
        &self,
        name: Sym,
        strict: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let bindingObject be envRec.[[BindingObject]].
        // 2. Let value be ? HasProperty(bindingObject, N).
        // 3. If value is false, then
        if !self
            .bindings
            .__has_property__(&context.interner().resolve_expect(name).into(), context)?
        {
            // a. If S is false, return the value undefined; otherwise throw a ReferenceError exception.
            if !strict {
                return Ok(JsValue::undefined());
            } else {
                return context.throw_reference_error(format!(
                    "{} has no binding",
                    context.interner().resolve_expect(name)
                ));
            }
        }

        // 4. Return ? Get(bindingObject, N).
        self.bindings
            .get(context.interner().resolve_expect(name).to_owned(), context)
    }

    /// `9.1.1.2.7 DeleteBinding ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-environment-records-deletebinding-n
    fn delete_binding(&self, name: Sym, context: &mut Context) -> JsResult<bool> {
        // 1. Let bindingObject be envRec.[[BindingObject]].
        // 2. Return ? bindingObject.[[Delete]](N).
        self.bindings
            .__delete__(&context.interner().resolve_expect(name).into(), context)
    }

    /// `9.1.1.2.8 HasThisBinding ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-environment-records-hasthisbinding
    fn has_this_binding(&self) -> bool {
        // 1. Return false.
        false
    }

    /// `9.1.1.2.9 HasSuperBinding ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-environment-records-hassuperbinding
    fn has_super_binding(&self) -> bool {
        // 1. Return false.
        false
    }

    /// `9.1.1.2.10 WithBaseObject ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-object-environment-records-hassuperbinding
    fn with_base_object(&self) -> Option<JsObject> {
        // 1. If envRec.[[IsWithEnvironment]] is true, return envRec.[[BindingObject]].
        // 2. Otherwise, return undefined.
        if self.with_environment {
            Some(self.bindings.clone())
        } else {
            None
        }
    }

    fn get_this_binding(&self, _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
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
