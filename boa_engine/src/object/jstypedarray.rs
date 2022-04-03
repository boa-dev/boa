use crate::{
    builtins::typed_array::TypedArray,
    object::{JsArray, JsObject, JsObjectType},
    value::IntoOrUndefined,
    Context, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// JavaScript `TypedArray` rust object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsTypedArray {
    inner: JsObject,
}

impl JsTypedArray {
    #[inline]
    pub fn from_object(object: JsObject, context: &mut Context) -> JsResult<Self> {
        if object.borrow().is_typed_array() {
            Ok(Self { inner: object })
        } else {
            context.throw_type_error("object is not an TypedArray")
        }
    }

    /// Get the length of the array.
    ///
    /// Same a `array.length` in JavaScript.
    #[inline]
    pub fn length(&self, context: &mut Context) -> JsResult<usize> {
        Ok(
            TypedArray::length(&self.inner.clone().into(), &[], context)?
                .as_number()
                .map(|x| x as usize)
                .expect("length should return a number"),
        )
    }

    /// Check if the array is empty, i.e. the `length` is zero.
    #[inline]
    pub fn is_empty(&self, context: &mut Context) -> JsResult<bool> {
        self.inner.length_of_array_like(context).map(|len| len == 0)
    }

    #[inline]
    pub fn at<T>(&self, index: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<i64>,
    {
        TypedArray::at(&self.inner.clone().into(), &[index.into().into()], context)
    }

    #[inline]
    pub fn byte_length(&self, context: &mut Context) -> JsResult<usize> {
        Ok(
            TypedArray::byte_length(&self.inner.clone().into(), &[], context)?
                .as_number()
                .map(|x| x as usize)
                .expect("byteLength should return a number"),
        )
    }

    #[inline]
    pub fn byte_offset(&self, context: &mut Context) -> JsResult<usize> {
        Ok(
            TypedArray::byte_offset(&self.inner.clone().into(), &[], context)?
                .as_number()
                .map(|x| x as usize)
                .expect("byteLength should return a number"),
        )
    }

    #[inline]
    pub fn fill<T>(
        &self,
        value: T,
        start: Option<usize>,
        end: Option<usize>,
        context: &mut Context,
    ) -> JsResult<Self>
    where
        T: Into<JsValue>,
    {
        TypedArray::fill(
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

    pub fn every(
        &self,
        predicate: JsObject,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<bool> {
        let result = TypedArray::every(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .expect("TypedArray.prototype.every should always return boolean");

        Ok(result)
    }

    #[inline]
    pub fn some(
        &self,
        callback: JsObject,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<bool> {
        let result = TypedArray::some(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .expect("TypedArray.prototype.some should always return boolean");

        Ok(result)
    }

    #[inline]
    pub fn sort(&self, compare_fn: Option<JsObject>, context: &mut Context) -> JsResult<Self> {
        TypedArray::sort(
            &self.inner.clone().into(),
            &[compare_fn.into_or_undefined()],
            context,
        )?;

        Ok(self.clone())
    }

    #[inline]
    pub fn filter(
        &self,
        callback: JsObject,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let object = TypedArray::filter(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_object()
        .cloned()
        .expect("TypedArray.prototype.filter should always return object");

        Ok(Self { inner: object })
    }

    #[inline]
    pub fn map(
        &self,
        callback: JsObject,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let object = TypedArray::map(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_object()
        .cloned()
        .expect("TypedArray.prototype.map should always return object");

        Ok(Self { inner: object })
    }

    #[inline]
    pub fn reduce(
        &self,
        callback: JsObject,
        initial_value: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        TypedArray::reduce(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    #[inline]
    pub fn reduce_right(
        &self,
        callback: JsObject,
        initial_value: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        TypedArray::reduceright(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    #[inline]
    pub fn reverse(&self, context: &mut Context) -> JsResult<Self> {
        TypedArray::reverse(&self.inner.clone().into(), &[], context)?;
        Ok(self.clone())
    }

    #[inline]
    pub fn slice(
        &self,
        start: Option<usize>,
        end: Option<usize>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let object = TypedArray::slice(
            &self.inner.clone().into(),
            &[start.into_or_undefined(), end.into_or_undefined()],
            context,
        )?
        .as_object()
        .cloned()
        .expect("TypedArray.prototype.slice should always return object");

        Ok(Self { inner: object })
    }

    #[inline]
    pub fn find(
        &self,
        predicate: JsObject,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        TypedArray::find(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    #[inline]
    pub fn index_of<T>(
        &self,
        search_element: T,
        from_index: Option<usize>,
        context: &mut Context,
    ) -> JsResult<Option<usize>>
    where
        T: Into<JsValue>,
    {
        let index = TypedArray::index_of(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_number()
        .expect("TypedArray.prototype.indexOf should always return number");

        #[allow(clippy::float_cmp)]
        if index == -1.0 {
            Ok(None)
        } else {
            Ok(Some(index as usize))
        }
    }

    #[inline]
    pub fn last_index_of<T>(
        &self,
        search_element: T,
        from_index: Option<usize>,
        context: &mut Context,
    ) -> JsResult<Option<usize>>
    where
        T: Into<JsValue>,
    {
        let index = TypedArray::last_index_of(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
        .as_number()
        .expect("TypedArray.prototype.lastIndexOf should always return number");

        #[allow(clippy::float_cmp)]
        if index == -1.0 {
            Ok(None)
        } else {
            Ok(Some(index as usize))
        }
    }

    #[inline]
    pub fn join(&self, separator: Option<JsString>, context: &mut Context) -> JsResult<JsString> {
        TypedArray::join(
            &self.inner.clone().into(),
            &[separator.into_or_undefined()],
            context,
        )
        .map(|x| {
            x.as_string()
                .cloned()
                .expect("TypedArray.prototype.join always returns string")
        })
    }
}

impl From<JsTypedArray> for JsObject {
    #[inline]
    fn from(o: JsTypedArray) -> Self {
        o.inner.clone()
    }
}

impl From<JsTypedArray> for JsValue {
    #[inline]
    fn from(o: JsTypedArray) -> Self {
        o.inner.clone().into()
    }
}

impl Deref for JsTypedArray {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl JsObjectType for JsTypedArray {}

macro_rules! JsTypedArrayType {
    ($name:ident, $constructor_function:ident, $constructor_object:ident, $element:ty) => {
        #[doc = concat!("JavaScript `", stringify!($constructor_function), "` rust object.")]
        #[derive(Debug, Clone, Trace, Finalize)]
        pub struct $name {
            inner: JsTypedArray,
        }

        impl $name {
            #[inline]
            pub fn from_iter<I>(elements: I, context: &mut Context) -> JsResult<Self>
            where
                I: IntoIterator<Item = $element>,
            {
                let array = JsArray::from_iter(elements.into_iter().map(JsValue::new), context);
                let new_target = context
                    .intrinsics()
                    .constructors()
                    .$constructor_object()
                    .constructor()
                    .into();
                let object = crate::builtins::typed_array::$constructor_function::constructor(
                    &new_target,
                    &[array.into()],
                    context,
                )?
                .as_object()
                .expect("object")
                .clone();

                Ok(Self {
                    inner: JsTypedArray { inner: object },
                })
            }
        }

        impl From<$name> for JsObject {
            #[inline]
            fn from(o: $name) -> Self {
                o.inner.inner.clone()
            }
        }

        impl From<$name> for JsValue {
            #[inline]
            fn from(o: $name) -> Self {
                o.inner.inner.clone().into()
            }
        }

        impl Deref for $name {
            type Target = JsTypedArray;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };
}

JsTypedArrayType!(JsUint8Array, Uint8Array, typed_uint8_array, u8);
JsTypedArrayType!(JsInt8Array, Int8Array, typed_int8_array, i8);
JsTypedArrayType!(JsUint16Array, Uint16Array, typed_uint16_array, u16);
JsTypedArrayType!(JsInt16Array, Int16Array, typed_int16_array, i16);
JsTypedArrayType!(JsUint32Array, Uint32Array, typed_uint32_array, u32);
JsTypedArrayType!(JsInt32Array, Int32Array, typed_int32_array, i32);
JsTypedArrayType!(JsFloat32Array, Float32Array, typed_float32_array, f32);
JsTypedArrayType!(JsFloat64Array, Float64Array, typed_float64_array, f64);
