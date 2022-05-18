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
    object::{ConstructorBuilder, FunctionBuilder, JsFunction, JsObject, ObjectData},
    Context, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use tap::{Conv, Pipe};

use super::function::NativeFunctionSignature;

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
        Self::create(args.get_or_undefined(0), args.get_or_undefined(1), context).map(JsValue::from)
    }

    // `10.5.14 ProxyCreate ( target, handler )`
    //
    // More information:
    //  - [ECMAScript reference][spec]
    //
    // [spec]: https://tc39.es/ecma262/#sec-proxycreate
    fn create(target: &JsValue, handler: &JsValue, context: &mut Context) -> JsResult<JsObject> {
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
        Ok(p)
    }

    fn revoker(proxy: JsObject, context: &mut Context) -> JsFunction {
        // 3. Let revoker be ! CreateBuiltinFunction(revokerClosure, 0, "", « [[RevocableProxy]] »).
        // 4. Set revoker.[[RevocableProxy]] to p.
        FunctionBuilder::closure_with_captures(
            context,
            |_, _, revocable_proxy, _| {
                // a. Let F be the active function object.
                // b. Let p be F.[[RevocableProxy]].
                // d. Set F.[[RevocableProxy]] to null.
                if let Some(p) = revocable_proxy.take() {
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
            Some(proxy),
        )
        .build()
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

        // Revoker creation steps on `Proxy::revoker`
        let revoker = Self::revoker(p.clone(), context);

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

/// Utility builder to create [`Proxy`] objects from native functions.
///
/// This builder can be used when you need to create [`Proxy`] objects
/// from Rust instead of Javascript, which should generate faster
/// trap functions than its JS counterpart.
#[must_use]
#[derive(Clone)]
pub struct ProxyBuilder {
    target: JsObject,
    apply: Option<NativeFunctionSignature>,
    construct: Option<NativeFunctionSignature>,
    define_property: Option<NativeFunctionSignature>,
    delete_property: Option<NativeFunctionSignature>,
    get: Option<NativeFunctionSignature>,
    get_own_property_descriptor: Option<NativeFunctionSignature>,
    get_prototype_of: Option<NativeFunctionSignature>,
    has: Option<NativeFunctionSignature>,
    is_extensible: Option<NativeFunctionSignature>,
    own_keys: Option<NativeFunctionSignature>,
    prevent_extensions: Option<NativeFunctionSignature>,
    set: Option<NativeFunctionSignature>,
    set_prototype_of: Option<NativeFunctionSignature>,
}

impl std::fmt::Debug for ProxyBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        struct NativeFunction;
        f.debug_struct("ProxyBuilder")
            .field("target", &self.target)
            .field("apply", &self.apply.map(|_| NativeFunction))
            .field("construct", &self.construct.map(|_| NativeFunction))
            .field(
                "define_property",
                &self.define_property.map(|_| NativeFunction),
            )
            .field(
                "delete_property",
                &self.delete_property.map(|_| NativeFunction),
            )
            .field("get", &self.get.map(|_| NativeFunction))
            .field(
                "get_own_property_descriptor",
                &self.get_own_property_descriptor.map(|_| NativeFunction),
            )
            .field(
                "get_prototype_of",
                &self.get_prototype_of.map(|_| NativeFunction),
            )
            .field("has", &self.has.map(|_| NativeFunction))
            .field("is_extensible", &self.is_extensible.map(|_| NativeFunction))
            .field("own_keys", &self.own_keys.map(|_| NativeFunction))
            .field(
                "prevent_extensions",
                &self.prevent_extensions.map(|_| NativeFunction),
            )
            .field("set", &self.set.map(|_| NativeFunction))
            .field(
                "set_prototype_of",
                &self.set_prototype_of.map(|_| NativeFunction),
            )
            .finish()
    }
}

impl ProxyBuilder {
    /// Create a new `ProxyBuilder` structure with every trap set to `undefined`.
    pub fn new(target: JsObject) -> Self {
        Self {
            target,
            apply: None,
            construct: None,
            define_property: None,
            delete_property: None,
            get: None,
            get_own_property_descriptor: None,
            get_prototype_of: None,
            has: None,
            is_extensible: None,
            own_keys: None,
            prevent_extensions: None,
            set: None,
            set_prototype_of: None,
        }
    }

    /// Set the `apply` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// # Note
    ///
    /// If the `target` object is not a function, then `apply` will be ignored
    /// when trying to call the proxy, which will throw a type error.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/apply
    pub fn apply(mut self, apply: NativeFunctionSignature) -> Self {
        self.apply = Some(apply);
        self
    }

    /// Set the `construct` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// # Note
    ///
    /// If the `target` object is not a constructor, then `construct` will be ignored
    /// when trying to construct an object using the proxy, which will throw a type error.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/construct
    pub fn construct(mut self, construct: NativeFunctionSignature) -> Self {
        self.construct = Some(construct);
        self
    }

    /// Set the `defineProperty` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/defineProperty
    pub fn define_property(mut self, define_property: NativeFunctionSignature) -> Self {
        self.define_property = Some(define_property);
        self
    }

    /// Set the `deleteProperty` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/deleteProperty
    pub fn delete_property(mut self, delete_property: NativeFunctionSignature) -> Self {
        self.delete_property = Some(delete_property);
        self
    }

    /// Set the `get` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/get
    pub fn get(mut self, get: NativeFunctionSignature) -> Self {
        self.get = Some(get);
        self
    }

    /// Set the `getOwnPropertyDescriptor` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/getOwnPropertyDescriptor
    pub fn get_own_property_descriptor(
        mut self,
        get_own_property_descriptor: NativeFunctionSignature,
    ) -> Self {
        self.get_own_property_descriptor = Some(get_own_property_descriptor);
        self
    }

    /// Set the `getPrototypeOf` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/getPrototypeOf
    pub fn get_prototype_of(mut self, get_prototype_of: NativeFunctionSignature) -> Self {
        self.get_prototype_of = Some(get_prototype_of);
        self
    }

    /// Set the `has` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/has
    pub fn has(mut self, has: NativeFunctionSignature) -> Self {
        self.has = Some(has);
        self
    }

    /// Set the `isExtensible` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/isExtensible
    pub fn is_extensible(mut self, is_extensible: NativeFunctionSignature) -> Self {
        self.is_extensible = Some(is_extensible);
        self
    }

    /// Set the `ownKeys` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/ownKeys
    pub fn own_keys(mut self, own_keys: NativeFunctionSignature) -> Self {
        self.own_keys = Some(own_keys);
        self
    }

    /// Set the `preventExtensions` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/preventExtensions
    pub fn prevent_extensions(mut self, prevent_extensions: NativeFunctionSignature) -> Self {
        self.prevent_extensions = Some(prevent_extensions);
        self
    }

    /// Set the `set` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/set
    pub fn set(mut self, set: NativeFunctionSignature) -> Self {
        self.set = Some(set);
        self
    }

    /// Set the `setPrototypeOf` proxy trap to the specified native function.
    ///
    /// More information:
    ///
    /// - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/setPrototypeOf
    pub fn set_prototype_of(mut self, set_prototype_of: NativeFunctionSignature) -> Self {
        self.set_prototype_of = Some(set_prototype_of);
        self
    }

    /// Build a [`JsObject`] of kind [`Proxy`].
    ///
    /// Equivalent to the `Proxy ( target, handler )` constructor, but returns a
    /// [`JsObject`] in case there's a need to manipulate the returned object
    /// inside Rust code.
    #[must_use]
    pub fn build(self, context: &mut Context) -> JsObject {
        let handler = context.construct_object();

        if let Some(apply) = self.apply {
            let f = FunctionBuilder::native(context, apply).length(3).build();
            handler
                .create_data_property_or_throw("apply", f, context)
                .expect("new object should be writable");
        }
        if let Some(construct) = self.construct {
            let f = FunctionBuilder::native(context, construct)
                .length(3)
                .build();
            handler
                .create_data_property_or_throw("construct", f, context)
                .expect("new object should be writable");
        }
        if let Some(define_property) = self.define_property {
            let f = FunctionBuilder::native(context, define_property)
                .length(3)
                .build();
            handler
                .create_data_property_or_throw("defineProperty", f, context)
                .expect("new object should be writable");
        }
        if let Some(delete_property) = self.delete_property {
            let f = FunctionBuilder::native(context, delete_property)
                .length(2)
                .build();
            handler
                .create_data_property_or_throw("deleteProperty", f, context)
                .expect("new object should be writable");
        }
        if let Some(get) = self.get {
            let f = FunctionBuilder::native(context, get).length(3).build();
            handler
                .create_data_property_or_throw("get", f, context)
                .expect("new object should be writable");
        }
        if let Some(get_own_property_descriptor) = self.get_own_property_descriptor {
            let f = FunctionBuilder::native(context, get_own_property_descriptor)
                .length(2)
                .build();
            handler
                .create_data_property_or_throw("getOwnPropertyDescriptor", f, context)
                .expect("new object should be writable");
        }
        if let Some(get_prototype_of) = self.get_prototype_of {
            let f = FunctionBuilder::native(context, get_prototype_of)
                .length(1)
                .build();
            handler
                .create_data_property_or_throw("getPrototypeOf", f, context)
                .expect("new object should be writable");
        }
        if let Some(has) = self.has {
            let f = FunctionBuilder::native(context, has).length(2).build();
            handler
                .create_data_property_or_throw("has", f, context)
                .expect("new object should be writable");
        }
        if let Some(is_extensible) = self.is_extensible {
            let f = FunctionBuilder::native(context, is_extensible)
                .length(1)
                .build();
            handler
                .create_data_property_or_throw("isExtensible", f, context)
                .expect("new object should be writable");
        }
        if let Some(own_keys) = self.own_keys {
            let f = FunctionBuilder::native(context, own_keys).length(1).build();
            handler
                .create_data_property_or_throw("ownKeys", f, context)
                .expect("new object should be writable");
        }
        if let Some(prevent_extensions) = self.prevent_extensions {
            let f = FunctionBuilder::native(context, prevent_extensions)
                .length(1)
                .build();
            handler
                .create_data_property_or_throw("preventExtensions", f, context)
                .expect("new object should be writable");
        }
        if let Some(set) = self.set {
            let f = FunctionBuilder::native(context, set).length(4).build();
            handler
                .create_data_property_or_throw("set", f, context)
                .expect("new object should be writable");
        }
        if let Some(set_prototype_of) = self.set_prototype_of {
            let f = FunctionBuilder::native(context, set_prototype_of)
                .length(2)
                .build();
            handler
                .create_data_property_or_throw("setPrototypeOf", f, context)
                .expect("new object should be writable");
        }

        let callable = self.target.is_callable();
        let constructor = self.target.is_constructor();

        let proxy = JsObject::from_proto_and_data(
            context.intrinsics().constructors().object().prototype(),
            ObjectData::proxy(Proxy::new(self.target, handler), callable, constructor),
        );

        proxy
    }

    /// Builds a [`JsObject`] of kind [`Proxy`] and a [`JsFunction`] that, when
    /// called, disables the proxy of the object.
    ///
    /// Equivalent to the `Proxy.revocable ( target, handler )` static method,
    /// but returns a [`JsObject`] for the proxy and a [`JsFunction`] for the
    /// revoker in case there's a need to manipulate the returned objects
    /// inside Rust code.
    #[must_use]
    pub fn build_revocable(self, context: &mut Context) -> (JsObject, JsFunction) {
        let proxy = self.build(context);
        let revoker = Proxy::revoker(proxy.clone(), context);

        (proxy, revoker)
    }
}
