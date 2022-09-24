//! This module implements a wrapper for the Dataview Builtin Javascript Object
use crate::{
    builtins::Dataview,
    object::{JsFunction, JsObject, JsObjectType, ObjectData},
    Context, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsDataview {
    inner: JsObject,
}


impl JsDataview {


    #[inline]
    pub fn from_object(object: JsObject, context: &mut Context) -> Self {
        if object.borrow().is_dataview() {
            Ok(Self {inner: object})
        } else {
            context.throw_type_error("object is not a Dataview")
        }
    }
}

impl From<JsDataview> for JsObject {
    #[inline]
    fn from(o:JsDataview) -> Self {
        o.inner.clone()
    }
}

impl From<JsDataview> for JsValue {
    #[inline]
    fn from(o: JsDataview) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsDataview {
    type Target = JsDataview;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsDataview {}
