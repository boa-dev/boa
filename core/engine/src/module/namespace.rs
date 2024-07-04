use std::{collections::HashSet, hash::BuildHasherDefault};

use indexmap::IndexSet;
use rustc_hash::FxHasher;

use boa_gc::{Finalize, Trace};

use crate::object::internal_methods::immutable_prototype::immutable_prototype_exotic_set_prototype_of;
use crate::object::internal_methods::{
    ordinary_define_own_property, ordinary_delete, ordinary_get, ordinary_get_own_property,
    ordinary_has_property, ordinary_own_property_keys, ordinary_try_get, InternalMethodContext,
    InternalObjectMethods, ORDINARY_INTERNAL_METHODS,
};
use crate::object::{JsData, JsPrototype};
use crate::property::{PropertyDescriptor, PropertyKey};
use crate::{js_string, object::JsObject, Context, JsResult, JsString, JsValue};
use crate::{JsNativeError, Module};

use super::BindingName;

/// Module namespace exotic object.
///
/// Exposes the bindings exported by a [`Module`] to be accessed from ECMAScript code.
#[derive(Debug, Trace, Finalize)]
pub struct ModuleNamespace {
    module: Module,
    #[unsafe_ignore_trace]
    exports: IndexSet<JsString, BuildHasherDefault<FxHasher>>,
}

impl JsData for ModuleNamespace {
    fn internal_methods(&self) -> &'static InternalObjectMethods {
        static METHODS: InternalObjectMethods = InternalObjectMethods {
            __get_prototype_of__: module_namespace_exotic_get_prototype_of,
            __set_prototype_of__: module_namespace_exotic_set_prototype_of,
            __is_extensible__: module_namespace_exotic_is_extensible,
            __prevent_extensions__: module_namespace_exotic_prevent_extensions,
            __get_own_property__: module_namespace_exotic_get_own_property,
            __define_own_property__: module_namespace_exotic_define_own_property,
            __has_property__: module_namespace_exotic_has_property,
            __try_get__: module_namespace_exotic_try_get,
            __get__: module_namespace_exotic_get,
            __set__: module_namespace_exotic_set,
            __delete__: module_namespace_exotic_delete,
            __own_property_keys__: module_namespace_exotic_own_property_keys,
            ..ORDINARY_INTERNAL_METHODS
        };

        &METHODS
    }
}

impl ModuleNamespace {
    /// Abstract operation [`ModuleNamespaceCreate ( module, exports )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-modulenamespacecreate
    pub(crate) fn create(module: Module, names: Vec<JsString>, context: &mut Context) -> JsObject {
        // 1. Assert: module.[[Namespace]] is empty.
        // ignored since this is ensured by `Module::namespace`.

        // 6. Let sortedExports be a List whose elements are the elements of exports ordered as if an Array of the same values had been sorted using %Array.prototype.sort% using undefined as comparefn.
        let mut exports = names.into_iter().collect::<IndexSet<_, _>>();
        exports.sort();

        // 2. Let internalSlotsList be the internal slots listed in Table 32.
        // 3. Let M be MakeBasicObject(internalSlotsList).
        // 4. Set M's essential internal methods to the definitions specified in 10.4.6.
        // 5. Set M.[[Module]] to module.
        // 7. Set M.[[Exports]] to sortedExports.
        // 8. Create own properties of M corresponding to the definitions in 28.3.
        let namespace = context
            .intrinsics()
            .templates()
            .namespace()
            .create(Self { module, exports }, vec![js_string!("Module").into()]);

        // 9. Set module.[[Namespace]] to M.
        // Ignored because this is done by `Module::namespace`

        // 10. Return M.
        namespace
    }

    /// Gets the export names of the Module Namespace object.
    pub(crate) const fn exports(&self) -> &IndexSet<JsString, BuildHasherDefault<FxHasher>> {
        &self.exports
    }

    /// Gest the module associated with this Module Namespace object.
    pub(crate) const fn module(&self) -> &Module {
        &self.module
    }
}

/// [`[[GetPrototypeOf]] ( )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-getprototypeof
#[allow(clippy::unnecessary_wraps)]
fn module_namespace_exotic_get_prototype_of(
    _: &JsObject,
    _: &mut Context,
) -> JsResult<JsPrototype> {
    // 1. Return null.
    Ok(None)
}

/// [`[[SetPrototypeOf]] ( V )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-setprototypeof-v
#[allow(clippy::unnecessary_wraps)]
fn module_namespace_exotic_set_prototype_of(
    obj: &JsObject,
    val: JsPrototype,
    context: &mut Context,
) -> JsResult<bool> {
    // 1. Return ! SetImmutablePrototype(O, V).
    Ok(
        immutable_prototype_exotic_set_prototype_of(obj, val, context)
            .expect("this must not fail per the spec"),
    )
}

/// [`[[IsExtensible]] ( )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-isextensible
#[allow(clippy::unnecessary_wraps)]
fn module_namespace_exotic_is_extensible(_: &JsObject, _: &mut Context) -> JsResult<bool> {
    // 1. Return false.
    Ok(false)
}

/// [`[[PreventExtensions]] ( )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-preventextensions
#[allow(clippy::unnecessary_wraps)]
fn module_namespace_exotic_prevent_extensions(_: &JsObject, _: &mut Context) -> JsResult<bool> {
    Ok(true)
}

/// [`[[GetOwnProperty]] ( P )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-getownproperty-p
fn module_namespace_exotic_get_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<Option<PropertyDescriptor>> {
    // 1. If P is a Symbol, return OrdinaryGetOwnProperty(O, P).
    let key = match key {
        PropertyKey::Symbol(_) => return ordinary_get_own_property(obj, key, context),
        PropertyKey::Index(idx) => js_string!(format!("{}", idx.get())),
        PropertyKey::String(s) => s.clone(),
    };

    {
        let obj = obj
            .downcast_ref::<ModuleNamespace>()
            .expect("internal method can only be called on module namespace objects");
        // 2. Let exports be O.[[Exports]].
        let exports = obj.exports();

        // 3. If exports does not contain P, return undefined.
        if !exports.contains(&key) {
            return Ok(None);
        }
    }

    // 4. Let value be ? O.[[Get]](P, O).
    let value = obj.get(key, context)?;

    // 5. Return PropertyDescriptor { [[Value]]: value, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: false }.
    Ok(Some(
        PropertyDescriptor::builder()
            .value(value)
            .writable(true)
            .enumerable(true)
            .configurable(false)
            .build(),
    ))
}

/// [`[[DefineOwnProperty]] ( P, Desc )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-defineownproperty-p-desc
fn module_namespace_exotic_define_own_property(
    obj: &JsObject,
    key: &PropertyKey,
    desc: PropertyDescriptor,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    // 1. If P is a Symbol, return ! OrdinaryDefineOwnProperty(O, P, Desc).
    if let PropertyKey::Symbol(_) = key {
        return ordinary_define_own_property(obj, key, desc, context);
    }

    // 2. Let current be ? O.[[GetOwnProperty]](P).
    let Some(current) = obj.__get_own_property__(key, context)? else {
        // 3. If current is undefined, return false.
        return Ok(false);
    };

    // 4. If Desc has a [[Configurable]] field and Desc.[[Configurable]] is true, return false.
    // 5. If Desc has an [[Enumerable]] field and Desc.[[Enumerable]] is false, return false.
    // 6. If IsAccessorDescriptor(Desc) is true, return false.
    // 7. If Desc has a [[Writable]] field and Desc.[[Writable]] is false, return false.
    if desc.configurable() == Some(true)
        || desc.enumerable() == Some(false)
        || desc.is_accessor_descriptor()
        || desc.writable() == Some(false)
    {
        return Ok(false);
    }

    // 8. If Desc has a [[Value]] field, return SameValue(Desc.[[Value]], current.[[Value]]).
    // 9. Return true.
    Ok(desc.value().map_or(true, |v| v == current.expect_value()))
}

/// [`[[HasProperty]] ( P )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-hasproperty-p
fn module_namespace_exotic_has_property(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    // 1. If P is a Symbol, return ! OrdinaryHasProperty(O, P).
    let key = match key {
        PropertyKey::Symbol(_) => return ordinary_has_property(obj, key, context),
        PropertyKey::Index(idx) => js_string!(format!("{}", idx.get())),
        PropertyKey::String(s) => s.clone(),
    };

    let obj = obj
        .downcast_ref::<ModuleNamespace>()
        .expect("internal method can only be called on module namespace objects");

    // 2. Let exports be O.[[Exports]].
    let exports = obj.exports();

    // 3. If exports contains P, return true.
    // 4. Return false.
    Ok(exports.contains(&key))
}

/// Internal optimization method for `Module Namespace` exotic objects.
///
/// This method combines the internal methods `[[HasProperty]]` and `[[Get]]`.
///
/// More information:
///  - [ECMAScript reference HasProperty][spec0]
///  - [ECMAScript reference Get][spec1]
///
/// [spec0]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-hasproperty-p
/// [spec1]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-get-p-receiver
fn module_namespace_exotic_try_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<Option<JsValue>> {
    // 1. If P is a Symbol, then
    //     a. Return ! OrdinaryGet(O, P, Receiver).
    let key = match key {
        PropertyKey::Symbol(_) => return ordinary_try_get(obj, key, receiver, context),
        PropertyKey::Index(idx) => js_string!(format!("{}", idx.get())),
        PropertyKey::String(s) => s.clone(),
    };

    let obj = obj
        .downcast_ref::<ModuleNamespace>()
        .expect("internal method can only be called on module namespace objects");

    // 2. Let exports be O.[[Exports]].
    let exports = obj.exports();

    // 3. If exports does not contain P, return undefined.
    let Some(export_name) = exports.get(&key).cloned() else {
        return Ok(None);
    };

    // 4. Let m be O.[[Module]].
    let m = obj.module();

    // 5. Let binding be m.ResolveExport(P).
    let binding = m
        .resolve_export(
            export_name.clone(),
            &mut HashSet::default(),
            context.interner(),
        )
        .expect("6. Assert: binding is a ResolvedBinding Record.");

    // 7. Let targetModule be binding.[[Module]].
    // 8. Assert: targetModule is not undefined.
    let target_module = binding.module();

    // TODO: cache binding resolution instead of doing the whole process on every access.
    if let BindingName::Name(name) = binding.binding_name() {
        // 10. Let targetEnv be targetModule.[[Environment]].
        let Some(env) = target_module.environment() else {
            // 11. If targetEnv is empty, throw a ReferenceError exception.
            let import = export_name.to_std_string_escaped();
            return Err(JsNativeError::reference()
                .with_message(format!(
                    "cannot get import `{import}` from an uninitialized module"
                ))
                .into());
        };

        let locator = env
            .compile_env()
            .get_binding(&name)
            .expect("checked before that the name was reachable");

        // 12. Return ? targetEnv.GetBindingValue(binding.[[BindingName]], true).
        env.get(locator.binding_index()).map(Some).ok_or_else(|| {
            let import = export_name.to_std_string_escaped();

            JsNativeError::reference()
                .with_message(format!("cannot get uninitialized import `{import}`"))
                .into()
        })
    } else {
        // 9. If binding.[[BindingName]] is namespace, then
        //     a. Return GetModuleNamespace(targetModule).
        Ok(Some(target_module.namespace(context).into()))
    }
}

/// [`[[Get]] ( P, Receiver )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-get-p-receiver
fn module_namespace_exotic_get(
    obj: &JsObject,
    key: &PropertyKey,
    receiver: JsValue,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<JsValue> {
    // 1. If P is a Symbol, then
    //     a. Return ! OrdinaryGet(O, P, Receiver).
    let key = match key {
        PropertyKey::Symbol(_) => return ordinary_get(obj, key, receiver, context),
        PropertyKey::Index(idx) => js_string!(format!("{}", idx.get())),
        PropertyKey::String(s) => s.clone(),
    };

    let obj = obj
        .downcast_ref::<ModuleNamespace>()
        .expect("internal method can only be called on module namespace objects");

    // 2. Let exports be O.[[Exports]].
    let exports = obj.exports();
    // 3. If exports does not contain P, return undefined.
    let Some(export_name) = exports.get(&key).cloned() else {
        return Ok(JsValue::undefined());
    };

    // 4. Let m be O.[[Module]].
    let m = obj.module();

    // 5. Let binding be m.ResolveExport(P).
    let binding = m
        .resolve_export(
            export_name.clone(),
            &mut HashSet::default(),
            context.interner(),
        )
        .expect("6. Assert: binding is a ResolvedBinding Record.");

    // 7. Let targetModule be binding.[[Module]].
    // 8. Assert: targetModule is not undefined.
    let target_module = binding.module();

    // TODO: cache binding resolution instead of doing the whole process on every access.
    if let BindingName::Name(name) = binding.binding_name() {
        // 10. Let targetEnv be targetModule.[[Environment]].
        let Some(env) = target_module.environment() else {
            // 11. If targetEnv is empty, throw a ReferenceError exception.
            let import = export_name.to_std_string_escaped();
            return Err(JsNativeError::reference()
                .with_message(format!(
                    "cannot get import `{import}` from an uninitialized module"
                ))
                .into());
        };

        let locator = env
            .compile_env()
            .get_binding(&name)
            .expect("checked before that the name was reachable");

        // 12. Return ? targetEnv.GetBindingValue(binding.[[BindingName]], true).
        env.get(locator.binding_index()).ok_or_else(|| {
            let import = export_name.to_std_string_escaped();

            JsNativeError::reference()
                .with_message(format!("cannot get uninitialized import `{import}`"))
                .into()
        })
    } else {
        // 9. If binding.[[BindingName]] is namespace, then
        //     a. Return GetModuleNamespace(targetModule).
        Ok(target_module.namespace(context).into())
    }
}

/// [`[[Set]] ( P, V, Receiver )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-set-p-v-receiver
#[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
fn module_namespace_exotic_set(
    _obj: &JsObject,
    _key: PropertyKey,
    _value: JsValue,
    _receiver: JsValue,
    _context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    // 1. Return false.
    Ok(false)
}

/// [`[[Delete]] ( P )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-delete-p
fn module_namespace_exotic_delete(
    obj: &JsObject,
    key: &PropertyKey,
    context: &mut InternalMethodContext<'_>,
) -> JsResult<bool> {
    // 1. If P is a Symbol, then
    //     a. Return ! OrdinaryDelete(O, P).
    let key = match key {
        PropertyKey::Symbol(_) => return ordinary_delete(obj, key, context),
        PropertyKey::Index(idx) => js_string!(format!("{}", idx.get())),
        PropertyKey::String(s) => s.clone(),
    };

    let obj = obj
        .downcast_ref::<ModuleNamespace>()
        .expect("internal method can only be called on module namespace objects");

    // 2. Let exports be O.[[Exports]].
    let exports = obj.exports();

    // 3. If exports contains P, return false.
    // 4. Return true.
    Ok(!exports.contains(&key))
}

/// [`[[OwnPropertyKeys]] ( )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects-ownpropertykeys
fn module_namespace_exotic_own_property_keys(
    obj: &JsObject,
    context: &mut Context,
) -> JsResult<Vec<PropertyKey>> {
    // 2. Let symbolKeys be OrdinaryOwnPropertyKeys(O).
    let symbol_keys = ordinary_own_property_keys(obj, context)?;

    let obj = obj
        .downcast_ref::<ModuleNamespace>()
        .expect("internal method can only be called on module namespace objects");

    // 1. Let exports be O.[[Exports]].
    let exports = obj.exports();

    // 3. Return the list-concatenation of exports and symbolKeys.
    Ok(exports
        .iter()
        .map(|k| PropertyKey::String(k.clone()))
        .chain(symbol_keys)
        .collect())
}
