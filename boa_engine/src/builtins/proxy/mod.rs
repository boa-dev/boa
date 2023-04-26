//! Boa's implementation of ECMAScript's global `Proxy` object.
//!
//! The `Proxy` object enables you to create a proxy for another object,
//! which can intercept and redefine fundamental operations for that object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-proxy-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy

use crate::{
    builtins::BuiltInObject,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsFunction, JsObject, ObjectData},
    realm::Realm,
    string::utf16,
    Context, JsArgs, JsResult, JsValue,
};
use boa_gc::{Finalize, GcRefCell, Trace};
use boa_profiler::Profiler;

use super::{BuiltInBuilder, BuiltInConstructor, IntrinsicObject};
/// Javascript `Proxy` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct Proxy {
    // (target, handler)
    data: Option<(JsObject, JsObject)>,
}

impl IntrinsicObject for Proxy {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(Self::revocable, "revocable", 2)
            .build_without_prototype();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Proxy {
    const NAME: &'static str = "Proxy";
}

impl BuiltInConstructor for Proxy {
    const LENGTH: usize = 2;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::proxy;

    /// `28.2.1.1 Proxy ( target, handler )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-proxy-target-handler
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Proxy constructor called on undefined new target")
                .into());
        }

        // 2. Return ? ProxyCreate(target, handler).
        Self::create(args.get_or_undefined(0), args.get_or_undefined(1), context).map(JsValue::from)
    }
}

impl Proxy {
    pub(crate) fn new(target: JsObject, handler: JsObject) -> Self {
        Self {
            data: Some((target, handler)),
        }
    }

    /// This is an internal method only built for usage in the proxy internal methods.
    ///
    /// It returns the (target, handler) of the proxy.
    pub(crate) fn try_data(&self) -> JsResult<(JsObject, JsObject)> {
        self.data.clone().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Proxy object has empty handler and target")
                .into()
        })
    }

    // `10.5.14 ProxyCreate ( target, handler )`
    //
    // More information:
    //  - [ECMAScript reference][spec]
    //
    // [spec]: https://tc39.es/ecma262/#sec-proxycreate
    pub(crate) fn create(
        target: &JsValue,
        handler: &JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // 1. If Type(target) is not Object, throw a TypeError exception.
        let target = target.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Proxy constructor called with non-object target")
        })?;

        // 2. If Type(handler) is not Object, throw a TypeError exception.
        let handler = handler.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Proxy constructor called with non-object handler")
        })?;

        // 3. Let P be ! MakeBasicObject(« [[ProxyHandler]], [[ProxyTarget]] »).
        // 4. Set P's essential internal methods, except for [[Call]] and [[Construct]], to the definitions specified in 10.5.
        // 5. If IsCallable(target) is true, then
        // a. Set P.[[Call]] as specified in 10.5.12.
        // b. If IsConstructor(target) is true, then
        // i. Set P.[[Construct]] as specified in 10.5.13.
        // 6. Set P.[[ProxyTarget]] to target.
        // 7. Set P.[[ProxyHandler]] to handler.
        let p = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            ObjectData::proxy(
                Self::new(target.clone(), handler.clone()),
                target.is_callable(),
                target.is_constructor(),
            ),
        );

        // 8. Return P.
        Ok(p)
    }

    pub(crate) fn revoker(proxy: JsObject, context: &mut Context<'_>) -> JsFunction {
        // 3. Let revoker be ! CreateBuiltinFunction(revokerClosure, 0, "", « [[RevocableProxy]] »).
        // 4. Set revoker.[[RevocableProxy]] to p.
        FunctionObjectBuilder::new(
            context,
            NativeFunction::from_copy_closure_with_captures(
                |_, _, revocable_proxy, _| {
                    // a. Let F be the active function object.
                    // b. Let p be F.[[RevocableProxy]].
                    // d. Set F.[[RevocableProxy]] to null.
                    if let Some(p) = std::mem::take(&mut *revocable_proxy.borrow_mut()) {
                        // e. Assert: p is a Proxy object.
                        // f. Set p.[[ProxyTarget]] to null.
                        // g. Set p.[[ProxyHandler]] to null.
                        p.borrow_mut()
                            .as_proxy_mut()
                            .expect("[[RevocableProxy]] must be a proxy object")
                            .data = None;
                    }

                    // c. If p is null, return undefined.
                    // h. Return undefined.
                    Ok(JsValue::undefined())
                },
                GcRefCell::new(Some(proxy)),
            ),
        )
        .build()
    }

    /// `28.2.2.1 Proxy.revocable ( target, handler )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-proxy.revocable
    fn revocable(_: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let p be ? ProxyCreate(target, handler).
        let p = Self::create(args.get_or_undefined(0), args.get_or_undefined(1), context)?;

        // Revoker creation steps on `Proxy::revoker`
        let revoker = Self::revoker(p.clone(), context);

        // 5. Let result be ! OrdinaryObjectCreate(%Object.prototype%).
        let result = JsObject::with_object_proto(context.intrinsics());

        // 6. Perform ! CreateDataPropertyOrThrow(result, "proxy", p).
        result
            .create_data_property_or_throw(utf16!("proxy"), p, context)
            .expect("CreateDataPropertyOrThrow cannot fail here");

        // 7. Perform ! CreateDataPropertyOrThrow(result, "revoke", revoker).
        result
            .create_data_property_or_throw(utf16!("revoke"), revoker, context)
            .expect("CreateDataPropertyOrThrow cannot fail here");

        // 8. Return result.
        Ok(result.into())
    }
}
