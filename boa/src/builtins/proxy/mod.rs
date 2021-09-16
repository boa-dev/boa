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
    gc::{Finalize, Trace},
    object::{ConstructorBuilder, FunctionBuilder, JsObject, ObjectData},
    property::Attribute,
    BoaProfiler, Context, JsResult, JsValue,
};

/// Javascript `Proxy` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct Proxy {
    pub(crate) target: Option<JsObject>,
    pub(crate) handler: Option<JsObject>,
}

impl BuiltIn for Proxy {
    const NAME: &'static str = "Proxy";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);

    fn init(context: &mut Context) -> JsValue {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().proxy_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .constructor_has_prototype(false)
        .static_method(Self::revocable, "revocable", 2)
        .build()
        .into()
    }
}

impl Proxy {
    const LENGTH: usize = 2;

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
        Self::proxy_create(&JsValue::undefined(), args, context)
    }

    // `10.5.14 ProxyCreate ( target, handler )`
    //
    // More information:
    //  - [ECMAScript reference][spec]
    //
    // [spec]: https://tc39.es/ecma262/#sec-proxycreate
    fn proxy_create(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. If Type(target) is not Object, throw a TypeError exception.
        let target = if let Some(obj) = args.get_or_undefined(0).as_object() {
            obj
        } else {
            return context.throw_type_error("Proxy constructor called with undefined target");
        };

        // 2. If Type(handler) is not Object, throw a TypeError exception.
        let handler = if let Some(obj) = args.get_or_undefined(1).as_object() {
            obj
        } else {
            return context.throw_type_error("Proxy constructor called with undefined handler");
        };

        // 3. Let P be ! MakeBasicObject(« [[ProxyHandler]], [[ProxyTarget]] »).
        // 4. Set P's essential internal methods, except for [[Call]] and [[Construct]], to the definitions specified in 10.5.
        // 5. If IsCallable(target) is true, then
        // a. Set P.[[Call]] as specified in 10.5.12.
        // b. If IsConstructor(target) is true, then
        // i. Set P.[[Construct]] as specified in 10.5.13.
        // 6. Set P.[[ProxyTarget]] to target.
        // 7. Set P.[[ProxyHandler]] to handler.
        let p = context.construct_object();
        p.borrow_mut().data = ObjectData::proxy(
            Self {
                target: target.clone().into(),
                handler: handler.clone().into(),
            },
            target.is_callable(),
            target.is_constructor(),
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
        let p = Self::proxy_create(&JsValue::undefined(), args, context)?;

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
                assert!(p.borrow().is_proxy());

                // f. Set p.[[ProxyTarget]] to null.
                // g. Set p.[[ProxyHandler]] to null.
                p.borrow_mut().data = ObjectData::proxy(
                    Self {
                        target: None,
                        handler: None,
                    },
                    p.is_callable(),
                    p.is_constructor(),
                );

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
