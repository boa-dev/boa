use std::ops::Deref;

use boa_gc::{Finalize, Trace};

use crate::{
    builtins::Set,
    object::{JsObject, JsObjectType},
    Context, JsResult, JsValue,
};

use super::JsFunction;

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

    /// Appends value to the Set object.
    /// Returns the Set object with added value.
    ///
    /// Same as JavaScript's `set.add(value)`.
    #[inline]
    pub fn add<T>(&self, value: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        // let object = Set::add(&self.inner.clone().into(), &[value.into()], context)?
        //     .as_object()
        //     .cloned()
        //     .expect("Set.prototype.add should always return `Set` object.");

        // Self::from_object(object, context)

        // TEST
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
        match Set::delete(&self.inner.clone().into(), &[value.into()], context)? {
            JsValue::Boolean(bool) => Ok(bool),
            _ => Err(JsValue::Undefined),
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
        match Set::has(&self.inner.clone().into(), &[value.into()], context)? {
            JsValue::Boolean(bool) => Ok(bool),
            _ => Err(JsValue::Undefined),
        }
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
    pub fn from_object(object: JsObject, context: &mut Context) -> JsResult<Self> {
        if object.borrow().is_set() {
            Ok(Self { inner: object })
        } else {
            context.throw_error("Object is not a Set")
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
