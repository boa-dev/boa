use crate::{
    builtins::{array_buffer::SharedMemoryOrder, typed_array::TypedArrayKind, BuiltIn, JsArgs},
    context::intrinsics::StandardConstructors,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        JsObject, ObjectData,
    },
    property::Attribute,
    symbol::WellKnownSymbols,
    value::JsValue,
    Context, JsResult,
};
use boa_gc::{Finalize, Trace};
use tap::{Conv, Pipe};

#[derive(Debug, Clone, Trace, Finalize)]
pub struct DataView {
    viewed_array_buffer: JsObject,
    byte_length: usize,
    byte_offset: usize,
}

impl BuiltIn for DataView {
    const NAME: &'static str = "DataView";

    fn init(context: &mut Context) -> Option<JsValue> {
        let flag_attributes = Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE;

        let get_buffer = FunctionBuilder::native(context, Self::get_buffer)
            .name("get buffer")
            .build();

        let get_byte_length = FunctionBuilder::native(context, Self::get_byte_length)
            .name("get byteLength")
            .build();

        let get_byte_offset = FunctionBuilder::native(context, Self::get_byte_offset)
            .name("get byteOffset")
            .build();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().data_view().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .accessor("buffer", Some(get_buffer), None, flag_attributes)
        .accessor("byteLength", Some(get_byte_length), None, flag_attributes)
        .accessor("byteOffset", Some(get_byte_offset), None, flag_attributes)
        .method(Self::get_big_int64, "getBigInt64", 1)
        .method(Self::get_big_uint64, "getBigUint64", 1)
        .method(Self::get_float32, "getFloat32", 1)
        .method(Self::get_float64, "getFloat64", 1)
        .method(Self::get_int8, "getInt8", 1)
        .method(Self::get_int16, "getInt16", 1)
        .method(Self::get_int32, "getInt32", 1)
        .method(Self::get_uint8, "getUint8", 1)
        .method(Self::get_uint16, "getUint16", 1)
        .method(Self::get_uint32, "getUint32", 1)
        .method(Self::set_big_int64, "setBigInt64", 2)
        .method(Self::set_big_uint64, "setBigUint64", 2)
        .method(Self::set_float32, "setFloat32", 2)
        .method(Self::set_float64, "setFloat64", 2)
        .method(Self::set_int8, "setInt8", 2)
        .method(Self::set_int16, "setInt16", 2)
        .method(Self::set_int32, "setInt32", 2)
        .method(Self::set_uint8, "setUint8", 2)
        .method(Self::set_uint16, "setUint16", 2)
        .method(Self::set_uint32, "setUint32", 2)
        .property(
            WellKnownSymbols::to_string_tag(),
            Self::NAME,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl DataView {
    pub(crate) const LENGTH: usize = 1;

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
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let byte_length = args.get_or_undefined(2);

        let buffer_obj = args
            .get_or_undefined(0)
            .as_object()
            .ok_or_else(|| context.construct_type_error("buffer must be an ArrayBuffer"))?;

        // 1. If NewTarget is undefined, throw a TypeError exception.
        let (offset, view_byte_length) = {
            if new_target.is_undefined() {
                return context.throw_type_error("new target is undefined");
            }
            // 2. Perform ? RequireInternalSlot(buffer, [[ArrayBufferData]]).
            let buffer_borrow = buffer_obj.borrow();
            let buffer = buffer_borrow
                .as_array_buffer()
                .ok_or_else(|| context.construct_type_error("buffer must be an ArrayBuffer"))?;

            // 3. Let offset be ? ToIndex(byteOffset).
            let offset = args.get_or_undefined(1).to_index(context)?;
            // 4. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
            if buffer.is_detached_buffer() {
                return context.throw_type_error("ArrayBuffer is detached");
            }
            // 5. Let bufferByteLength be buffer.[[ArrayBufferByteLength]].
            let buffer_byte_length = buffer.array_buffer_byte_length();
            // 6. If offset > bufferByteLength, throw a RangeError exception.
            if offset > buffer_byte_length {
                return context
                    .throw_range_error("Start offset is outside the bounds of the buffer");
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
                    return context.throw_range_error("Invalid data view length");
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
            .as_array_buffer()
            .ok_or_else(|| context.construct_type_error("buffer must be an ArrayBuffer"))?
            .is_detached_buffer()
        {
            return context.throw_type_error("ArrayBuffer can't be detached");
        }

        let obj = JsObject::from_proto_and_data(
            prototype,
            ObjectData::data_view(Self {
                // 11. Set O.[[ViewedArrayBuffer]] to buffer.
                viewed_array_buffer: buffer_obj.clone(),
                // 12. Set O.[[ByteLength]] to viewByteLength.
                byte_length: view_byte_length,
                // 13. Set O.[[ByteOffset]] to offset.
                byte_offset: offset,
            }),
        );

        // 14. Return O.
        Ok(obj.into())
    }

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
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[DataView]]).
        let dataview = this.as_object().map(JsObject::borrow);
        let dataview = dataview
            .as_ref()
            .and_then(|obj| obj.as_data_view())
            .ok_or_else(|| context.construct_type_error("`this` is not a DataView"))?;
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        let buffer = dataview.viewed_array_buffer.clone();
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
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[DataView]]).
        let dataview = this.as_object().map(JsObject::borrow);
        let dataview = dataview
            .as_ref()
            .and_then(|obj| obj.as_data_view())
            .ok_or_else(|| context.construct_type_error("`this` is not a DataView"))?;
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        let buffer_borrow = dataview.viewed_array_buffer.borrow();
        let borrow = buffer_borrow
            .as_array_buffer()
            .expect("DataView must be constructed with an ArrayBuffer");
        // 5. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        if borrow.is_detached_buffer() {
            return context.throw_type_error("ArrayBuffer is detached");
        }
        // 6. Let size be O.[[ByteLength]].
        let size = dataview.byte_length;
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
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[DataView]]).
        let dataview = this.as_object().map(JsObject::borrow);
        let dataview = dataview
            .as_ref()
            .and_then(|obj| obj.as_data_view())
            .ok_or_else(|| context.construct_type_error("`this` is not a DataView"))?;
        // 3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
        // 4. Let buffer be O.[[ViewedArrayBuffer]].
        let buffer_borrow = dataview.viewed_array_buffer.borrow();
        let borrow = buffer_borrow
            .as_array_buffer()
            .expect("DataView must be constructed with an ArrayBuffer");
        // 5. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        if borrow.is_detached_buffer() {
            return context.throw_type_error("ArrayBuffer is detached");
        }
        // 6. Let offset be O.[[ByteOffset]].
        let offset = dataview.byte_offset;
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
    fn get_view_value(
        view: &JsValue,
        request_index: &JsValue,
        is_little_endian: &JsValue,
        t: TypedArrayKind,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Perform ? RequireInternalSlot(view, [[DataView]]).
        // 2. Assert: view has a [[ViewedArrayBuffer]] internal slot.
        let view = view.as_object().map(JsObject::borrow);
        let view = view
            .as_ref()
            .and_then(|obj| obj.as_data_view())
            .ok_or_else(|| context.construct_type_error("`this` is not a DataView"))?;
        // 3. Let getIndex be ? ToIndex(requestIndex).
        let get_index = request_index.to_index(context)?;

        // 4. Set isLittleEndian to ! ToBoolean(isLittleEndian).
        let is_little_endian = is_little_endian.to_boolean();

        // 5. Let buffer be view.[[ViewedArrayBuffer]].
        let buffer = &view.viewed_array_buffer;
        let buffer_borrow = buffer.borrow();
        let buffer = buffer_borrow
            .as_array_buffer()
            .expect("Should be unreachable");

        // 6. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        if buffer.is_detached_buffer() {
            return context.throw_type_error("ArrayBuffer is detached");
        }
        // 7. Let viewOffset be view.[[ByteOffset]].
        let view_offset = view.byte_offset;

        // 8. Let viewSize be view.[[ByteLength]].
        let view_size = view.byte_length;

        // 9. Let elementSize be the Element Size value specified in Table 72 for Element Type type.
        let element_size = t.element_size();

        // 10. If getIndex + elementSize > viewSize, throw a RangeError exception.
        if get_index + element_size > view_size {
            return context.throw_range_error("Offset is outside the bounds of the DataView");
        }

        // 11. Let bufferIndex be getIndex + viewOffset.
        let buffer_index = get_index + view_offset;

        // 12. Return GetValueFromBuffer(buffer, bufferIndex, type, false, Unordered, isLittleEndian).
        Ok(buffer.get_value_from_buffer(
            buffer_index,
            t,
            false,
            SharedMemoryOrder::Unordered,
            Some(is_little_endian),
        ))
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
        Self::get_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::BigInt64,
            context,
        )
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
        Self::get_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::BigUint64,
            context,
        )
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
        Self::get_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Float32,
            context,
        )
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
        Self::get_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Float64,
            context,
        )
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
        Self::get_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Int8,
            context,
        )
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
        Self::get_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Int16,
            context,
        )
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
        Self::get_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Int32,
            context,
        )
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
        Self::get_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Uint8,
            context,
        )
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
        Self::get_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Uint16,
            context,
        )
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
        Self::get_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Uint32,
            context,
        )
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
    fn set_view_value(
        view: &JsValue,
        request_index: &JsValue,
        is_little_endian: &JsValue,
        t: TypedArrayKind,
        value: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Perform ? RequireInternalSlot(view, [[DataView]]).
        // 2. Assert: view has a [[ViewedArrayBuffer]] internal slot.
        let view = view.as_object().map(JsObject::borrow);
        let view = view
            .as_ref()
            .and_then(|obj| obj.as_data_view())
            .ok_or_else(|| context.construct_type_error("`this` is not a DataView"))?;
        // 3. Let getIndex be ? ToIndex(requestIndex).
        let get_index = request_index.to_index(context)?;

        let number_value = if t.is_big_int_element_type() {
            // 4. If ! IsBigIntElementType(type) is true, let numberValue be ? ToBigInt(value).
            value.to_bigint(context)?.into()
        } else {
            // 5. Otherwise, let numberValue be ? ToNumber(value).
            value.to_number(context)?.into()
        };

        // 6. Set isLittleEndian to ! ToBoolean(isLittleEndian).
        let is_little_endian = is_little_endian.to_boolean();
        // 7. Let buffer be view.[[ViewedArrayBuffer]].
        let buffer = &view.viewed_array_buffer;
        let mut buffer_borrow = buffer.borrow_mut();
        let buffer = buffer_borrow
            .as_array_buffer_mut()
            .expect("Should be unreachable");

        // 8. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
        if buffer.is_detached_buffer() {
            return context.throw_type_error("ArrayBuffer is detached");
        }

        // 9. Let viewOffset be view.[[ByteOffset]].
        let view_offset = view.byte_offset;

        // 10. Let viewSize be view.[[ByteLength]].
        let view_size = view.byte_length;

        // 11. Let elementSize be the Element Size value specified in Table 72 for Element Type type.
        let element_size = t.element_size();

        // 12. If getIndex + elementSize > viewSize, throw a RangeError exception.
        if get_index + element_size > view_size {
            return context.throw_range_error("Offset is outside the bounds of DataView");
        }

        // 13. Let bufferIndex be getIndex + viewOffset.
        let buffer_index = get_index + view_offset;

        // 14. Return SetValueInBuffer(buffer, bufferIndex, type, numberValue, false, Unordered, isLittleEndian).
        buffer.set_value_in_buffer(
            buffer_index,
            t,
            &number_value,
            SharedMemoryOrder::Unordered,
            Some(is_little_endian),
            context,
        )
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
        Self::set_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::BigInt64,
            value,
            context,
        )
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
        Self::set_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::BigUint64,
            value,
            context,
        )
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
        Self::set_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Float32,
            value,
            context,
        )
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
        Self::set_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Float64,
            value,
            context,
        )
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
        Self::set_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Int8,
            value,
            context,
        )
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
        Self::set_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Int16,
            value,
            context,
        )
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
        Self::set_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Int32,
            value,
            context,
        )
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
        Self::set_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Uint8,
            value,
            context,
        )
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
        Self::set_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Uint16,
            value,
            context,
        )
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
        Self::set_view_value(
            this,
            byte_offset,
            is_little_endian,
            TypedArrayKind::Uint32,
            value,
            context,
        )
    }
}
