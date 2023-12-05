//! Boa's implementation of ECMAScript's global `DataView` object.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-dataview-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView

use std::mem;

use crate::{
    builtins::BuiltInObject,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    property::Attribute,
    realm::Realm,
    string::common::StaticJsStrings,
    symbol::JsSymbol,
    value::JsValue,
    Context, JsArgs, JsData, JsResult, JsString,
};
use boa_gc::{Finalize, Trace};
use bytemuck::{bytes_of, bytes_of_mut};

use super::{
    array_buffer::utils::{memcpy, SliceRef, SliceRefMut},
    typed_array::{self, TypedArrayElement},
    BuiltInBuilder, BuiltInConstructor, IntrinsicObject,
};

/// The internal representation of a `DataView` object.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct DataView {
    pub(crate) viewed_array_buffer: JsObject,
    pub(crate) byte_length: u64,
    pub(crate) byte_offset: u64,
}

impl IntrinsicObject for DataView {
    fn init(realm: &Realm) {
        let flag_attributes = Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE;

        let get_buffer = BuiltInBuilder::callable(realm, Self::get_buffer)
            .name(js_string!("get buffer"))
            .build();

        let get_byte_length = BuiltInBuilder::callable(realm, Self::get_byte_length)
            .name(js_string!("get byteLength"))
            .build();

        let get_byte_offset = BuiltInBuilder::callable(realm, Self::get_byte_offset)
            .name(js_string!("get byteOffset"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("buffer"),
                Some(get_buffer),
                None,
                flag_attributes,
            )
            .accessor(
                js_string!("byteLength"),
                Some(get_byte_length),
                None,
                flag_attributes,
            )
            .accessor(
                js_string!("byteOffset"),
                Some(get_byte_offset),
                None,
                flag_attributes,
            )
            .method(Self::get_big_int64, js_string!("getBigInt64"), 1)
            .method(Self::get_big_uint64, js_string!("getBigUint64"), 1)
            .method(Self::get_float32, js_string!("getFloat32"), 1)
            .method(Self::get_float64, js_string!("getFloat64"), 1)
            .method(Self::get_int8, js_string!("getInt8"), 1)
            .method(Self::get_int16, js_string!("getInt16"), 1)
            .method(Self::get_int32, js_string!("getInt32"), 1)
            .method(Self::get_uint8, js_string!("getUint8"), 1)
            .method(Self::get_uint16, js_string!("getUint16"), 1)
            .method(Self::get_uint32, js_string!("getUint32"), 1)
            .method(Self::set_big_int64, js_string!("setBigInt64"), 2)
            .method(Self::set_big_uint64, js_string!("setBigUint64"), 2)
            .method(Self::set_float32, js_string!("setFloat32"), 2)
            .method(Self::set_float64, js_string!("setFloat64"), 2)
            .method(Self::set_int8, js_string!("setInt8"), 2)
            .method(Self::set_int16, js_string!("setInt16"), 2)
            .method(Self::set_int32, js_string!("setInt32"), 2)
            .method(Self::set_uint8, js_string!("setUint8"), 2)
            .method(Self::set_uint16, js_string!("setUint16"), 2)
            .method(Self::set_uint32, js_string!("setUint32"), 2)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DataView {
    const NAME: JsString = StaticJsStrings::DATA_VIEW;
}

impl BuiltInConstructor for DataView {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::data_view;

    /// `25.3.2.1 DataView ( buffer [ , byteOffset [ , byteLength ] ] )`
    ///
    /// The `DataView` view provides a low-level interface for reading and writing multiple number
    /// types in a binary `ArrayBuffer`, without having to care about the platform's endianness.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview-buffer-byteoffset-bytelength
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/DataView
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_length = args.get_or_undefined(2);

        let buffer_obj = args
            .get_or_undefined(0)
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("buffer must be an ArrayBuffer"))?;

        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("new target is undefined")
                .into());
        }

        let (offset, view_byte_length) = {
            // 2. Perform ? RequireInternalSlot(buffer, [[ArrayBufferData]]).
            let buffer_borrow = buffer_obj.borrow();
            let buffer = buffer_borrow.as_buffer().ok_or_else(|| {
                JsNativeError::typ().with_message("buffer must be an ArrayBuffer")
            })?;

            // 3. Let offset be ? ToIndex(byteOffset).
            let offset = args.get_or_undefined(1).to_index(context)?;
            // 4. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
            let Some(buffer) = buffer.data() else {
                return Err(JsNativeError::typ()
                    .with_message("ArrayBuffer is detached")
                    .into());
            };

            // 5. Let bufferByteLength be buffer.[[ArrayBufferByteLength]].
            let buffer_byte_length = buffer.len() as u64;
            // 6. If offset > bufferByteLength, throw a RangeError exception.
            if offset > buffer_byte_length {
                return Err(JsNativeError::range()
                    .with_message("Start offset is outside the bounds of the buffer")
                    .into());
            }
            // 7. If byteLength is undefined, then
            let view_byte_length = if byte_length.is_undefined() {
                // a. Let viewByteLength be bufferByteLength - offset.
                buffer_byte_length - offset
            } else {
                // 8.a. Let viewByteLength be ? ToIndex(byteLength).
                let view_byte_length = byte_length.to_index(context)?;
                // 8.b. If offset + viewByteLength > bufferByteLength, throw a RangeError exception.
                if offset + view_byte_length > buffer_byte_length {
                    return Err(JsNativeError::range()
                        .with_message("Invalid data view length")
                        .into());
                }

                view_byte_length
            };
            (offset, view_byte_length)
        };

        // 9. Let O be ? OrdinaryCreateFromConstructor(NewTarget, "%DataView.prototype%", « [[DataView]], [[ViewedArrayBuffer]], [[ByteLength]], [[ByteOffset]] »).
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::data_view, context)?;

        // 10. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        if buffer_obj
            .borrow()
            .as_buffer()
            .expect("already checked that `buffer_obj` was a buffer")
            .is_detached()
        {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer can't be detached")
                .into());
        }

        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            Self {
                // 11. Set O.[[ViewedArrayBuffer]] to buffer.
                viewed_array_buffer: buffer_obj.clone(),
                // 12. Set O.[[ByteLength]] to viewByteLength.
                byte_length: view_byte_length,
                // 13. Set O.[[ByteOffset]] to offset.
                byte_offset: offset,
            },
        );

        // 14. Return O.
        Ok(obj.into())
    }
}

impl DataView {
    /// `25.3.4.1 get DataView.prototype.buffer`
    ///
    /// The buffer accessor property represents the `ArrayBuffer` or `SharedArrayBuffer` referenced
    /// by the `DataView` at construction time.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-dataview.prototype.buffer
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/buffer
    pub(crate) fn get_buffer(
        this: &JsValue,
        _args: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[DataView]]).
        let view = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a DataView"))?;
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        let buffer = view.viewed_array_buffer.clone();
        // 5. Return buffer.
        Ok(buffer.into())
    }

    /// `25.3.4.1 get DataView.prototype.byteLength`
    ///
    /// The `byteLength` accessor property represents the length (in bytes) of the dataview.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-dataview.prototype.bytelength
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/byteLength
    pub(crate) fn get_byte_length(
        this: &JsValue,
        _args: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[DataView]]).
        let view = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a DataView"))?;
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        let buffer_borrow = view.viewed_array_buffer.borrow();
        let borrow = buffer_borrow
            .as_buffer()
            .expect("DataView must be constructed with a Buffer");
        // 5. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        if borrow.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer is detached")
                .into());
        }
        // 6. Let size be O.[[ByteLength]].
        let size = view.byte_length;
        // 7. Return 𝔽(size).
        Ok(size.into())
    }

    /// `25.3.4.1 get DataView.prototype.byteOffset`
    ///
    /// The `byteOffset` accessor property represents the offset (in bytes) of this view from the
    /// start of its `ArrayBuffer` or `SharedArrayBuffer`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-dataview.prototype.byteoffset
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/byteOffset
    pub(crate) fn get_byte_offset(
        this: &JsValue,
        _args: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[DataView]]).
        let view = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a DataView"))?;
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        let buffer_borrow = view.viewed_array_buffer.borrow();
        let borrow = buffer_borrow
            .as_buffer()
            .expect("DataView must be constructed with a Buffer");
        // 5. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        if borrow.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("Buffer is detached")
                .into());
        }
        // 6. Let offset be O.[[ByteOffset]].
        let offset = view.byte_offset;
        // 7. Return 𝔽(offset).
        Ok(offset.into())
    }

    /// `25.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )`
    ///
    /// The abstract operation `GetViewValue` takes arguments view, requestIndex, `isLittleEndian`,
    /// and type. It is used by functions on `DataView` instances to retrieve values from the
    /// view's buffer.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getviewvalue
    fn get_view_value<T: typed_array::Element>(
        view: &JsValue,
        request_index: &JsValue,
        is_little_endian: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Perform ? RequireInternalSlot(view, [[DataView]]).
        // 2. Assert: view has a [[ViewedArrayBuffer]] internal slot.
        let view = view
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a DataView"))?;
        // 3. Let getIndex be ? ToIndex(requestIndex).
        let get_index = request_index.to_index(context)?;

        // 4. Set isLittleEndian to ! ToBoolean(isLittleEndian).
        let is_little_endian = is_little_endian.to_boolean();

        // 5. Let buffer be view.[[ViewedArrayBuffer]].
        let buffer = &view.viewed_array_buffer;
        let buffer_borrow = buffer.borrow();
        let buffer = buffer_borrow.as_buffer().expect("Should be unreachable");

        // 6. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        let Some(data) = buffer.data() else {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer is detached")
                .into());
        };

        // 7. Let viewOffset be view.[[ByteOffset]].
        let view_offset = view.byte_offset;

        // 8. Let viewSize be view.[[ByteLength]].
        let view_size = view.byte_length;

        // 9. Let elementSize be the Element Size value specified in Table 72 for Element Type type.
        let element_size = mem::size_of::<T>() as u64;

        // 10. If getIndex + elementSize > viewSize, throw a RangeError exception.
        if get_index + element_size > view_size {
            return Err(JsNativeError::range()
                .with_message("Offset is outside the bounds of the DataView")
                .into());
        }

        // 11. Let bufferIndex be getIndex + viewOffset.
        let buffer_index = (get_index + view_offset) as usize;

        // 12. Return GetValueFromBuffer(buffer, bufferIndex, type, false, Unordered, isLittleEndian).
        // SAFETY: All previous checks ensure the element fits in the buffer.
        let value: TypedArrayElement = unsafe {
            let mut value = T::zeroed();
            memcpy(
                data.subslice(buffer_index..),
                SliceRefMut::Slice(bytes_of_mut(&mut value)),
                mem::size_of::<T>(),
            );

            if is_little_endian {
                value.to_little_endian()
            } else {
                value.to_big_endian()
            }
            .into()
        };

        Ok(value.into())
    }

    /// `25.3.4.5 DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )`
    ///
    /// The `getBigInt64()` method gets a signed 64-bit integer (long long) at the specified byte
    /// offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.getbigint64
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getBigInt64
    pub(crate) fn get_big_int64(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let is_little_endian = args.get_or_undefined(1);
        // 1. Let v be the this value.
        // 2. Return ? GetViewValue(v, byteOffset, littleEndian, BigInt64).
        Self::get_view_value::<i64>(this, byte_offset, is_little_endian, context)
    }

    /// `25.3.4.6 DataView.prototype.getBigUint64 ( byteOffset [ , littleEndian ] )`
    ///
    /// The `getBigUint64()` method gets an unsigned 64-bit integer (unsigned long long) at the
    /// specified byte offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.getbiguint64
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getBigUint64
    pub(crate) fn get_big_uint64(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let is_little_endian = args.get_or_undefined(1);
        // 1. Let v be the this value.
        // 2. Return ? GetViewValue(v, byteOffset, littleEndian, BigInt64).
        Self::get_view_value::<u64>(this, byte_offset, is_little_endian, context)
    }

    /// `25.3.4.7 DataView.prototype.getBigUint64 ( byteOffset [ , littleEndian ] )`
    ///
    /// The `getFloat32()` method gets a signed 32-bit float (float) at the specified byte offset
    /// from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.getfloat32
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getFloat32
    pub(crate) fn get_float32(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let is_little_endian = args.get_or_undefined(1);
        // 1. Let v be the this value.
        // 2. Return ? GetViewValue(v, byteOffset, littleEndian, BigInt64).
        Self::get_view_value::<f32>(this, byte_offset, is_little_endian, context)
    }

    /// `25.3.4.8 DataView.prototype.getFloat64 ( byteOffset [ , littleEndian ] )`
    ///
    /// The `getFloat64()` method gets a signed 64-bit float (double) at the specified byte offset
    /// from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.getfloat64
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getFloat64
    pub(crate) fn get_float64(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let is_little_endian = args.get_or_undefined(1);
        // 1. Let v be the this value.
        // 2. Return ? GetViewValue(v, byteOffset, littleEndian, BigInt64).
        Self::get_view_value::<f64>(this, byte_offset, is_little_endian, context)
    }

    /// `25.3.4.9 DataView.prototype.getInt8 ( byteOffset [ , littleEndian ] )`
    ///
    /// The `getInt8()` method gets a signed 8-bit integer (byte) at the specified byte offset
    /// from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.getint8
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getInt8
    pub(crate) fn get_int8(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let is_little_endian = args.get_or_undefined(1);
        // 1. Let v be the this value.
        // 2. Return ? GetViewValue(v, byteOffset, littleEndian, BigInt64).
        Self::get_view_value::<i8>(this, byte_offset, is_little_endian, context)
    }

    /// `25.3.4.10 DataView.prototype.getInt16 ( byteOffset [ , littleEndian ] )`
    ///
    /// The `getInt16()` method gets a signed 16-bit integer (short) at the specified byte offset
    /// from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.getint16
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getInt16
    pub(crate) fn get_int16(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let is_little_endian = args.get_or_undefined(1);
        // 1. Let v be the this value.
        // 2. Return ? GetViewValue(v, byteOffset, littleEndian, BigInt64).
        Self::get_view_value::<i16>(this, byte_offset, is_little_endian, context)
    }

    /// `25.3.4.11 DataView.prototype.getInt32 ( byteOffset [ , littleEndian ] )`
    ///
    /// The `getInt32()` method gets a signed 32-bit integer (long) at the specified byte offset
    /// from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.getint32
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getInt32
    pub(crate) fn get_int32(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let is_little_endian = args.get_or_undefined(1);
        // 1. Let v be the this value.
        // 2. Return ? GetViewValue(v, byteOffset, littleEndian, BigInt64).
        Self::get_view_value::<i32>(this, byte_offset, is_little_endian, context)
    }

    /// `25.3.4.12 DataView.prototype.getUint8 ( byteOffset [ , littleEndian ] )`
    ///
    /// The `getUint8()` method gets an unsigned 8-bit integer (unsigned byte) at the specified
    /// byte offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.getuint8
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getUint8
    pub(crate) fn get_uint8(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let is_little_endian = args.get_or_undefined(1);
        // 1. Let v be the this value.
        // 2. Return ? GetViewValue(v, byteOffset, littleEndian, BigInt64).
        Self::get_view_value::<u8>(this, byte_offset, is_little_endian, context)
    }

    /// `25.3.4.13 DataView.prototype.getUint16 ( byteOffset [ , littleEndian ] )`
    ///
    /// The `getUint16()` method gets an unsigned 16-bit integer (unsigned short) at the specified
    /// byte offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.getuint16
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getUint16
    pub(crate) fn get_uint16(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let is_little_endian = args.get_or_undefined(1);
        // 1. Let v be the this value.
        // 2. Return ? GetViewValue(v, byteOffset, littleEndian, BigInt64).
        Self::get_view_value::<u16>(this, byte_offset, is_little_endian, context)
    }

    /// `25.3.4.14 DataView.prototype.getUint32 ( byteOffset [ , littleEndian ] )`
    ///
    /// The `getUint32()` method gets an unsigned 32-bit integer (unsigned long) at the specified
    /// byte offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.getuint32
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/getUint32
    pub(crate) fn get_uint32(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let is_little_endian = args.get_or_undefined(1);
        // 1. Let v be the this value.
        // 2. Return ? GetViewValue(v, byteOffset, littleEndian, BigInt64).
        Self::get_view_value::<u32>(this, byte_offset, is_little_endian, context)
    }

    /// `25.3.1.1 SetViewValue ( view, requestIndex, isLittleEndian, type )`
    ///
    /// The abstract operation `SetViewValue` takes arguments view, requestIndex, `isLittleEndian`,
    /// type, and value. It is used by functions on `DataView` instances to store values into the
    /// view's buffer.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-setviewvalue
    fn set_view_value<T: typed_array::Element>(
        view: &JsValue,
        request_index: &JsValue,
        is_little_endian: &JsValue,
        value: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Perform ? RequireInternalSlot(view, [[DataView]]).
        // 2. Assert: view has a [[ViewedArrayBuffer]] internal slot.
        let view = view
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| JsNativeError::typ().with_message("`this` is not a DataView"))?;
        // 3. Let getIndex be ? ToIndex(requestIndex).
        let get_index = request_index.to_index(context)?;

        // 4. If ! IsBigIntElementType(type) is true, let numberValue be ? ToBigInt(value).
        // 5. Otherwise, let numberValue be ? ToNumber(value).
        let value = T::from_js_value(value, context)?;

        // 6. Set isLittleEndian to ! ToBoolean(isLittleEndian).
        let is_little_endian = is_little_endian.to_boolean();
        // 7. Let buffer be view.[[ViewedArrayBuffer]].
        let buffer = &view.viewed_array_buffer;
        let mut buffer_borrow = buffer.borrow_mut();
        let mut buffer = buffer_borrow
            .as_buffer_mut()
            .expect("Should be unreachable");

        // 8. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        let Some(mut data) = buffer.data_mut() else {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer is detached")
                .into());
        };

        // 9. Let viewOffset be view.[[ByteOffset]].
        let view_offset = view.byte_offset;

        // 10. Let viewSize be view.[[ByteLength]].
        let view_size = view.byte_length;

        // 11. Let elementSize be the Element Size value specified in Table 72 for Element Type type.
        // 12. If getIndex + elementSize > viewSize, throw a RangeError exception.
        if get_index + mem::size_of::<T>() as u64 > view_size {
            return Err(JsNativeError::range()
                .with_message("Offset is outside the bounds of DataView")
                .into());
        }

        // 13. Let bufferIndex be getIndex + viewOffset.
        let buffer_index = (get_index + view_offset) as usize;

        // 14. Return SetValueInBuffer(buffer, bufferIndex, type, numberValue, false, Unordered, isLittleEndian).

        // SAFETY: All previous checks ensure the element fits in the buffer.
        unsafe {
            let value = if is_little_endian {
                value.to_little_endian()
            } else {
                value.to_big_endian()
            };

            memcpy(
                SliceRef::Slice(bytes_of(&value)),
                data.subslice_mut(buffer_index..),
                mem::size_of::<T>(),
            );
        }

        Ok(JsValue::undefined())
    }

    /// `25.3.4.15 DataView.prototype.setBigInt64 ( byteOffset, value [ , littleEndian ] )`
    ///
    /// The `setBigInt64()` method stores a signed 64-bit integer (long long) value at the
    /// specified byte offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.setbigint64
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setBigInt64
    pub(crate) fn set_big_int64(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);
        let is_little_endian = args.get_or_undefined(2);
        // 1. Let v be the this value.
        // 2. Return ? SetViewValue(v, byteOffset, littleEndian, BigUint64, value).
        Self::set_view_value::<i64>(this, byte_offset, is_little_endian, value, context)
    }

    /// `25.3.4.16 DataView.prototype.setBigUint64 ( byteOffset, value [ , littleEndian ] )`
    ///
    /// The `setBigUint64()` method stores an unsigned 64-bit integer (unsigned long long) value at
    /// the specified byte offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.setbiguint64
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setBigUint64
    pub(crate) fn set_big_uint64(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);
        let is_little_endian = args.get_or_undefined(2);
        // 1. Let v be the this value.
        // 2. Return ? SetViewValue(v, byteOffset, littleEndian, BigUint64, value).
        Self::set_view_value::<u64>(this, byte_offset, is_little_endian, value, context)
    }

    /// `25.3.4.17 DataView.prototype.setFloat32 ( byteOffset, value [ , littleEndian ] )`
    ///
    /// The `setFloat32()` method stores a signed 32-bit float (float) value at the specified byte
    /// offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.setfloat32
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setFloat32
    pub(crate) fn set_float32(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);
        let is_little_endian = args.get_or_undefined(2);
        // 1. Let v be the this value.
        // 2. Return ? SetViewValue(v, byteOffset, littleEndian, Float32, value).
        Self::set_view_value::<f32>(this, byte_offset, is_little_endian, value, context)
    }

    /// `25.3.4.18 DataView.prototype.setFloat64 ( byteOffset, value [ , littleEndian ] )`
    ///
    /// The `setFloat64()` method stores a signed 64-bit float (double) value at the specified byte
    /// offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.setfloat64
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setFloat64
    pub(crate) fn set_float64(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);
        let is_little_endian = args.get_or_undefined(2);
        // 1. Let v be the this value.
        // 2. Return ? SetViewValue(v, byteOffset, littleEndian, Float64, value).
        Self::set_view_value::<f64>(this, byte_offset, is_little_endian, value, context)
    }

    /// `25.3.4.19 DataView.prototype.setInt8 ( byteOffset, value [ , littleEndian ] )`
    ///
    /// The `setInt8()` method stores a signed 8-bit integer (byte) value at the specified byte
    /// offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.setint8
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setInt8
    pub(crate) fn set_int8(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);
        let is_little_endian = args.get_or_undefined(2);
        // 1. Let v be the this value.
        // 2. Return ? SetViewValue(v, byteOffset, littleEndian, Int8, value).
        Self::set_view_value::<i8>(this, byte_offset, is_little_endian, value, context)
    }

    /// `25.3.4.20 DataView.prototype.setInt16 ( byteOffset, value [ , littleEndian ] )`
    ///
    /// The `setInt16()` method stores a signed 16-bit integer (short) value at the specified byte
    /// offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.setint16
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setInt16
    pub(crate) fn set_int16(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);
        let is_little_endian = args.get_or_undefined(2);
        // 1. Let v be the this value.
        // 2. Return ? SetViewValue(v, byteOffset, littleEndian, Int16, value).
        Self::set_view_value::<i16>(this, byte_offset, is_little_endian, value, context)
    }

    /// `25.3.4.21 DataView.prototype.setInt32 ( byteOffset, value [ , littleEndian ] )`
    ///
    /// The `setInt32()` method stores a signed 32-bit integer (long) value at the specified byte
    /// offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.setint32
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setInt32
    pub(crate) fn set_int32(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);
        let is_little_endian = args.get_or_undefined(2);
        // 1. Let v be the this value.
        // 2. Return ? SetViewValue(v, byteOffset, littleEndian, Int32, value).
        Self::set_view_value::<i32>(this, byte_offset, is_little_endian, value, context)
    }

    /// `25.3.4.22 DataView.prototype.setUint8 ( byteOffset, value [ , littleEndian ] )`
    ///
    /// The `setUint8()` method stores an unsigned 8-bit integer (byte) value at the specified byte
    /// offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.setuint8
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setUint8
    pub(crate) fn set_uint8(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);
        let is_little_endian = args.get_or_undefined(2);
        // 1. Let v be the this value.
        // 2. Return ? SetViewValue(v, byteOffset, littleEndian, Uint8, value).
        Self::set_view_value::<u8>(this, byte_offset, is_little_endian, value, context)
    }

    /// `25.3.4.23 DataView.prototype.setUint16 ( byteOffset, value [ , littleEndian ] )`
    ///
    /// The `setUint16()` method stores an unsigned 16-bit integer (unsigned short) value at the
    /// specified byte offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.setuint16
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setUint16
    pub(crate) fn set_uint16(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);
        let is_little_endian = args.get_or_undefined(2);
        // 1. Let v be the this value.
        // 2. Return ? SetViewValue(v, byteOffset, littleEndian, Uint16, value).
        Self::set_view_value::<u16>(this, byte_offset, is_little_endian, value, context)
    }

    /// `25.3.4.24 DataView.prototype.setUint32 ( byteOffset, value [ , littleEndian ] )`
    ///
    /// The `setUint32()` method stores an unsigned 32-bit integer (unsigned long) value at the
    /// specified byte offset from the start of the `DataView`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-dataview.prototype.setuint32
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView/setUint32
    pub(crate) fn set_uint32(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_offset = args.get_or_undefined(0);
        let value = args.get_or_undefined(1);
        let is_little_endian = args.get_or_undefined(2);
        // 1. Let v be the this value.
        // 2. Return ? SetViewValue(v, byteOffset, littleEndian, Uint32, value).
        Self::set_view_value::<u32>(this, byte_offset, is_little_endian, value, context)
    }
}
