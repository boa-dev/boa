//! This module implements the global `Proxy` object.
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
    builtins::{BuiltIn, JsArgs},
    object::{ConstructorBuilder, FunctionBuilder, JsObject, ObjectData},
    Context, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use tap::{Conv, Pipe};

/// Javascript `Proxy` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct Proxy {
    // (target, handler)
    data: Option<(JsObject, JsObject)>,
}

impl BuiltIn for Proxy {
    const NAME: &'static str = "Proxy";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().proxy().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .has_prototype_property(false)
        .static_method(Self::revocable, "revocable", 2)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl Proxy {
    const LENGTH: usize = 2;

    fn new(target: JsObject, handler: JsObject) -> Self {
        Self {
            data: Some((target, handler)),
        }
    }

    /// This is an internal method only built for usage in the proxy internal methods.
    ///
    /// It returns the (target, handler) of the proxy.
    pub(crate) fn try_data(&self, context: &mut Context) -> JsResult<(JsObject, JsObject)> {
        self.data.clone().ok_or_else(|| {
            context.construct_type_error("Proxy object has empty handler and target")
        })
    }

    /// `28.2.1.1 Proxy ( target, handler )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-proxy-target-handler
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return context.throw_type_error("Proxy constructor called on undefined new target");
        }

        // 2. Return ? ProxyCreate(target, handler).
        Self::create(args.get_or_undefined(0), args.get_or_undefined(1), context)
    }

    // `10.5.14 ProxyCreate ( target, handler )`
    //
    // More information:
    //  - [ECMAScript reference][spec]
    //
    // [spec]: https://tc39.es/ecma262/#sec-proxycreate
    fn create(target: &JsValue, handler: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        // 1. If Type(target) is not Object, throw a TypeError exception.
        let target = target.as_object().ok_or_else(|| {
            context.construct_type_error("Proxy constructor called with non-object target")
        })?;

        // 2. If Type(handler) is not Object, throw a TypeError exception.
        let handler = handler.as_object().ok_or_else(|| {
            context.construct_type_error("Proxy constructor called with non-object handler")
        })?;

        // 3. Let P be ! MakeBasicObject(« [[ProxyHandler]], [[ProxyTarget]] »).
        // 4. Set P's essential internal methods, except for [[Call]] and [[Construct]], to the definitions specified in 10.5.
        // 5. If IsCallable(target) is true, then
        // a. Set P.[[Call]] as specified in 10.5.12.
        // b. If IsConstructor(target) is true, then
        // i. Set P.[[Construct]] as specified in 10.5.13.
        // 6. Set P.[[ProxyTarget]] to target.
        // 7. Set P.[[ProxyHandler]] to handler.
        let p = JsObject::from_proto_and_data(
            context.intrinsics().constructors().object().prototype(),
            ObjectData::proxy(
                Self::new(target.clone(), handler.clone()),
                target.is_callable(),
                target.is_constructor(),
            ),
        );

        // 8. Return P.
        Ok(p.into())
    }

    /// `28.2.2.1 Proxy.revocable ( target, handler )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-proxy.revocable
    fn revocable(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let p be ? ProxyCreate(target, handler).
        let p = Self::create(args.get_or_undefined(0), args.get_or_undefined(1), context)?;

        // 3. Let revoker be ! CreateBuiltinFunction(revokerClosure, 0, "", « [[RevocableProxy]] »).
        // 4. Set revoker.[[RevocableProxy]] to p.
        let revoker = FunctionBuilder::closure_with_captures(
            context,
            |_, _, revocable_proxy, _| {
                // a. Let F be the active function object.
                // b. Let p be F.[[RevocableProxy]].
                // c. If p is null, return undefined.
                if revocable_proxy.is_null() {
                    return Ok(JsValue::undefined());
                }

                let p = revocable_proxy
                    .as_object()
                    .expect("[[RevocableProxy]] must be an object or null");

                // e. Assert: p is a Proxy object.
                // f. Set p.[[ProxyTarget]] to null.
                // g. Set p.[[ProxyHandler]] to null.
                p.borrow_mut()
                    .as_proxy_mut()
                    .expect("[[RevocableProxy]] must be a proxy object")
                    .data = None;

                // d. Set F.[[RevocableProxy]] to null.
                *revocable_proxy = JsValue::Null;

                // h. Return undefined.
                Ok(JsValue::undefined())
            },
            p.clone(),
        )
        .build();

        // 5. Let result be ! OrdinaryObjectCreate(%Object.prototype%).
        let result = context.construct_object();

        // 6. Perform ! CreateDataPropertyOrThrow(result, "proxy", p).
        result
            .create_data_property_or_throw("proxy", p, context)
            .expect("CreateDataPropertyOrThrow cannot fail here");

        // 7. Perform ! CreateDataPropertyOrThrow(result, "revoke", revoker).
        result
            .create_data_property_or_throw("revoke", revoker, context)
            .expect("CreateDataPropertyOrThrow cannot fail here");

        // 8. Return result.
        Ok(result.into())
    }
}
