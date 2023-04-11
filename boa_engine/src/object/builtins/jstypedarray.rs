//! Rust API wrappers for the `TypedArray` Builtin ECMAScript Objects
use crate::{
    builtins::typed_array::TypedArray,
    builtins::BuiltInConstructor,
    error::JsNativeError,
    object::{JsArrayBuffer, JsFunction, JsObject, JsObjectType},
    value::{IntoOrUndefined, TryFromJs},
    Context, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsTypedArray` provides a wrapper for Boa's implementation of the ECMAScript `TypedArray`
/// builtin object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsTypedArray {
    inner: JsValue,
}

impl JsTypedArray {
    /// Create a [`JsTypedArray`] from a [`JsObject`], if the object is not a typed array throw a
    /// `TypeError`.
    ///
    /// This does not clone the fields of the typed array, it only does a shallow clone of the
    /// object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is_typed_array() {
            Ok(Self {
                inner: object.into(),
            })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a TypedArray")
                .into())
        }
    }

    /// Get the length of the array.
    ///
    /// Same a `array.length` in JavaScript.
    #[inline]
    pub fn length(&self, context: &mut Context<'_>) -> JsResult<usize> {
        Ok(TypedArray::length(&self.inner, &[], context)?
            .as_number()
            .map(|x| x as usize)
            .expect("length should return a number"))
    }

    /// Check if the array is empty, i.e. the `length` is zero.
    #[inline]
    pub fn is_empty(&self, context: &mut Context<'_>) -> JsResult<bool> {
        Ok(self.length(context)? == 0)
    }

    /// Calls `TypedArray.prototype.at()`.
    pub fn at<T>(&self, index: T, context: &mut Context<'_>) -> JsResult<JsValue>
    where
        T: Into<i64>,
    {
        TypedArray::at(&self.inner, &[index.into().into()], context)
    }

    /// Returns `TypedArray.prototype.byteLength`.
    #[inline]
    pub fn byte_length(&self, context: &mut Context<'_>) -> JsResult<usize> {
        Ok(TypedArray::byte_length(&self.inner, &[], context)?
            .as_number()
            .map(|x| x as usize)
            .expect("byteLength should return a number"))
    }

    /// Returns `TypedArray.prototype.byteOffset`.
    #[inline]
    pub fn byte_offset(&self, context: &mut Context<'_>) -> JsResult<usize> {
        Ok(TypedArray::byte_offset(&self.inner, &[], context)?
            .as_number()
            .map(|x| x as usize)
            .expect("byteLength should return a number"))
    }

    /// Calls `TypedArray.prototype.fill()`.
    pub fn fill<T>(
        &self,
        value: T,
        start: Option<usize>,
        end: Option<usize>,
        context: &mut Context<'_>,
    ) -> JsResult<Self>
    where
        T: Into<JsValue>,
    {
        TypedArray::fill(
            &self.inner,
            &[
                value.into(),
                start.into_or_undefined(),
                end.into_or_undefined(),
            ],
            context,
        )?;
        Ok(self.clone())
    }

    /// Calls `TypedArray.prototype.every()`.
    pub fn every(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        let result = TypedArray::every(
            &self.inner,
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .expect("TypedArray.prototype.every should always return boolean");

        Ok(result)
    }

    /// Calls `TypedArray.prototype.some()`.
    #[inline]
    pub fn some(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<bool> {
        let result = TypedArray::some(
            &self.inner,
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
        .as_boolean()
        .expect("TypedArray.prototype.some should always return boolean");

        Ok(result)
    }

    /// Calls `TypedArray.prototype.sort()`.
    #[inline]
    pub fn sort(
        &self,
        compare_fn: Option<JsFunction>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        TypedArray::sort(&self.inner, &[compare_fn.into_or_undefined()], context)?;

        Ok(self.clone())
    }

    /// Calls `TypedArray.prototype.filter()`.
    #[inline]
    pub fn filter(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let object = TypedArray::filter(
            &self.inner,
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?;

        Ok(Self { inner: object })
    }

    /// Calls `TypedArray.prototype.map()`.
    #[inline]
    pub fn map(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let object = TypedArray::map(
            &self.inner,
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?;

        Ok(Self { inner: object })
    }

    /// Calls `TypedArray.prototype.reduce()`.
    #[inline]
    pub fn reduce(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        TypedArray::reduce(
            &self.inner,
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    /// Calls `TypedArray.prototype.reduceRight()`.
    #[inline]
    pub fn reduce_right(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        TypedArray::reduceright(
            &self.inner,
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    /// Calls `TypedArray.prototype.reverse()`.
    #[inline]
    pub fn reverse(&self, context: &mut Context<'_>) -> JsResult<Self> {
        TypedArray::reverse(&self.inner, &[], context)?;
        Ok(self.clone())
    }

    /// Calls `TypedArray.prototype.slice()`.
    #[inline]
    pub fn slice(
        &self,
        start: Option<usize>,
        end: Option<usize>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let object = TypedArray::slice(
            &self.inner,
            &[start.into_or_undefined(), end.into_or_undefined()],
            context,
        )?;

        Ok(Self { inner: object })
    }

    /// Calls `TypedArray.prototype.find()`.
    #[inline]
    pub fn find(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        TypedArray::find(
            &self.inner,
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    /// Calls `TypedArray.prototype.indexOf()`.
    pub fn index_of<T>(
        &self,
        search_element: T,
        from_index: Option<usize>,
        context: &mut Context<'_>,
    ) -> JsResult<Option<usize>>
    where
        T: Into<JsValue>,
    {
        let index = TypedArray::index_of(
            &self.inner,
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

    /// Calls `TypedArray.prototype.lastIndexOf()`.
    pub fn last_index_of<T>(
        &self,
        search_element: T,
        from_index: Option<usize>,
        context: &mut Context<'_>,
    ) -> JsResult<Option<usize>>
    where
        T: Into<JsValue>,
    {
        let index = TypedArray::last_index_of(
            &self.inner,
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

    /// Calls `TypedArray.prototype.join()`.
    #[inline]
    pub fn join(
        &self,
        separator: Option<JsString>,
        context: &mut Context<'_>,
    ) -> JsResult<JsString> {
        TypedArray::join(&self.inner, &[separator.into_or_undefined()], context).map(|x| {
            x.as_string()
                .cloned()
                .expect("TypedArray.prototype.join always returns string")
        })
    }
}

impl From<JsTypedArray> for JsObject {
    #[inline]
    fn from(o: JsTypedArray) -> Self {
        o.inner
            .as_object()
            .expect("should always be an object")
            .clone()
    }
}

impl From<JsTypedArray> for JsValue {
    #[inline]
    fn from(o: JsTypedArray) -> Self {
        o.inner.clone()
    }
}

impl Deref for JsTypedArray {
    type Target = JsObject;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.as_object().expect("should always be an object")
    }
}

impl JsObjectType for JsTypedArray {}

impl TryFromJs for JsTypedArray {
    fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
        match value {
            JsValue::Object(o) => Self::from_object(o.clone()),
            _ => Err(JsNativeError::typ()
                .with_message("value is not a TypedArray object")
                .into()),
        }
    }
}

macro_rules! JsTypedArrayType {
    (
        $name:ident,
        $constructor_function:ident,
        $checker_function:ident,
        $constructor_object:ident,
        $element:ty
    ) => {

        #[doc = concat!(
            "`", stringify!($name),
            "` provides a wrapper for Boa's implementation of the ECMAScript `",
            stringify!($constructor_function) ,"` builtin object."
        )]
        #[derive(Debug, Clone, Trace, Finalize)]
        pub struct $name {
            inner: JsTypedArray,
        }

        impl $name {
            #[doc = concat!("Creates a `", stringify!($name),
                "` using a [`JsObject`]. It will make sure that the object is of the correct kind."
            )]
            #[inline]
            pub fn from_object(object: JsObject) -> JsResult<Self> {
                if object.$checker_function() {
                    Ok(Self {
                        inner: JsTypedArray {
                            inner: object.into(),
                        },
                    })
                } else {
                    Err(JsNativeError::typ()
                        .with_message("object is not a TypedArray")
                        .into())
                }
            }

            /// Create the typed array from a [`JsArrayBuffer`].
            pub fn from_array_buffer(
                array_buffer: JsArrayBuffer,
                context: &mut Context<'_>,
            ) -> JsResult<Self> {
                let new_target = context
                    .intrinsics()
                    .constructors()
                    .$constructor_object()
                    .constructor()
                    .into();
                let object = crate::builtins::typed_array::$constructor_function::constructor(
                    &new_target,
                    &[array_buffer.into()],
                    context,
                )?
                .as_object()
                .expect("object")
                .clone();

                Ok(Self {
                    inner: JsTypedArray {
                        inner: object.into(),
                    },
                })
            }

            /// Create the typed array from an iterator.
            pub fn from_iter<I>(elements: I, context: &mut Context<'_>) -> JsResult<Self>
            where
                I: IntoIterator<Item = $element>,
            {
                let bytes: Vec<_> = elements
                    .into_iter()
                    .flat_map(<$element>::to_ne_bytes)
                    .collect();
                let array_buffer = JsArrayBuffer::from_byte_block(bytes, context)?;
                let new_target = context
                    .intrinsics()
                    .constructors()
                    .$constructor_object()
                    .constructor()
                    .into();
                let object = crate::builtins::typed_array::$constructor_function::constructor(
                    &new_target,
                    &[array_buffer.into()],
                    context,
                )?
                .as_object()
                .expect("object")
                .clone();

                Ok(Self {
                    inner: JsTypedArray {
                        inner: object.into(),
                    },
                })
            }
        }

        impl From<$name> for JsObject {
            #[inline]
            fn from(o: $name) -> Self {
                o.inner
                    .inner
                    .as_object()
                    .expect("should always be an object")
                    .clone()
            }
        }

        impl From<$name> for JsValue {
            #[inline]
            fn from(o: $name) -> Self {
                o.inner.inner.clone()
            }
        }

        impl Deref for $name {
            type Target = JsTypedArray;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl TryFromJs for $name {
            fn try_from_js(value: &JsValue, _context: &mut Context<'_>) -> JsResult<Self> {
                match value {
                    JsValue::Object(o) => Self::from_object(o.clone()),
                    _ => Err(JsNativeError::typ()
                        .with_message(concat!(
                            "value is not a ",
                            stringify!($constructor_function),
                            " object"
                        ))
                        .into()),
                }
            }
        }
    };
}

JsTypedArrayType!(
    JsUint8Array,
    Uint8Array,
    is_typed_uint8_array,
    typed_uint8_array,
    u8
);
JsTypedArrayType!(
    JsInt8Array,
    Int8Array,
    is_typed_int8_array,
    typed_int8_array,
    i8
);
JsTypedArrayType!(
    JsUint16Array,
    Uint16Array,
    is_typed_uint16_array,
    typed_uint16_array,
    u16
);
JsTypedArrayType!(
    JsInt16Array,
    Int16Array,
    is_typed_int16_array,
    typed_int16_array,
    i16
);
JsTypedArrayType!(
    JsUint32Array,
    Uint32Array,
    is_typed_uint32_array,
    typed_uint32_array,
    u32
);
JsTypedArrayType!(
    JsInt32Array,
    Int32Array,
    is_typed_int32_array,
    typed_int32_array,
    i32
);
JsTypedArrayType!(
    JsFloat32Array,
    Float32Array,
    is_typed_float32_array,
    typed_float32_array,
    f32
);
JsTypedArrayType!(
    JsFloat64Array,
    Float64Array,
    is_typed_float64_array,
    typed_float64_array,
    f64
);
