//! A Rust API wrapper for Boa's `AsyncGenerator` Builtin ECMAScript Object
use super::JsPromise;
use crate::{
    Context, JsNativeError, JsResult, JsValue, builtins::async_generator::AsyncGenerator,
    object::JsObject, value::TryFromJs,
};
use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsAsyncGenerator` provides a wrapper for Boa's implementation of the ECMAScript `AsyncGenerator` builtin object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsAsyncGenerator {
    inner: JsObject,
}

impl JsAsyncGenerator {
    /// Creates a `JsAsyncGenerator` from an async generator `JsObject`.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AsyncGenerator
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is::<AsyncGenerator>() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not an AsyncGenerator")
                .into())
        }
    }

    /// Calls `AsyncGenerator.prototype.next()`.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AsyncGenerator/next
    pub fn next<T>(&self, value: T, context: &mut Context) -> JsResult<JsPromise>
    where
        T: Into<JsValue>,
    {
        let value = AsyncGenerator::next(&self.inner.clone().into(), &[value.into()], context)?;
        let obj = value
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ().with_message("async generator did not return a Promise")
            })?
            .clone();
        JsPromise::from_object(obj)
    }

    /// Calls `AsyncGenerator.prototype.return()`.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AsyncGenerator/return
    pub fn r#return<T>(&self, value: T, context: &mut Context) -> JsResult<JsPromise>
    where
        T: Into<JsValue>,
    {
        let value = AsyncGenerator::r#return(&self.inner.clone().into(), &[value.into()], context)?;
        let obj = value
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ().with_message("async generator did not return a Promise")
            })?
            .clone();
        JsPromise::from_object(obj)
    }

    /// Calls `AsyncGenerator.prototype.throw()`.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AsyncGenerator/throw
    pub fn throw<T>(&self, value: T, context: &mut Context) -> JsResult<JsPromise>
    where
        T: Into<JsValue>,
    {
        let value = AsyncGenerator::throw(&self.inner.clone().into(), &[value.into()], context)?;
        let obj = value
            .as_object()
            .ok_or_else(|| {
                JsNativeError::typ().with_message("async generator did not return a Promise")
            })?
            .clone();
        JsPromise::from_object(obj)
    }
}

impl From<JsAsyncGenerator> for JsObject {
    #[inline]
    fn from(o: JsAsyncGenerator) -> Self {
        o.inner.clone()
    }
}

impl From<JsAsyncGenerator> for JsValue {
    #[inline]
    fn from(o: JsAsyncGenerator) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsAsyncGenerator {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsAsyncGenerator {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        if let Some(o) = value.as_object() {
            Self::from_object(o.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not an AsyncGenerator object")
                .into())
        }
    }
}
