//! This module implements a wrapper for the RegExp Builtin Javascript Object
use crate::{
    builtins::RegExp,
    object::{JsObject, JsObjectType},
    Context, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsRegExp` provides a wrapper for Boa's implementation of the JavaScript `RegExp` builtin object
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsRegExp {
    inner: JsObject,
}

impl JsRegExp {

    #[inline]
    pub fn new() -> Self {

        Self {
            inner: JsObject::new()
        }
    }

    #[inline]
    pub fn from_object(object: JsObject, context: &mut Context) -> JsResult<Self> {
        if object.borrow().is_regexp() {
            Ok(Self{inner: object})
        } else {
            context.throw_type_error("object is not a RegExp")
        }
    }

    #[inline]
    pub fn flags(&self) -> JsResult<JsValue> {

    }

    #[inline]
    pub fn dot_all(&self) -> JsResult<JsValue> {

    }

    #[inline]
    pub fn global(&self) -> JsResult<JsValue> {

    }

    #[inline]
    pub fn has_indices(&self) -> JsResult<JsValue> {

    }

    #[inline]
    pub fn ignore_case(&self) -> JsResult<JsValue> {

    }

    #[inline]
    pub fn multiline(&self) -> JsResult<JsValue> {

    }

    #[inline]
    pub fn source(&self) -> JsResult<JsValue> {
        
    }

    #[inline]
    pub fn unicode(&self) -> JsResult<JsValue> {

    }

    #[inline]
    pub fn last_index(&self) -> JsResult<JsValue> {

    }

    #[inline]
    pub fn exec(&self) -> JsResult<JsValue> {

    }

    #[inline]
    pub fn test(&self) -> JsResult<JsValue> {

    }

    #[inline]
    pub fn to_string(&self) -> JsResult<JsValue> {

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
    fn deref(&self) -> &self::Target {
        &self.inner
    }
}
