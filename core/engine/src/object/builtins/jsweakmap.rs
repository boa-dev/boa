//! A Rust API wrapper for the `WeakMap` Builtin ECMAScript Object
use std::ops::Deref;

use boa_gc::{Finalize, Trace};

use crate::{
    Context, JsResult, JsValue,
    builtins::weak_map::WeakMap,
    error::JsNativeError,
    object::{ErasedVTableObject, JsFunction, JsObject},
    value::TryFromJs,
};

type NativeWeakMap = boa_gc::WeakMap<ErasedVTableObject, JsValue>;

/// `JsWeakMap` provides a wrapper for Boa's implementation of the ECMAScript `WeakMap` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsWeakMap {
    inner: JsObject,
}

impl JsWeakMap {
    /// Create a new empty `WeakMap`.
    ///
    /// Same as JavaScript's `new WeakMap()`.
    #[inline]
    pub fn new(context: &mut Context) -> Self {
        Self {
            inner: JsObject::from_proto_and_data_with_shared_shape(
                context.root_shape(),
                context.intrinsics().constructors().weak_map().prototype(),
                NativeWeakMap::new(),
            )
            .upcast(),
        }
    }

    /// Removes the element associated with the key.
    /// Returns `true` if the element existed, `false` otherwise.
    ///
    /// Same as JavaScript's `weakmap.delete(key)`.
    #[inline]
    pub fn delete(&self, key: &JsObject, context: &mut Context) -> JsResult<bool> {
        WeakMap::delete(&self.inner.clone().into(), &[key.clone().into()], context)
            .map(|v| v.as_boolean().unwrap_or(false))
    }

    /// Returns the value associated with the key, or `undefined` if not present.
    ///
    /// Same as JavaScript's `weakmap.get(key)`.
    #[inline]
    pub fn get(&self, key: &JsObject, context: &mut Context) -> JsResult<JsValue> {
        WeakMap::get(&self.inner.clone().into(), &[key.clone().into()], context)
    }

    /// Returns `true` if the key exists in the `WeakMap`.
    ///
    /// Same as JavaScript's `weakmap.has(key)`.
    #[inline]
    pub fn has(&self, key: &JsObject, context: &mut Context) -> JsResult<bool> {
        WeakMap::has(&self.inner.clone().into(), &[key.clone().into()], context)
            .map(|v| v.as_boolean().unwrap_or(false))
    }

    /// Sets the value for the given key.
    /// Returns the `JsWeakMap` itself.
    ///
    /// Same as JavaScript's `weakmap.set(key, value)`.
    #[inline]
    pub fn set(&self, key: &JsObject, value: JsValue, context: &mut Context) -> JsResult<Self> {
        WeakMap::set(
            &self.inner.clone().into(),
            &[key.clone().into(), value],
            context,
        )?;
        Ok(self.clone())
    }

    /// Returns the existing value if the key exists; otherwise inserts `default` and returns it.
    ///
    /// Same as JavaScript's `weakmap.getOrInsert(key, value)`.
    #[inline]
    pub fn get_or_insert(
        &self,
        key: &JsObject,
        default: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        WeakMap::get_or_insert(
            &self.inner.clone().into(),
            &[key.clone().into(), default],
            context,
        )
    }

    /// Returns the existing value if the key exists; otherwise calls `callback(key)`,
    /// inserts the result, and returns it.
    ///
    /// Same as JavaScript's `weakmap.getOrInsertComputed(key, callback)`.
    #[inline]
    pub fn get_or_insert_computed(
        &self,
        key: &JsObject,
        callback: JsFunction,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        WeakMap::get_or_insert_computed(
            &self.inner.clone().into(),
            &[key.clone().into(), callback.into()],
            context,
        )
    }

    /// Creates a `JsWeakMap` from a `JsObject`, or returns the original object as `Err`.
    #[inline]
    pub fn from_object(object: JsObject) -> Result<Self, JsObject> {
        if object.downcast_ref::<NativeWeakMap>().is_some() {
            Ok(Self { inner: object })
        } else {
            Err(object)
        }
    }
}

impl From<JsWeakMap> for JsObject {
    #[inline]
    fn from(o: JsWeakMap) -> Self {
        o.inner.clone()
    }
}

impl From<JsWeakMap> for JsValue {
    #[inline]
    fn from(o: JsWeakMap) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsWeakMap {
    type Target = JsObject;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsWeakMap {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        if let Some(o) = value.as_object()
            && let Ok(weak_map) = Self::from_object(o.clone())
        {
            Ok(weak_map)
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not a WeakMap object")
                .into())
        }
    }
}
