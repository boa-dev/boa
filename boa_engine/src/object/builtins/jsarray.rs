//! A Rust API wrapper for Boa's `Array` Builtin ECMAScript Object
use crate::{
    builtins::Array,
    error::JsNativeError,
    object::{JsFunction, JsObject, JsObjectType},
    value::{IntoOrUndefined, TryFromJs},
    Context, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsArray` provides a wrapper for Boa's implementation of the JavaScript `Array` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsArray {
    inner: JsObject,
}

impl JsArray {
    /// Create a new empty array.
    #[inline]
    pub fn new(context: &mut Context<'_>) -> Self {
        let inner = Array::array_create(0, None, context)
            .expect("creating an empty array with the default prototype must not fail");

        Self { inner }
    }

    /// Create an array from a `IntoIterator<Item = JsValue>` convertible object.
    pub fn from_iter<I>(elements: I, context: &mut Context<'_>) -> Self
    where
        I: IntoIterator<Item = JsValue>,
    {
        Self {
            inner: Array::create_array_from_list(elements, context),
        }
    }

    /// Create a [`JsArray`] from a [`JsObject`], if the object is not an array throw a `TypeError`.
    ///
    /// This does not clone the fields of the array, it only does a shallow clone of the object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is_array() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not an Array")
                .into())
        }
    }

    /// Get the length of the array.
    ///
    /// Same a `array.length` in JavaScript.
    #[inline]
    pub fn length(&self, context: &mut Context<'_>) -> JsResult<u64> {
        self.inner.length_of_array_like(context)
    }

    /// Check if the array is empty, i.e. the `length` is zero.
    #[inline]
    pub fn is_empty(&self, context: &mut Context<'_>) -> JsResult<bool> {
        self.inner.length_of_array_like(context).map(|len| len == 0)
    }

    /// Push an element to the array.
    pub fn push<T>(&self, value: T, context: &mut Context<'_>) -> JsResult<JsValue>
    where
        T: Into<JsValue>,
    {
        self.push_items(&[value.into()], context)
    }

    /// Pushes a slice of elements to the array.
    #[inline]
    pub fn push_items(&self, items: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Array::push(&self.inner.clone().into(), items, context)
    }

    /// Pops an element from the array.
    #[inline]
    pub fn pop(&self, context: &mut Context<'_>) -> JsResult<JsValue> {
        Array::pop(&self.inner.clone().into(), &[], context)
    }

    /// Calls `Array.prototype.at()`.
    pub fn at<T>(&self, index: T, context: &mut Context<'_>) -> JsResult<JsValue>
    where
        T: Into<i64>,
    {
        Array::at(&self.inner.clone().into(), &[index.into().into()], context)
    }

    /// Calls `Array.prototype.shift()`.
    #[inline]
    pub fn shift(&self, context: &mut Context<'_>) -> JsResult<JsValue> {
        Array::shift(&self.inner.clone().into(), &[], context)
    }

    /// Calls `Array.prototype.unshift()`.
    #[inline]
    pub fn unshift(&self, items: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        Array::shift(&self.inner.clone().into(), items, context)
    }

    /// Calls `Array.prototype.reverse()`.
    #[inline]
    pub fn reverse(&self, context: &mut Context<'_>) -> JsResult<Self> {
        Array::reverse(&self.inner.clone().into(), &[], context)?;
        Ok(self.clone())
    }

    /// Calls `Array.prototype.concat()`.
    #[inline]
    pub fn concat(&self, items: &[JsValue], context: &mut Context<'_>) -> JsResult<Self> {
        let object = Array::concat(&self.inner.clone().into(), items, context)?
            .as_object()
            .cloned()
            .expect("Array.prototype.filter should always return object");

        Self::from_object(object)
    }

    /// Calls `Array.prototype.join()`.
    #[inline]
    pub fn join(
        &self,
        separator: Option<JsString>,
        context: &mut Context<'_>,
    ) -> JsResult<JsString> {
        Array::join(
            &self.inner.clone().into(),
            &[separator.into_or_undefined()],
            context,
        )
        .map(|x| {
            x.as_string()
                .cloned()
                .expect("Array.prototype.join always returns string")
        })
    }

    /// Calls `Array.prototype.fill()`.
    pub fn fill<T>(
        &self,
        value: T,
        start: Option<u32>,
        end: Option<u32>,
        context: &mut Context<'_>,
    ) -> JsResult<Self>
    where
        T: Into<JsValue>,
    {
        Array::fill(
            &self.inner.clone().into(),
            &[
                value.into(),
                start.into_or_undefined(),
                end.into_or_undefined(),
            ],
            context,
        )?;
        Ok(self.clone())
    }

    /// Calls `Array.prototype.indexOf()`.
    pub fn index_of<T>(
        &self,
        search_element: T,
        from_index: Option<u32>,
        context: &mut Context<'_>,
    ) -> JsResult<Option<u32>>
    where
        T: Into<JsValue>,
    {
        let index = Array::index_of(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_number()
        .expect("Array.prototype.indexOf should always return number");

        #[allow(clippy::float_cmp)]
        if index == -1.0 {
            Ok(None)
        } else {
            Ok(Some(index as u32))
        }
    }

    /// Calls `Array.prototype.lastIndexOf()`.
    pub fn last_index_of<T>(
        &self,
        search_element: T,
        from_index: Option<u32>,
        context: &mut Context<'_>,
    ) -> JsResult<Option<u32>>
    where
        T: Into<JsValue>,
    {
        let index = Array::last_index_of(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_number()
        .expect("Array.prototype.lastIndexOf should always return number");

        #[allow(clippy::float_cmp)]
        if index == -1.0 {
            Ok(None)
        } else {
            Ok(Some(index as u32))
        }
    }

    /// Calls `Array.prototype.find()`.
    #[inline]
    pub fn find(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Array::find(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    /// Calls `Array.prototype.filter()`.
    #[inline]
    pub fn filter(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let object = Array::filter(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_object()
        .cloned()
        .expect("Array.prototype.filter should always return object");

        Self::from_object(object)
    }

    /// Calls `Array.prototype.map()`.
    #[inline]
    pub fn map(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let object = Array::map(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_object()
        .cloned()
        .expect("Array.prototype.map should always return object");

        Self::from_object(object)
    }

    /// Calls `Array.prototype.every()`.
    #[inline]
    pub fn every(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        let result = Array::every(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .expect("Array.prototype.every should always return boolean");

        Ok(result)
    }

    /// Calls `Array.prototype.some()`.
    #[inline]
    pub fn some(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        let result = Array::some(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .expect("Array.prototype.some should always return boolean");

        Ok(result)
    }

    /// Calls `Array.prototype.sort()`.
    #[inline]
    pub fn sort(
        &self,
        compare_fn: Option<JsFunction>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        Array::sort(
            &self.inner.clone().into(),
            &[compare_fn.into_or_undefined()],
            context,
        )?;

        Ok(self.clone())
    }

    /// Calls `Array.prototype.slice()`.
    #[inline]
    pub fn slice(
        &self,
        start: Option<u32>,
        end: Option<u32>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let object = Array::slice(
            &self.inner.clone().into(),
            &[start.into_or_undefined(), end.into_or_undefined()],
            context,
        )?
        .as_object()
        .cloned()
        .expect("Array.prototype.slice should always return object");

        Self::from_object(object)
    }

    /// Calls `Array.prototype.reduce()`.
    #[inline]
    pub fn reduce(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Array::reduce(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    /// Calls `Array.prototype.reduceRight()`.
    #[inline]
    pub fn reduce_right(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Array::reduce_right(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }
}

impl From<JsArray> for JsObject {
    #[inline]
    fn from(o: JsArray) -> Self {
        o.inner.clone()
    }
}

impl From<JsArray> for JsValue {
    #[inline]
    fn from(o: JsArray) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsArray {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsArray {}

impl TryFromJs for JsArray {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Self::from_object(o.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("value is not an Array object")
                .into()),
        }
    }
}
