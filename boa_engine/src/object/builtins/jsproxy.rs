//! This module implements a wrapper for the `Proxy` Builtin JavaScript Object
use boa_gc::{Finalize, Trace};

use crate::{
    builtins::{function::NativeFunctionSignature, Proxy},
    object::{FunctionBuilder, JsObject, JsObjectType, ObjectData},
    Context, JsResult, JsValue,
};

use super::JsFunction;

/// JavaScript [`Proxy`][proxy] rust object.
///
/// This is a wrapper type for the [`Proxy`][proxy] API that allows customizing
/// essential behaviour for an object, like [property accesses][get] or the
/// [`delete`][delete] operator.
///
/// The only way to construct this type is to use the [`JsProxyBuilder`] type; also
/// accessible from [`JsProxy::builder`].
///
/// [get]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/get
/// [delete]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/deleteProperty
/// [proxy]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsProxy {
    inner: JsObject,
}

impl JsProxy {
    pub fn builder(target: JsObject) -> JsProxyBuilder {
        JsProxyBuilder::new(target)
    }
}

impl From<JsProxy> for JsObject {
    #[inline]
    fn from(o: JsProxy) -> Self {
        o.inner.clone()
    }
}

impl From<JsProxy> for JsValue {
    #[inline]
    fn from(o: JsProxy) -> Self {
        o.inner.clone().into()
    }
}

impl std::ops::Deref for JsProxy {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsProxy {}

/// JavaScript [`Proxy`][proxy] rust object that can be disabled.
///
/// Safe interface for the [`Proxy.revocable`][revocable] method that creates a
/// proxy that can be disabled using the [`JsRevocableProxy::revoke`] method.
/// The internal proxy is accessible using the [`Deref`][`std::ops::Deref`] operator.
///
/// The only way to construct this type is to use the [`JsProxyBuilder`] type; also
/// accessible from [`JsProxy::builder`]; with the [`JsProxyBuilder::build_revocable`]
/// method.
///
/// [proxy]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy
/// [revocable]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy/revocable
#[derive(Debug, Trace, Finalize)]
pub struct JsRevocableProxy {
    proxy: JsProxy,
    revoker: JsFunction,
}

impl JsRevocableProxy {
    /// Disables the traps of the internal `proxy` object, essentially
    /// making it unusable and throwing `TypeError`s for all the traps.
    pub fn revoke(self, context: &mut Context) -> JsResult<()> {
        self.revoker.call(&JsValue::undefined(), &[], context)?;
        Ok(())
    }
}

impl std::ops::Deref for JsRevocableProxy {
    type Target = JsProxy;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.proxy
    }
}

/// Utility builder to create [`JsProxy`] objects from native functions.
///
/// This builder can be used when you need to create [`Proxy`] objects
/// from Rust instead of Javascript, which should generate faster
/// trap functions than its Javascript counterparts.
#[must_use]
#[derive(Clone)]
pub struct JsProxyBuilder {
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

impl std::fmt::Debug for JsProxyBuilder {
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

impl JsProxyBuilder {
    /// Create a new `ProxyBuilder` with every trap set to `undefined`.
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
    pub fn build(self, context: &mut Context) -> JsProxy {
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

        JsProxy { inner: proxy }
    }

    /// Builds a [`JsObject`] of kind [`Proxy`] and a [`JsFunction`] that, when
    /// called, disables the proxy of the object.
    ///
    /// Equivalent to the `Proxy.revocable ( target, handler )` static method,
    /// but returns a [`JsObject`] for the proxy and a [`JsFunction`] for the
    /// revoker in case there's a need to manipulate the returned objects
    /// inside Rust code.
    #[must_use]
    pub fn build_revocable(self, context: &mut Context) -> JsRevocableProxy {
        let proxy = self.build(context);
        let revoker = Proxy::revoker(proxy.inner.clone(), context);

        JsRevocableProxy { proxy, revoker }
    }
}
