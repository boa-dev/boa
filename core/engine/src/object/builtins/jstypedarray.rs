//! Rust API wrappers for the `TypedArray` Builtin ECMAScript Objects
use crate::{
    builtins::typed_array::BuiltinTypedArray,
    builtins::{typed_array::TypedArray, BuiltInConstructor},
    error::JsNativeError,
    object::{JsArrayBuffer, JsFunction, JsObject},
    value::{IntoOrUndefined, TryFromJs},
    Context, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use std::ops::Deref;

/// `JsTypedArray` provides a wrapper for Boa's implementation of the ECMAScript `TypedArray`
/// builtin object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct JsTypedArray {
    inner: JsObject,
}

impl JsTypedArray {
    /// Create a [`JsTypedArray`] from a [`JsObject`], if the object is not a typed array throw a
    /// `TypeError`.
    ///
    /// This does not clone the fields of the typed array, it only does a shallow clone of the
    /// object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        if object.is::<TypedArray>() {
            Ok(Self { inner: object })
        } else {
            Err(JsNativeError::typ()
                .with_message("object is not a TypedArray")
                .into())
        }
    }

    /// Get the length of the array.
    ///
    /// Same as `array.length` in JavaScript.
    #[inline]
    pub fn length(&self, context: &mut Context) -> JsResult<usize> {
        Ok(
            BuiltinTypedArray::length(&self.inner.clone().into(), &[], context)?
                .as_number()
                .map(|x| x as usize)
                .expect("length should return a number"),
        )
    }

    /// Check if the array is empty, i.e. the `length` is zero.
    #[inline]
    pub fn is_empty(&self, context: &mut Context) -> JsResult<bool> {
        Ok(self.length(context)? == 0)
    }

    /// Calls `TypedArray.prototype.at()`.
    pub fn at<T>(&self, index: T, context: &mut Context) -> JsResult<JsValue>
    where
        T: Into<i64>,
    {
        BuiltinTypedArray::at(&self.inner.clone().into(), &[index.into().into()], context)
    }

    /// Returns the `ArrayBuffer` referenced by this typed array at construction time.
    ///
    /// Calls `TypedArray.prototype.buffer()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{js_string, JsResult, object::{builtins::{JsUint8Array, JsArrayBuffer}}, property::{PropertyKey}, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::default();
    /// let array_buffer8 = JsArrayBuffer::new(8, context)?;
    /// let array = JsUint8Array::from_array_buffer(array_buffer8, context)?;
    /// assert_eq!(
    ///     array.buffer(context)?.as_object().unwrap().get(PropertyKey::String(js_string!("byteLength")), context).unwrap(),
    ///     JsValue::new(8)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn buffer(&self, context: &mut Context) -> JsResult<JsValue> {
        BuiltinTypedArray::buffer(&self.inner.clone().into(), &[], context)
    }

    /// Returns `TypedArray.prototype.byteLength`.
    #[inline]
    pub fn byte_length(&self, context: &mut Context) -> JsResult<usize> {
        Ok(
            BuiltinTypedArray::byte_length(&self.inner.clone().into(), &[], context)?
                .as_number()
                .map(|x| x as usize)
                .expect("byteLength should return a number"),
        )
    }

    /// Returns `TypedArray.prototype.byteOffset`.
    #[inline]
    pub fn byte_offset(&self, context: &mut Context) -> JsResult<usize> {
        Ok(
            BuiltinTypedArray::byte_offset(&self.inner.clone().into(), &[], context)?
                .as_number()
                .map(|x| x as usize)
                .expect("byteLength should return a number"),
        )
    }

    /// Function that created the instance object. It is the hidden `TypedArray` constructor function,
    /// but each typed array subclass also defines its own constructor property.
    ///
    /// Returns `TypedArray.prototype.constructor`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{JsResult, object::{builtins::JsUint8Array}, JsNativeError, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::default();
    /// let array = JsUint8Array::from_iter(vec![1, 2, 3, 4, 5], context)?;
    /// assert_eq!(
    ///     Err(JsNativeError::typ()
    ///         .with_message("the TypedArray constructor should never be called directly")
    ///         .into()),
    ///     array.constructor(context)
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn constructor(&self, context: &mut Context) -> JsResult<JsValue> {
        BuiltinTypedArray::constructor(&self.inner.clone().into(), &[], context)
    }

    /// Shallow copies part of this typed array to another location in the same typed
    /// array and returns this typed array without modifying its length.
    ///
    /// Returns `TypedArray.prototype.copyWithin()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{JsResult, JsValue, object::{builtins::{JsUint8Array}}, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::default();
    /// let array = JsUint8Array::from_iter(vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8], context)?;
    /// array.copy_within(3, 1, Some(3), context)?;
    /// assert_eq!(array.get(0, context)?, JsValue::new(1.0));
    /// assert_eq!(array.get(1, context)?, JsValue::new(2.0));
    /// assert_eq!(array.get(2, context)?, JsValue::new(3.0));
    /// assert_eq!(array.get(3, context)?, JsValue::new(2.0));
    /// assert_eq!(array.get(4, context)?, JsValue::new(3.0));
    /// assert_eq!(array.get(5, context)?, JsValue::new(6.0));
    /// assert_eq!(array.get(6, context)?, JsValue::new(7.0));
    /// assert_eq!(array.get(7, context)?, JsValue::new(8.0));
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn copy_within<T>(
        &self,
        target: T,
        start: u64,
        end: Option<u64>,
        context: &mut Context,
    ) -> JsResult<Self>
    where
        T: Into<JsValue>,
    {
        let object = BuiltinTypedArray::copy_within(
            &self.inner.clone().into(),
            &[target.into(), start.into(), end.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: object
                .as_object()
                .cloned()
                .expect("`copyWithin` must always return a `TypedArray` on success"),
        })
    }

    /// Calls `TypedArray.prototype.fill()`.
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
        BuiltinTypedArray::fill(
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

    /// Calls `TypedArray.prototype.every()`.
    pub fn every(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<bool> {
        let result = BuiltinTypedArray::every(
            &self.inner.clone().into(),
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
        context: &mut Context,
    ) -> JsResult<bool> {
        let result = BuiltinTypedArray::some(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?
            .as_boolean()
            .expect("TypedArray.prototype.some should always return boolean");

        Ok(result)
    }

    /// Calls `TypedArray.prototype.sort()`.
    #[inline]
    pub fn sort(&self, compare_fn: Option<JsFunction>, context: &mut Context) -> JsResult<Self> {
        BuiltinTypedArray::sort(
            &self.inner.clone().into(),
            &[compare_fn.into_or_undefined()],
            context,
        )?;

        Ok(self.clone())
    }

    /// Returns a new typed array on the same `ArrayBuffer` store and with the same element
    /// types as for this typed array.
    /// The begin offset is inclusive and the end offset is exclusive.
    ///
    /// Calls `TypedArray.prototype.subarray()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{JsResult, object::{builtins::JsUint8Array}, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::default();
    /// let array = JsUint8Array::from_iter(vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8], context)?;
    /// let subarray2_6 = array.subarray(2, 6, context)?;
    /// assert_eq!(subarray2_6.length(context)?, 4);
    /// assert_eq!(subarray2_6.get(0, context)?, JsValue::new(3.0));
    /// assert_eq!(subarray2_6.get(1, context)?, JsValue::new(4.0));
    /// assert_eq!(subarray2_6.get(2, context)?, JsValue::new(5.0));
    /// assert_eq!(subarray2_6.get(3, context)?, JsValue::new(6.0));
    /// let subarray4_6 = array.subarray(-4, 6, context)?;
    /// assert_eq!(subarray4_6.length(context)?, 2);
    /// assert_eq!(subarray4_6.get(0, context)?, JsValue::new(5.0));
    /// assert_eq!(subarray4_6.get(1, context)?, JsValue::new(6.0));
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn subarray(&self, begin: i64, end: i64, context: &mut Context) -> JsResult<Self> {
        let subarray = BuiltinTypedArray::subarray(
            &self.inner.clone().into(),
            &[begin.into(), end.into()],
            context,
        )?;

        Ok(Self {
            inner: subarray
                .as_object()
                .cloned()
                .expect("`subarray` must always return a `TypedArray` on success"),
        })
    }

    /// Calls `TypedArray.prototype.toLocaleString()`
    #[inline]
    pub fn to_locale_string(
        &self,
        reserved1: Option<JsValue>,
        reserved2: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        BuiltinTypedArray::to_locale_string(
            &self.inner.clone().into(),
            &[reserved1.into_or_undefined(), reserved2.into_or_undefined()],
            context,
        )
    }

    /// Calls `TypedArray.prototype.filter()`.
    #[inline]
    pub fn filter(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let object = BuiltinTypedArray::filter(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: object
                .as_object()
                .cloned()
                .expect("`filter` must always return a `TypedArray` on success"),
        })
    }

    /// Calls `TypedArray.prototype.map()`.
    #[inline]
    pub fn map(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let object = BuiltinTypedArray::map(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: object
                .as_object()
                .cloned()
                .expect("`map` must always return a `TypedArray` on success"),
        })
    }

    /// Calls `TypedArray.prototype.reduce()`.
    #[inline]
    pub fn reduce(
        &self,
        callback: JsFunction,
        initial_value: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        BuiltinTypedArray::reduce(
            &self.inner.clone().into(),
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
        context: &mut Context,
    ) -> JsResult<JsValue> {
        BuiltinTypedArray::reduceright(
            &self.inner.clone().into(),
            &[callback.into(), initial_value.into_or_undefined()],
            context,
        )
    }

    /// Calls `TypedArray.prototype.reverse()`.
    #[inline]
    pub fn reverse(&self, context: &mut Context) -> JsResult<Self> {
        BuiltinTypedArray::reverse(&self.inner.clone().into(), &[], context)?;
        Ok(self.clone())
    }

    /// Stores multiple values in the typed array, reading input values from a specified array.
    ///
    /// Returns `TypedArray.prototype.set()`.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{JsResult, object::{builtins::{JsUint8Array, JsArray, JsArrayBuffer}}, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::default();
    /// let array_buffer8 = JsArrayBuffer::new(8, context)?;
    /// let initialized8_array = JsUint8Array::from_array_buffer(array_buffer8, context)?;
    /// initialized8_array.set_values(
    ///   JsArray::from_iter(vec![JsValue::new(1), JsValue::new(2)], context).into(),
    ///   Some(3),
    ///   context,
    /// )?;
    /// assert_eq!(initialized8_array.get(0, context)?, JsValue::ZERO);
    /// assert_eq!(initialized8_array.get(1, context)?, JsValue::ZERO);
    /// assert_eq!(initialized8_array.get(2, context)?, JsValue::ZERO);
    /// assert_eq!(initialized8_array.get(3, context)?, JsValue::new(1.0));
    /// assert_eq!(initialized8_array.get(4, context)?, JsValue::new(2.0));
    /// assert_eq!(initialized8_array.get(5, context)?, JsValue::ZERO);
    /// assert_eq!(initialized8_array.get(6, context)?, JsValue::ZERO);
    /// assert_eq!(initialized8_array.get(7, context)?, JsValue::ZERO);
    /// assert_eq!(initialized8_array.get(8, context)?, JsValue::UNDEFINED);
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_values(
        &self,
        source: JsValue,
        offset: Option<u64>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        BuiltinTypedArray::set(
            &self.inner.clone().into(),
            &[source, offset.into_or_undefined()],
            context,
        )
    }

    /// Calls `TypedArray.prototype.slice()`.
    #[inline]
    pub fn slice(
        &self,
        start: Option<usize>,
        end: Option<usize>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let object = BuiltinTypedArray::slice(
            &self.inner.clone().into(),
            &[start.into_or_undefined(), end.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: object
                .as_object()
                .cloned()
                .expect("`slice` must always return a `TypedArray` on success"),
        })
    }

    /// Calls `TypedArray.prototype.find()`.
    #[inline]
    pub fn find(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        BuiltinTypedArray::find(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    /// Returns the index of the first element in an array that satisfies the
    /// provided testing function.
    /// If no elements satisfy the testing function, `JsResult::Ok(None)` is returned.
    ///
    /// Calls `TypedArray.prototype.findIndex()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{JsResult, object::{builtins::JsUint8Array, FunctionObjectBuilder}, NativeFunction, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::default();
    /// let data: Vec<u8> = (0..=255).collect();
    /// let array = JsUint8Array::from_iter(data, context)?;
    ///
    /// let greter_than_10_predicate = FunctionObjectBuilder::new(
    ///     context.realm(),
    ///     NativeFunction::from_fn_ptr(|_this, args, _context| {
    ///         let element = args
    ///             .first()
    ///             .cloned()
    ///             .unwrap_or_default()
    ///             .as_number()
    ///             .expect("error at number conversion");
    ///         Ok(JsValue::from(element > 10.0))
    ///     }),
    /// )
    /// .build();
    /// assert_eq!(
    ///     array.find_index(greter_than_10_predicate, None, context),
    ///     Ok(Some(11))
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn find_index(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<Option<u64>> {
        let index = BuiltinTypedArray::find_index(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )?
            .as_number()
            .expect("TypedArray.prototype.findIndex() should always return number");

        if index >= 0.0 {
            Ok(Some(index as u64))
        } else {
            Ok(None)
        }
    }

    /// Iterates the typed array in reverse order and returns the value of
    /// the first element that satisfies the provided testing function.
    /// If no elements satisfy the testing function, `JsResult::Ok(None)` is returned.
    ///
    /// Calls `TypedArray.prototype.findLast()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{JsResult, object::{builtins::JsUint8Array, FunctionObjectBuilder}, NativeFunction, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::default();
    /// let data: Vec<u8> = (0..=255).collect();
    /// let array = JsUint8Array::from_iter(data, context)?;
    ///
    /// let lower_than_200_predicate = FunctionObjectBuilder::new(
    ///     context.realm(),
    ///     NativeFunction::from_fn_ptr(|_this, args, _context| {
    ///         let element = args
    ///             .first()
    ///             .cloned()
    ///             .unwrap_or_default()
    ///             .as_number()
    ///             .expect("error at number conversion");
    ///         Ok(JsValue::from(element < 200.0))
    ///     }),
    /// )
    /// .build();
    /// assert_eq!(
    ///     array.find_last(lower_than_200_predicate.clone(), None, context),
    ///     Ok(JsValue::new(199))
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn find_last(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        BuiltinTypedArray::find_last(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    /// Iterates the typed array in reverse order and returns the index of
    /// the first element that satisfies the provided testing function.
    /// If no elements satisfy the testing function, `JsResult::OK(None)` is returned.
    ///
    /// Calls `TypedArray.prototype.findLastIndex()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{JsResult, object::{builtins::JsUint8Array, FunctionObjectBuilder}, NativeFunction, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::default();
    /// let data: Vec<u8> = (0..=255).collect();
    /// let array = JsUint8Array::from_iter(data, context)?;
    ///
    /// let lower_than_200_predicate = FunctionObjectBuilder::new(
    ///     context.realm(),
    ///     NativeFunction::from_fn_ptr(|_this, args, _context| {
    ///         let element = args
    ///             .first()
    ///             .cloned()
    ///             .unwrap_or_default()
    ///             .as_number()
    ///             .expect("error at number conversion");
    ///         Ok(JsValue::from(element < 200.0))
    ///     }),
    /// )
    /// .build();
    /// assert_eq!(
    ///     array.find_last(lower_than_200_predicate.clone(), None, context),
    ///     Ok(JsValue::Integer(199))
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn find_last_index(
        &self,
        predicate: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<Option<u64>> {
        let index = BuiltinTypedArray::find_last_index(
            &self.inner.clone().into(),
            &[predicate.into(), this_arg.into_or_undefined()],
            context,
        )?
            .as_number()
            .expect("TypedArray.prototype.findLastIndex() should always return number");

        if index >= 0.0 {
            Ok(Some(index as u64))
        } else {
            Ok(None)
        }
    }

    /// Executes a provided function once for each typed array element.
    ///
    /// Calls `TypedArray.prototype.forEach()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_gc::{Gc, GcRefCell};
    /// # use boa_engine::{JsResult, object::{builtins::JsUint8Array, FunctionObjectBuilder}, NativeFunction, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::default();
    /// let array = JsUint8Array::from_iter(vec![1, 2, 3, 4, 5], context)?;
    /// let num_to_modify = Gc::new(GcRefCell::new(0u8));
    ///
    /// let js_function = FunctionObjectBuilder::new(
    ///     context.realm(),
    ///     NativeFunction::from_copy_closure_with_captures(
    ///         |_, args, captures, inner_context| {
    ///             let element = args
    ///                 .first()
    ///                 .cloned()
    ///                 .unwrap_or_default()
    ///                 .to_uint8(inner_context)
    ///                 .expect("error at number conversion");
    ///             *captures.borrow_mut() += element;
    ///             Ok(JsValue::Undefined)
    ///         },
    ///         Gc::clone(&num_to_modify),
    ///     ),
    /// )
    /// .build();
    ///
    /// array.for_each(js_function, None, context);
    /// let borrow = *num_to_modify.borrow();
    /// assert_eq!(borrow, 15u8);
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn for_each(
        &self,
        callback: JsFunction,
        this_arg: Option<JsValue>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        BuiltinTypedArray::for_each(
            &self.inner.clone().into(),
            &[callback.into(), this_arg.into_or_undefined()],
            context,
        )
    }

    /// Determines whether a typed array includes a certain value among its entries,
    /// returning true or false as appropriate.
    ///
    /// Calls `TypedArray.prototype.includes()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{JsResult, object::{builtins::JsUint8Array}, JsValue, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::default();
    /// let data: Vec<u8> = (0..=255).collect();
    /// let array = JsUint8Array::from_iter(data, context)?;
    ///
    /// assert_eq!(array.includes(JsValue::new(2), None, context), Ok(true));
    /// let empty_array = JsUint8Array::from_iter(vec![], context)?;
    /// assert_eq!(
    ///     empty_array.includes(JsValue::new(2), None, context),
    ///     Ok(false)
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn includes<T>(
        &self,
        search_element: T,
        from_index: Option<u64>,
        context: &mut Context,
    ) -> JsResult<bool>
    where
        T: Into<JsValue>,
    {
        let result = BuiltinTypedArray::includes(
            &self.inner.clone().into(),
            &[search_element.into(), from_index.into_or_undefined()],
            context,
        )?
            .as_boolean()
            .expect("TypedArray.prototype.includes should always return boolean");

        Ok(result)
    }

    /// Calls `TypedArray.prototype.indexOf()`.
    pub fn index_of<T>(
        &self,
        search_element: T,
        from_index: Option<usize>,
        context: &mut Context,
    ) -> JsResult<Option<usize>>
    where
        T: Into<JsValue>,
    {
        let index = BuiltinTypedArray::index_of(
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

    /// Calls `TypedArray.prototype.lastIndexOf()`.
    pub fn last_index_of<T>(
        &self,
        search_element: T,
        from_index: Option<usize>,
        context: &mut Context,
    ) -> JsResult<Option<usize>>
    where
        T: Into<JsValue>,
    {
        let index = BuiltinTypedArray::last_index_of(
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

    /// Calls `TypedArray.prototype.join()`.
    #[inline]
    pub fn join(&self, separator: Option<JsString>, context: &mut Context) -> JsResult<JsString> {
        BuiltinTypedArray::join(
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

    /// Calls `TypedArray.prototype.toReversed ( )`.
    #[inline]
    pub fn to_reversed(&self, context: &mut Context) -> JsResult<Self> {
        let array = BuiltinTypedArray::to_reversed(&self.inner.clone().into(), &[], context)?;

        Ok(Self {
            inner: array
                .as_object()
                .cloned()
                .expect("`to_reversed` must always return a `TypedArray` on success"),
        })
    }

    /// Calls `TypedArray.prototype.toSorted ( comparefn )`.
    #[inline]
    pub fn to_sorted(
        &self,
        compare_fn: Option<JsFunction>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let array = BuiltinTypedArray::to_sorted(
            &self.inner.clone().into(),
            &[compare_fn.into_or_undefined()],
            context,
        )?;

        Ok(Self {
            inner: array
                .as_object()
                .cloned()
                .expect("`to_sorted` must always return a `TypedArray` on success"),
        })
    }

    /// Calls `TypedArray.prototype.with ( index, value )`.
    #[inline]
    pub fn with(&self, index: u64, value: JsValue, context: &mut Context) -> JsResult<Self> {
        let array =
            BuiltinTypedArray::with(&self.inner.clone().into(), &[index.into(), value], context)?;

        Ok(Self {
            inner: array
                .as_object()
                .cloned()
                .expect("`with` must always return a `TypedArray` on success"),
        })
    }

    /// It is a getter that returns the same string as the typed array constructor's name.
    /// It returns `Ok(JsValue::Undefined)` if the this value is not one of the typed array subclasses.
    ///
    /// Returns `TypedArray.prototype.toStringTag()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use boa_engine::{JsResult, js_string, object::{builtins::{JsUint8Array}}, Context};
    /// # fn main() -> JsResult<()> {
    ///
    /// let context = &mut Context::default();
    /// let array = JsUint8Array::from_iter(vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8], context)?;
    /// let tag = array.to_string_tag(context)?.to_string(context)?;
    /// assert_eq!(tag, js_string!("Uint8Array"));
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn to_string_tag(&self, context: &mut Context) -> JsResult<JsValue> {
        BuiltinTypedArray::to_string_tag(&self.inner.clone().into(), &[], context)
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

impl TryFromJs for JsTypedArray {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
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
        $value_to_elem:ident,
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
                if object.borrow().$checker_function() {
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
                context: &mut Context,
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
            pub fn from_iter<I>(elements: I, context: &mut Context) -> JsResult<Self>
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

            /// Create an iterator over the typed array's elements.
            pub fn iter<'a>(&'a self, context: &'a mut Context) -> impl Iterator<Item = $element> + 'a {
                let length = self.length(context).unwrap_or(0);
                let mut index = 0;
                std::iter::from_fn(move || {
                    if index < length {
                        let value = self.get(index, context).ok()?;
                        index += 1;
                        value.$value_to_elem(context).ok()
                    } else {
                        None
                    }
                })
            }
        }

        impl From<$name> for JsObject {
            #[inline]
            fn from(o: $name) -> Self {
                o.inner
                    .inner
                    .clone()
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

        impl TryFromJs for $name {
            fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
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
    to_uint8,
    u8
);
JsTypedArrayType!(
    JsInt8Array,
    Int8Array,
    is_typed_int8_array,
    typed_int8_array,
    to_int8,
    i8
);
JsTypedArrayType!(
    JsUint16Array,
    Uint16Array,
    is_typed_uint16_array,
    typed_uint16_array,
    to_uint16,
    u16
);
JsTypedArrayType!(
    JsInt16Array,
    Int16Array,
    is_typed_int16_array,
    typed_int16_array,
    to_int16,
    i16
);
JsTypedArrayType!(
    JsUint32Array,
    Uint32Array,
    is_typed_uint32_array,
    typed_uint32_array,
    to_u32,
    u32
);
JsTypedArrayType!(
    JsInt32Array,
    Int32Array,
    is_typed_int32_array,
    typed_int32_array,
    to_i32,
    i32
);
JsTypedArrayType!(
    JsFloat32Array,
    Float32Array,
    is_typed_float32_array,
    typed_float32_array,
    to_f32,
    f32
);
JsTypedArrayType!(
    JsFloat64Array,
    Float64Array,
    is_typed_float64_array,
    typed_float64_array,
    to_number,
    f64
);

#[test]
fn typed_iterators_uint8() {
    let context = &mut Context::default();
    let vec = vec![1u8, 2, 3, 4, 5, 6, 7, 8];

    let array = JsUint8Array::from_iter(vec.clone(), context).unwrap();
    let vec2 = array.iter(context).collect::<Vec<_>>();
    assert_eq!(vec, vec2);
}

#[test]
fn typed_iterators_uint32() {
    let context = &mut Context::default();
    let vec = vec![1u32, 2, 0xFFFF, 4, 0xFF12_3456, 6, 7, 8];

    let array = JsUint32Array::from_iter(vec.clone(), context).unwrap();
    let vec2 = array.iter(context).collect::<Vec<_>>();
    assert_eq!(vec, vec2);
}

#[test]
fn typed_iterators_f32() {
    let context = &mut Context::default();
    let vec = vec![0.1f32, 0.2, 0.3, 0.4, 1.1, 9.99999];

    let array = JsFloat32Array::from_iter(vec.clone(), context).unwrap();
    let vec2 = array.iter(context).collect::<Vec<_>>();
    assert_eq!(vec, vec2);
}
