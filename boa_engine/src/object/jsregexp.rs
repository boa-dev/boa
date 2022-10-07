//! This module implements a wrapper for the `RegExp` Builtin Javascript Object
use crate::{
    builtins::RegExp,
    object::{JsArray, JsObject, JsObjectType},
    Context, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsRegExp` provides a wrapper for Boa's implementation of the JavaScript `RegExp` builtin object
///
/// # Examples
///
/// Create a `JsRegExp` and run RegExp.prototype.test( String )
///
/// ```
/// # use boa_engine::{
/// #  object::JsRegExp,
/// #  Context, JsValue,
/// # };
///
/// // Initialize the `Context`
/// let context = &mut Context::default();
///
/// // Create a new RegExp with pattern and flags
/// let regexp = JsRegExp::new("foo", "gi", context).unwrap();
///
/// let test_result = regexp.test("football", context).unwrap();
/// assert!(test_result);
///
/// let to_string = regexp.to_string(context).unwrap();
/// assert_eq!(to_string, String::from("/foo/gi"));
///
/// ```
///
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsRegExp {
    inner: JsObject,
}

impl JsRegExp {
    #[inline]
    pub fn new<S>(pattern: S, flags: S, context: &mut Context) -> JsResult<Self>
    where
        S: Into<JsValue>,
    {
        let constructor = &context
            .intrinsics()
            .constructors()
            .regexp()
            .constructor()
            .into();
        let obj = RegExp::alloc(constructor, context)?;

        let regexp = RegExp::initialize(obj, &pattern.into(), &flags.into(), context)?
            .as_object()
            .expect("RegExp::initialize must return a RegExp object")
            .clone();

        Ok(Self { inner: regexp })
    }

    #[inline]
    pub fn from_object(object: JsObject, context: &mut Context) -> JsResult<Self> {
        if object.borrow().is_regexp() {
            Ok(Self { inner: object })
        } else {
            context.throw_type_error("object is not a RegExp")
        }
    }

    #[inline]
    pub fn has_indices(&self, context: &mut Context) -> JsResult<bool> {
        RegExp::get_has_indices(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    #[inline]
    pub fn global(&self, context: &mut Context) -> JsResult<bool> {
        RegExp::get_global(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    #[inline]
    pub fn ignore_case(&self, context: &mut Context) -> JsResult<bool> {
        RegExp::get_ignore_case(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    #[inline]
    pub fn multiline(&self, context: &mut Context) -> JsResult<bool> {
        RegExp::get_multiline(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    #[inline]
    pub fn dot_all(&self, context: &mut Context) -> JsResult<bool> {
        RegExp::get_dot_all(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    #[inline]
    pub fn unicode(&self, context: &mut Context) -> JsResult<bool> {
        RegExp::get_unicode(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    #[inline]
    pub fn sticky(&self, context: &mut Context) -> JsResult<bool> {
        RegExp::get_sticky(&self.inner.clone().into(), &[], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    #[inline]
    pub fn flags(&self, context: &mut Context) -> JsResult<String> {
        RegExp::get_flags(&self.inner.clone().into(), &[], context).map(|v| {
            v.as_string()
                .expect("value must be string")
                .deref()
                .to_owned()
        })
    }

    #[inline]
    pub fn source(&self, context: &mut Context) -> JsResult<String> {
        RegExp::get_source(&self.inner.clone().into(), &[], context).map(|v| {
            v.as_string()
                .expect("value must be string")
                .deref()
                .to_owned()
        })
    }

    #[inline]
    pub fn test<S>(&self, search_string: S, context: &mut Context) -> JsResult<bool>
    where
        S: Into<JsValue>,
    {
        RegExp::test(&self.inner.clone().into(), &[search_string.into()], context)
            .map(|v| v.as_boolean().expect("value must be a bool"))
    }

    #[inline]
    pub fn exec<S>(&self, search_string: S, context: &mut Context) -> JsResult<Option<JsArray>>
    where
        S: Into<JsValue>,
    {
        RegExp::exec(&self.inner.clone().into(), &[search_string.into()], context).map(|v| {
            if v.is_null() {
                None
            } else {
                Some(
                    JsArray::from_object(
                        v.to_object(context).expect("v must be an array"),
                        context,
                    )
                    .expect("from_object must not fail if v is an array object"),
                )
            }
        })
    }

    #[inline]
    pub fn to_string(&self, context: &mut Context) -> JsResult<String> {
        RegExp::to_string(&self.inner.clone().into(), &[], context).map(|v| {
            v.as_string()
                .expect("value must be a string")
                .deref()
                .to_owned()
        })
    }
}

impl From<JsRegExp> for JsObject {
    #[inline]
    fn from(o: JsRegExp) -> Self {
        o.inner.clone()
    }
}

impl From<JsRegExp> for JsValue {
    #[inline]
    fn from(o: JsRegExp) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsRegExp {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsRegExp {}
