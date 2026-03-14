//! Boa's implementation of ECMAScript's global `ShadowRealm` object.
//!
//! The `ShadowRealm` object is a distinct global environment that can execute
//! JavaScript code in a new, isolated realm.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/proposal-shadowrealm/

#[cfg(test)]
mod tests;

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{JsData, JsObject, internal_methods::get_prototype_from_constructor},
    realm::Realm,
    string::StaticJsStrings,
    Context, JsArgs, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};

/// The `ShadowRealm` built-in object.
#[derive(Debug, Trace, Finalize)]
pub struct ShadowRealm {
    inner: Realm,
}

impl JsData for ShadowRealm {}

impl IntrinsicObject for ShadowRealm {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::evaluate, js_string!("evaluate"), 1)
            .method(Self::import_value, js_string!("importValue"), 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ShadowRealm {
    const NAME: JsString = StaticJsStrings::SHADOW_REALM;
}

impl BuiltInConstructor for ShadowRealm {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 2;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::shadow_realm;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("ShadowRealm constructor: NewTarget is undefined")
                .into());
        }

        // 2. Let realmRec be ? CreateRealm().
        let realm = context.create_realm()?;

        // 3. Let shadowRealm be ? OrdinaryCreateFromConstructor(newTarget, "%ShadowRealm.prototype%", « [[ShadowRealm]] »).
        // 4. Set shadowRealm.[[ShadowRealm]] to realmRec.
        let prototype = get_prototype_from_constructor(new_target, StandardConstructors::shadow_realm, context)?;
        let shadow_realm = JsObject::from_proto_and_data(prototype, ShadowRealm { inner: realm });

        // 5. Return shadowRealm.
        Ok(shadow_realm.into())
    }
}

impl ShadowRealm {
    /// `ShadowRealm.prototype.evaluate ( sourceText )`
    pub(crate) fn evaluate(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let shadowRealm be the this value.
        // 2. Perform ? ValidateShadowRealm(shadowRealm).
        let shadow_realm_obj = this
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("ShadowRealm.prototype.evaluate: this is not a ShadowRealm object")
            })?;

        let shadow_realm = shadow_realm_obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("ShadowRealm.prototype.evaluate: this is not a ShadowRealm object")
        })?;

        // 3. If Type(sourceText) is not String, throw a TypeError exception.
        let source_text = args.get_or_undefined(0);
        if !source_text.is_string() {
            return Err(JsNativeError::typ()
                .with_message("ShadowRealm.prototype.evaluate: sourceText is not a string")
                .into());
        }

        // 4. Let realmRec be shadowRealm.[[ShadowRealm]].
        let realm = shadow_realm.inner.clone();

        // 5. Return ? PerformShadowRealmEval(sourceText, realmRec).

        // Switch realm
        let old_realm = context.enter_realm(realm);

        // Perform eval (indirect)
        let result =
            crate::builtins::eval::Eval::perform_eval(source_text, false, None, false, context);

        // Restore realm
        context.enter_realm(old_realm);

        let result = result?;

        // 6. Return ? GetWrappedValue(realm, result).
        // TODO: Implement GetWrappedValue (Callable Masking)
        // For now, just return the result if it's not a function.
        if result.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("ShadowRealm: Callable masking (function wrapping) not yet implemented")
                .into());
        }

        Ok(result)
    }

    /// `ShadowRealm.prototype.importValue ( specifier, name )`
    pub(crate) fn import_value(
        _this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // TODO: Implementation of importValue
        Err(JsNativeError::typ()
            .with_message("ShadowRealm.prototype.importValue: not yet implemented")
            .into())
    }
}
