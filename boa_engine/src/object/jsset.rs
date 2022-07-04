use std::ops::Deref;

use boa_gc::{Finalize, Trace};

use crate::{
    builtins::Set,
    object::{JsObject, JsObjectType},
    Context, JsResult, JsValue,
};

// This is an wrapper for `JsSet`
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsSet {
    inner: JsObject,
}

impl JsSet {
    /// Create a new empty set.
    ///
    /// Doesn't matches JavaScript `new Set()` as it doesn't takes an iterator
    /// similar to Rust initialization.
    #[inline]
    pub fn new(context: &mut Context) -> Self {
        let inner = Set::set_create(None, context)
            .expect("creating an empty set with the default prototype must not fail");

        Self { inner }
    }

    /// Returns the size of the `Set` as an integer.
    ///
    /// Same as JavaScript's `set.size`.
    #[inline]
    pub fn size(&self, context: &mut Context) -> JsResult<usize> {
        Set::get_size(&self.inner.clone().into(), context)
    }

    /// Add an element to `Set` and returns `Set` with value appended to the end.
    ///
    /// Same as JavaScript's `set.add(value)`.
    #[inline]
    pub fn add<T>(&self, value: T, context: &mut Context) -> JsResult<Self>
    where
        T: Into<JsValue>,
    {
        let object = Set::add(&self.inner.clone().into(), &[value.into()], context)?
            .as_object()
            .cloned()
            .expect("Set.prototype.add should always return `Set` object.");

        Self::from_object(object, context)
    }

    /// Removes all the elements for the `Set` and returns `Undefined`.
    ///
    /// Same as JavaScript's `set.clear()`.
    pub fn clear(&self, context: &mut Context) -> JsResult<JsValue> {
        Set::clear(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// Removes given value from the `Set` and returns `Bool`.
    ///
    /// Same as JavaScript's `set.delete(value)`.
    pub fn delete<T>(&self, value: T, context: &mut Context) -> JsResult<bool>
    where
        T: Into<JsValue>,
    {
        match Set::delete(&self.inner.clone().into(), &[value.into()], context)? {
            JsValue::Boolean(bool) => Ok(bool),
            _ => Err(JsValue::Undefined),
        }
    }

    /// Checks if given value is in the `Set` and returns `Bool`.
    ///
    /// Same as JavaScript's `set.has(value)`.
    pub fn has<T>(&self, value: T, context: &mut Context) -> JsResult<bool>
    where
        T: Into<JsValue>,
    {
        match Set::has(&self.inner.clone().into(), &[value.into()], context)? {
            JsValue::Boolean(bool) => Ok(bool),
            _ => Err(JsValue::Undefined),
        }
    }

    /// Utility: Creates `JsSet` from `JsObject`, if not a Set throw `TypeError`.
    #[inline]
    pub fn from_object(object: JsObject, context: &mut Context) -> JsResult<Self> {
        if object.borrow().is_set() {
            Ok(Self { inner: object })
        } else {
            context.throw_error("Object is not a Set")
        }
    }
}

impl From<JsSet> for JsObject {
    #[inline]
    fn from(o: JsSet) -> Self {
        o.inner.clone()
    }
}

impl From<JsSet> for JsValue {
    #[inline]
    fn from(o: JsSet) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsSet {
    type Target = JsObject;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsSet {}
