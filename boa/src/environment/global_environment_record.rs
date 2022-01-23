//! # Global Environment Records
//!
//! A global Environment Record is used to represent the outer most scope that is shared by all
//! of the ECMAScript Script elements that are processed in a common realm.
//! A global Environment Record provides the bindings for built-in globals (clause 18),
//! properties of the global object, and for all top-level declarations (13.2.8, 13.2.10)
//! that occur within a Script.
//! More info:  <https://tc39.es/ecma262/#sec-global-environment-records>

use crate::{
    environment::{
        declarative_environment_record::DeclarativeEnvironmentRecord,
        environment_record_trait::EnvironmentRecordTrait,
        lexical_environment::{Environment, EnvironmentType, VariableScope},
        object_environment_record::ObjectEnvironmentRecord,
    },
    gc::{self, Finalize, Gc, Trace},
    object::JsObject,
    property::PropertyDescriptor,
    Context, JsResult, JsValue,
};
use boa_interner::Sym;
use rustc_hash::FxHashSet;

#[derive(Debug, Trace, Finalize, Clone)]
pub struct GlobalEnvironmentRecord {
    pub object_record: ObjectEnvironmentRecord,
    pub global_this_binding: JsObject,
    pub declarative_record: DeclarativeEnvironmentRecord,
    pub var_names: gc::Cell<FxHashSet<Sym>>,
}

impl GlobalEnvironmentRecord {
    pub fn new(global: JsObject, this_value: JsObject) -> GlobalEnvironmentRecord {
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

        let dcl_rec = DeclarativeEnvironmentRecord::new(None);

        GlobalEnvironmentRecord {
            object_record: obj_rec,
            global_this_binding: this_value,
            declarative_record: dcl_rec,
            var_names: gc::Cell::new(FxHashSet::default()),
        }
    }

    /// `9.1.1.4.12 HasVarDeclaration ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasvardeclaration
    pub fn has_var_declaration(&self, name: Sym) -> bool {
        // 1. Let varDeclaredNames be envRec.[[VarNames]].
        // 2. If varDeclaredNames contains N, return true.
        // 3. Return false.
        self.var_names.borrow().contains(&name)
    }

    /// `9.1.1.4.13 HasLexicalDeclaration ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-haslexicaldeclaration
    pub fn has_lexical_declaration(&self, name: Sym, context: &mut Context) -> JsResult<bool> {
        // 1. Let DclRec be envRec.[[DeclarativeRecord]].
        // 2. Return DclRec.HasBinding(N).
        self.declarative_record.has_binding(name, context)
    }

    /// `9.1.1.4.14 HasRestrictedGlobalProperty ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-hasrestrictedglobalproperty
    pub fn has_restricted_global_property(
        &self,
        name: Sym,
        context: &mut Context,
    ) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = &self.object_record.bindings;

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let existing_prop = global_object.__get_own_property__(
            &context
                .interner()
                .resolve(name)
                .expect("string disappeared")
                .into(),
            context,
        )?;

        if let Some(existing_prop) = existing_prop {
            // 5. If existingProp.[[Configurable]] is true, return false.
            // 6. Return true.
            Ok(!existing_prop.expect_configurable())
        } else {
            // 4. If existingProp is undefined, return false.
            Ok(false)
        }
    }

    /// `9.1.1.4.15 CanDeclareGlobalVar ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalvar
    pub fn can_declare_global_var(&self, name: Sym, context: &mut Context) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = &self.object_record.bindings;

        // 3. Let hasProperty be ? HasOwnProperty(globalObject, N).
        let key = context
            .interner()
            .resolve(name)
            .expect("string disappeared")
            .to_owned();
        let has_property = global_object.has_own_property(key, context)?;

        // 4. If hasProperty is true, return true.
        if has_property {
            return Ok(true);
        }

        // 5. Return ? IsExtensible(globalObject).
        global_object.is_extensible(context)
    }

    /// `9.1.1.4.16 CanDeclareGlobalFunction ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalfunction
    pub fn can_declare_global_function(&self, name: Sym, context: &mut Context) -> JsResult<bool> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = &self.object_record.bindings;

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let existing_prop = global_object.__get_own_property__(
            &context
                .interner()
                .resolve(name)
                .expect("string disappeared")
                .into(),
            context,
        )?;

        if let Some(existing_prop) = existing_prop {
            // 5. If existingProp.[[Configurable]] is true, return true.
            // 6. If IsDataDescriptor(existingProp) is true and existingProp has attribute values { [[Writable]]: true, [[Enumerable]]: true }, return true.
            if existing_prop.expect_configurable()
                || matches!(
                    (
                        existing_prop.is_data_descriptor(),
                        existing_prop.writable(),
                        existing_prop.enumerable(),
                    ),
                    (true, Some(true), Some(true))
                )
            {
                Ok(true)
            } else {
                // 7. Return false.
                Ok(false)
            }
        } else {
            // 4. If existingProp is undefined, return ? IsExtensible(globalObject).
            global_object.is_extensible(context)
        }
    }

    /// `9.1.1.4.17 CreateGlobalVarBinding ( N, D )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createglobalvarbinding
    pub fn create_global_var_binding(
        &mut self,
        name: Sym,
        deletion: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = &self.object_record.bindings;

        // 3. Let hasProperty be ? HasOwnProperty(globalObject, N).
        let has_property = global_object.has_own_property(
            context
                .interner()
                .resolve(name)
                .expect("string disappeared")
                .to_owned(),
            context,
        )?;
        // 4. Let extensible be ? IsExtensible(globalObject).
        let extensible = global_object.is_extensible(context)?;

        // 5. If hasProperty is false and extensible is true, then
        if !has_property && extensible {
            // a. Perform ? ObjRec.CreateMutableBinding(N, D).
            self.object_record
                .create_mutable_binding(name, deletion, false, context)?;
            // b. Perform ? ObjRec.InitializeBinding(N, undefined).
            self.object_record
                .initialize_binding(name, JsValue::undefined(), context)?;
        }

        // 6. Let varDeclaredNames be envRec.[[VarNames]].
        let mut var_declared_names = self.var_names.borrow_mut();
        // 7. If varDeclaredNames does not contain N, then
        if !var_declared_names.contains(&name) {
            // a. Append N to varDeclaredNames.
            var_declared_names.insert(name);
        }

        // 8. Return NormalCompletion(empty).
        Ok(())
    }

    /// `9.1.1.4.18 CreateGlobalFunctionBinding ( N, V, D )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createglobalfunctionbinding
    pub fn create_global_function_binding(
        &mut self,
        name: Sym,
        value: JsValue,
        deletion: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let ObjRec be envRec.[[ObjectRecord]].
        // 2. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = &self.object_record.bindings;

        // 3. Let existingProp be ? globalObject.[[GetOwnProperty]](N).
        let existing_prop = global_object.__get_own_property__(
            &context
                .interner()
                .resolve(name)
                .expect("string disappeared")
                .into(),
            context,
        )?;

        // 4. If existingProp is undefined or existingProp.[[Configurable]] is true, then
        let desc = if existing_prop
            .map(|f| f.expect_configurable())
            .unwrap_or(true)
        {
            // a. Let desc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: D }.
            PropertyDescriptor::builder()
                .value(value.clone())
                .writable(true)
                .enumerable(true)
                .configurable(deletion)
                .build()
        // 5. Else,
        } else {
            // a. Let desc be the PropertyDescriptor { [[Value]]: V }.
            PropertyDescriptor::builder().value(value.clone()).build()
        };

        let name_str = context
            .interner()
            .resolve(name)
            .expect("string disappeared")
            .to_owned();

        // 6. Perform ? DefinePropertyOrThrow(globalObject, N, desc).
        global_object.define_property_or_throw(name_str.as_str(), desc, context)?;
        // 7. Perform ? Set(globalObject, N, V, false).
        global_object.set(name_str, value, false, context)?;

        // 8. Let varDeclaredNames be envRec.[[VarNames]].
        // 9. If varDeclaredNames does not contain N, then
        if !self.var_names.borrow().contains(&name) {
            // a. Append N to varDeclaredNames.
            self.var_names.borrow_mut().insert(name);
        }

        // 10. Return NormalCompletion(empty).
        Ok(())
    }
}

impl EnvironmentRecordTrait for GlobalEnvironmentRecord {
    /// `9.1.1.4.1 HasBinding ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-hasbinding-n
    fn has_binding(&self, name: Sym, context: &mut Context) -> JsResult<bool> {
        // 1. Let DclRec be envRec.[[DeclarativeRecord]].
        // 2. If DclRec.HasBinding(N) is true, return true.
        if self.declarative_record.has_binding(name, context)? {
            return Ok(true);
        }

        // 3. Let ObjRec be envRec.[[ObjectRecord]].
        // 4. Return ? ObjRec.HasBinding(N).
        self.object_record.has_binding(name, context)
    }

    /// `9.1.1.4.2 CreateMutableBinding ( N, D )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-createmutablebinding-n-d
    fn create_mutable_binding(
        &self,
        name: Sym,
        deletion: bool,
        allow_name_reuse: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let DclRec be envRec.[[DeclarativeRecord]].
        // 2. If DclRec.HasBinding(N) is true, throw a TypeError exception.
        if !allow_name_reuse && self.declarative_record.has_binding(name, context)? {
            return context.throw_type_error(format!(
                "Binding already exists for {}",
                context
                    .interner()
                    .resolve(name)
                    .expect("string disappeared")
            ));
        }

        // 3. Return DclRec.CreateMutableBinding(N, D).
        self.declarative_record
            .create_mutable_binding(name, deletion, allow_name_reuse, context)
    }

    /// `9.1.1.4.3 CreateImmutableBinding ( N, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-createimmutablebinding-n-s
    fn create_immutable_binding(
        &self,
        name: Sym,
        strict: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let DclRec be envRec.[[DeclarativeRecord]].
        // 2. If DclRec.HasBinding(N) is true, throw a TypeError exception.
        if self.declarative_record.has_binding(name, context)? {
            return context.throw_type_error(format!(
                "Binding already exists for {}",
                context
                    .interner()
                    .resolve(name)
                    .expect("string disappeared")
            ));
        }

        // 3. Return DclRec.CreateImmutableBinding(N, S).
        self.declarative_record
            .create_immutable_binding(name, strict, context)
    }

    /// `9.1.1.4.4 InitializeBinding ( N, V )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-initializebinding-n-v
    fn initialize_binding(&self, name: Sym, value: JsValue, context: &mut Context) -> JsResult<()> {
        // 1. Let DclRec be envRec.[[DeclarativeRecord]].
        // 2. If DclRec.HasBinding(N) is true, then
        if self.declarative_record.has_binding(name, context)? {
            // a. Return DclRec.InitializeBinding(N, V).
            return self
                .declarative_record
                .initialize_binding(name, value, context);
        }

        // 3. Assert: If the binding exists, it must be in the object Environment Record.
        assert!(
            self.object_record.has_binding(name, context)?,
            "Binding must be in object_record"
        );

        // 4. Let ObjRec be envRec.[[ObjectRecord]].
        // 5. Return ? ObjRec.InitializeBinding(N, V).
        self.object_record.initialize_binding(name, value, context)
    }

    /// `9.1.1.4.5 SetMutableBinding ( N, V, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-setmutablebinding-n-v-s
    fn set_mutable_binding(
        &self,
        name: Sym,
        value: JsValue,
        strict: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let DclRec be envRec.[[DeclarativeRecord]].
        // 2. If DclRec.HasBinding(N) is true, then
        if self.declarative_record.has_binding(name, context)? {
            // a. Return DclRec.SetMutableBinding(N, V, S).
            return self
                .declarative_record
                .set_mutable_binding(name, value, strict, context);
        }

        // 3. Let ObjRec be envRec.[[ObjectRecord]].
        // 4. Return ? ObjRec.SetMutableBinding(N, V, S).
        self.object_record
            .set_mutable_binding(name, value, strict, context)
    }

    /// `9.1.1.4.6 GetBindingValue ( N, S )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-getbindingvalue-n-s
    fn get_binding_value(
        &self,
        name: Sym,
        strict: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let DclRec be envRec.[[DeclarativeRecord]].
        // 2. If DclRec.HasBinding(N) is true, then
        if self.declarative_record.has_binding(name, context)? {
            // a. Return DclRec.GetBindingValue(N, S).
            return self
                .declarative_record
                .get_binding_value(name, strict, context);
        }

        // 3. Let ObjRec be envRec.[[ObjectRecord]].
        // 4. Return ? ObjRec.GetBindingValue(N, S).
        self.object_record.get_binding_value(name, strict, context)
    }

    /// `9.1.1.4.7 DeleteBinding ( N )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-deletebinding-n
    fn delete_binding(&self, name: Sym, context: &mut Context) -> JsResult<bool> {
        // 1. Let DclRec be envRec.[[DeclarativeRecord]].
        // 2. If DclRec.HasBinding(N) is true, then
        if self.declarative_record.has_binding(name, context)? {
            // a. Return DclRec.DeleteBinding(N).
            return self.declarative_record.delete_binding(name, context);
        }

        // 3. Let ObjRec be envRec.[[ObjectRecord]].
        // 4. Let globalObject be ObjRec.[[BindingObject]].
        let global_object = &self.object_record.bindings;

        // 5. Let existingProp be ? HasOwnProperty(globalObject, N).
        // 6. If existingProp is true, then
        if global_object.has_own_property(
            context
                .interner()
                .resolve(name)
                .expect("string disappeared")
                .to_owned(),
            context,
        )? {
            // a. Let status be ? ObjRec.DeleteBinding(N).
            let status = self.object_record.delete_binding(name, context)?;

            // b. If status is true, then
            if status {
                // i. Let varNames be envRec.[[VarNames]].
                // ii. If N is an element of varNames, remove that element from the varNames.
                self.var_names.borrow_mut().remove(&name);
            }

            // c. Return status.
            return Ok(status);
        }

        // 7. Return true.
        Ok(true)
    }

    /// `9.1.1.4.8 HasThisBinding ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-hasthisbinding
    fn has_this_binding(&self) -> bool {
        // 1. Return true.
        true
    }

    /// `9.1.1.4.9 HasSuperBinding ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-hassuperbinding
    fn has_super_binding(&self) -> bool {
        // 1. Return false.
        false
    }

    /// `9.1.1.4.10 WithBaseObject ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-withbaseobject
    fn with_base_object(&self) -> Option<JsObject> {
        // 1. Return undefined.
        None
    }

    /// `9.1.1.4.11 GetThisBinding ( )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-global-environment-records-getthisbinding
    fn get_this_binding(&self, _context: &mut Context) -> JsResult<JsValue> {
        // 1. Return envRec.[[GlobalThisValue]].
        Ok(self.global_this_binding.clone().into())
    }

    fn get_outer_environment(&self) -> Option<Environment> {
        None
    }

    fn get_outer_environment_ref(&self) -> Option<&Environment> {
        None
    }

    fn set_outer_environment(&mut self, _env: Environment) {
        // TODO: Implement
        todo!("Not implemented yet")
    }

    fn get_environment_type(&self) -> EnvironmentType {
        EnvironmentType::Global
    }

    fn recursive_create_mutable_binding(
        &self,
        name: Sym,
        deletion: bool,
        _scope: VariableScope,
        context: &mut Context,
    ) -> JsResult<()> {
        self.create_mutable_binding(name, deletion, false, context)
    }

    fn recursive_create_immutable_binding(
        &self,
        name: Sym,
        deletion: bool,
        _scope: VariableScope,
        context: &mut Context,
    ) -> JsResult<()> {
        self.create_immutable_binding(name, deletion, context)
    }

    fn recursive_set_mutable_binding(
        &self,
        name: Sym,
        value: JsValue,
        strict: bool,
        context: &mut Context,
    ) -> JsResult<()> {
        self.set_mutable_binding(name, value, strict, context)
    }

    fn recursive_initialize_binding(
        &self,
        name: Sym,
        value: JsValue,
        context: &mut Context,
    ) -> JsResult<()> {
        self.initialize_binding(name, value, context)
    }
}

impl From<GlobalEnvironmentRecord> for Environment {
    fn from(env: GlobalEnvironmentRecord) -> Environment {
        Gc::new(Box::new(env))
    }
}
