//! This module implements a wrapper for the `Set` Builtin JavaScript Object
use std::ops::Deref;

use boa_gc::{Finalize, Trace};

use crate::{
    builtins::Set,
    error::JsNativeError,
    object::{JsFunction, JsObject, JsObjectType, JsSetIterator},
    Context, JsResult, JsValue,
};

/// `JsSet` provides a wrapper for Boa's implementation of the JavaScript `Set` object.
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
        let inner = Set::set_create(None, context);

        Self { inner }
    }

    /// Returns the size of the `Set` as an integer.
    ///
    /// Same as JavaScript's `set.size`.
    #[inline]
    pub fn size(&self) -> JsResult<usize> {
        Set::get_size(&self.inner.clone().into())
    }

    /// Appends value to the Set object.
    /// Returns the Set object with added value.
    ///
    /// Same as JavaScript's `set.add(value)`.
    #[inline]
    pub fn add<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        self.add_items(&[value.into()], context)
    }

    /// Adds slice as a single element.
    /// Returns the Set object with added slice.
    ///
    /// Same as JavaScript's `set.add(["one", "two", "three"])`
    #[inline]
    pub fn add_items(&self, items: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Set::add(&self.inner.clone().into(), items, context)
    }

    /// Removes all elements from the Set object.
    /// Returns `Undefined`.
    ///
    /// Same as JavaScript's `set.clear()`.
    #[inline]
    pub fn clear(&self, context: &mut Context) -> JsResult<JsValue> {
        Set::clear(&self.inner.clone().into(), &[JsValue::Null], context)
    }

    /// Removes the element associated to the value.
    /// Returns a boolean asserting whether an element was
    /// successfully removed or not.
    ///
    /// Same as JavaScript's `set.delete(value)`.
    #[inline]
    pub fn delete<T>(&self, value: T, context: &mut Context) -> JsResult<bool>
    where
        T: Into<JsValue>,
    {
        // TODO: Make `delete` return a native `bool`
        match Set::delete(&self.inner.clone().into(), &[value.into()], context)? {
            JsValue::Boolean(bool) => Ok(bool),
            _ => unreachable!("`delete` must always return a bool"),
        }
    }

    /// Returns a boolean asserting whether an element is present
    /// with the given value in the Set object or not.
    ///
    /// Same as JavaScript's `set.has(value)`.
    #[inline]
    pub fn has<T>(&self, value: T, context: &mut Context) -> JsResult<bool>
    where
        T: Into<JsValue>,
    {
        // TODO: Make `has` return a native `bool`
        match Set::has(&self.inner.clone().into(), &[value.into()], context)? {
            JsValue::Boolean(bool) => Ok(bool),
            _ => unreachable!("`has` must always return a bool"),
        }
    }

    /// Returns a new iterator object that yields the values
    /// for each element in the Set object in insertion order.
    ///
    /// Same as JavaScript's `set.values()`.
    #[inline]
    pub fn values(&self, context: &mut Context) -> JsResult<JsSetIterator> {
        let iterator_object = Set::values(&self.inner.clone().into(), &[JsValue::Null], context)?
            .get_iterator(context, None, None)?;

        JsSetIterator::from_object(iterator_object.iterator().clone())
    }

    /// Alias for `Set.prototype.values()`
    /// Returns a new iterator object that yields the values
    /// for each element in the Set object in insertion order.
    ///
    /// Same as JavaScript's `set.keys()`.
    #[inline]
    pub fn keys(&self, context: &mut Context) -> JsResult<JsSetIterator> {
        let iterator_object = Set::values(&self.inner.clone().into(), &[JsValue::Null], context)?
            .get_iterator(context, None, None)?;

        JsSetIterator::from_object(iterator_object.iterator().clone())
    }

    /// Calls callbackFn once for each value present in the Set object,
    /// in insertion order.
    /// Returns `Undefined`.
    ///
    /// Same as JavaScript's `set.forEach(values)`.
    #[inline]
    pub fn for_each(
        &self,
        callback: JsFunction,
        this_arg: JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Set::for_each(
            &self.inner.clone().into(),
            &[callback.into(), this_arg],
            context,
        )
    }

    /// Utility: Creates `JsSet` from `JsObject`, if not a Set throw `TypeError`.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.borrow().is_set() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("Object is not a Set")
                .into())
        }
    }

    /// Utility: Creates a `JsSet` from a `<IntoIterator<Item = JsValue>` convertible object.
    #[inline]
    pub fn from_iter<I>(elements: I, context: &mut Context) -> Self
    where
        I: IntoIterator<Item = JsValue>,
    {
        let inner = Set::create_set_from_list(elements, context);
        Self { inner }
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
