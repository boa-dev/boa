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

    #[inline]
    pub fn buffer(&self, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_buffer(&self.inner.clone().into(), &[], context)
    }

    #[inline]
    pub fn byte_length(&self, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_byte_length(&self.inner.clone().into(), &[], context)
    }

    #[inline]
    pub fn byte_offset(&self, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_byte_offset(&self.inner.clone().into(), &[], context)
    }

    #[inline]
    pub fn get_big_int64(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_big_int64(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn get_big_uint64(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_big_uint64(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn get_float32(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_float32(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn get_float64(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_float64(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn get_int8(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_int8(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn get_int16(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_int16(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn get_int32(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_int32(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }
    
    #[inline]
    pub fn get_uint8(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_uint8(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn get_unit16(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_uint16(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn get_unit32(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::get_uint32(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn set_big_int64(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::set_big_int64(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn set_big_uint64(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::set_big_uint64(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn set_float32(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::set_float32(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn set_float64(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::set_float64(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn set_int8(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::set_int8(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn set_int16(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::set_int16(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn set_int32(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::set_int32(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }
    
    #[inline]
    pub fn set_uint8(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::set_uint8(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn set_unit16(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::set_uint16(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
    }

    #[inline]
    pub fn set_unit32(&self, byte_offset: usize, is_little_edian: bool, context: &mut Context) -> JsResult<JsValue> {
        DataView::set_uint32(&self.inner.clone().into(), &[byte_offset.into(), is_little_edian.into()], context)
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
