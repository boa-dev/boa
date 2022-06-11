// API wrapper for MapIterator
use crate::{
    builtins::map::map_iterator::MapIterator, object::JsObject, Context, JsResult, JsValue,
};

use boa_gc::{Finalize, Trace};
use std::ops::Deref;

#[derive(Debug, Clone, Finalize, Trace)]
pub struct JsMapIterator {
    inner: JsObject,
}

impl JsMapIterator {
    /// Create a [`JsMapIterator`] from a [`JsObject`]. If object is not a `MapIterator`, throw `TypeError`
    #[inline]
    pub fn from_object(object: JsObject, context: &mut Context) -> JsResult<Self> {
        if object.borrow().is_map_iterator() {
            Ok(Self { inner: object })
        } else {
            context.throw_type_error("object is not a MapIterator")
        }
    }

    pub fn next(&self, context: &mut Context) -> JsResult<JsValue> {
        MapIterator::next(&self.inner.clone().into(), &[], context)
    }
}

impl From<JsMapIterator> for JsObject {
    #[inline]
    fn from(o: JsMapIterator) -> Self {
        o.inner.clone()
    }
}

impl From<JsMapIterator> for JsValue {
    #[inline]
    fn from(o: JsMapIterator) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsMapIterator {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
