#[cfg(test)]
mod tests;

use crate::{
    builtins::{typed_array::TypedArrayKind, BuiltIn, JsArgs},
    context::intrinsics::StandardConstructors,
    error::JsNativeError,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        JsObject, ObjectData,
    },
    property::Attribute,
    symbol::WellKnownSymbols,
    value::{IntegerOrInfinity, Numeric},
    Context, JsResult, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;
use num_traits::{Signed, ToPrimitive};
use tap::{Conv, Pipe};

#[derive(Debug, Clone, Trace, Finalize)]
pub struct ArrayBuffer {
    pub array_buffer_data: Option<Vec<u8>>,
    pub array_buffer_byte_length: u64,
    pub array_buffer_detach_key: JsValue,
}

impl ArrayBuffer {
    pub(crate) fn array_buffer_byte_length(&self) -> u64 {
        self.array_buffer_byte_length
    }
}

impl BuiltIn for ArrayBuffer {
    const NAME: &'static str = "ArrayBuffer";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        let flag_attributes = Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE;

        let get_species = FunctionBuilder::native(context, Self::get_species)
            .name("get [Symbol.species]")
            .constructor(false)
            .build();

        let get_byte_length = FunctionBuilder::native(context, Self::get_byte_length)
            .name("get byteLength")
            .build();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().array_buffer().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .accessor("byteLength", Some(get_byte_length), None, flag_attributes)
        .static_accessor(
            WellKnownSymbols::species(),
            Some(get_species),
            None,
            Attribute::CONFIGURABLE,
        )
        .static_method(Self::is_view, "isView", 1)
        .method(Self::slice, "slice", 2)
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

impl ArrayBuffer {
    const LENGTH: usize = 1;

    /// `25.1.3.1 ArrayBuffer ( length )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer-length
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is undefined, throw a TypeError exception.
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer.constructor called with undefined new target")
                .into());
        }

        // 2. Let byteLength be ? ToIndex(length).
        let byte_length = args.get_or_undefined(0).to_index(context)?;

        // 3. Return ? AllocateArrayBuffer(NewTarget, byteLength).
        Ok(Self::allocate(new_target, byte_length, context)?.into())
    }

    /// `25.1.4.3 get ArrayBuffer [ @@species ]`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-arraybuffer-@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `25.1.4.1 ArrayBuffer.isView ( arg )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer.isview
    #[allow(clippy::unnecessary_wraps)]
    fn is_view(_: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // 1. If Type(arg) is not Object, return false.
        // 2. If arg has a [[ViewedArrayBuffer]] internal slot, return true.
        // 3. Return false.
        Ok(args
            .get_or_undefined(0)
            .as_object()
            .map(|obj| obj.borrow().has_viewed_array_buffer())
            .unwrap_or_default()
            .into())
    }

    /// `25.1.5.1 get ArrayBuffer.prototype.byteLength`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-arraybuffer.prototype.bytelength
    pub(crate) fn get_byte_length(
        this: &JsValue,
        _args: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ArrayBuffer.byteLength called with non-object value")
        })?;
        let obj = obj.borrow();
        let buf = obj.as_array_buffer().ok_or_else(|| {
            JsNativeError::typ().with_message("ArrayBuffer.byteLength called with invalid object")
        })?;

        // TODO: Shared Array Buffer
        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.

        // 4. If IsDetachedBuffer(O) is true, return +0𝔽.
        if Self::is_detached_buffer(buf) {
            return Ok(0.into());
        }

        // 5. Let length be O.[[ArrayBufferByteLength]].
        // 6. Return 𝔽(length).
        Ok(buf.array_buffer_byte_length.into())
    }

    /// `25.1.5.3 ArrayBuffer.prototype.slice ( start, end )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer.prototype.slice
    fn slice(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ArrayBuffer.slice called with non-object value")
        })?;
        let obj_borrow = obj.borrow();
        let buf = obj_borrow.as_array_buffer().ok_or_else(|| {
            JsNativeError::typ().with_message("ArrayBuffer.slice called with invalid object")
        })?;

        // TODO: Shared Array Buffer
        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.

        // 4. If IsDetachedBuffer(O) is true, throw a TypeError exception.
        if Self::is_detached_buffer(buf) {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer.slice called with detached buffer")
                .into());
        }

        // 5. Let len be O.[[ArrayBufferByteLength]].
        let len = buf.array_buffer_byte_length as i64;

        // 6. Let relativeStart be ? ToIntegerOrInfinity(start).
        let relative_start = args.get_or_undefined(0).to_integer_or_infinity(context)?;

        let first = match relative_start {
            // 7. If relativeStart is -∞, let first be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 8. Else if relativeStart < 0, let first be max(len + relativeStart, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(len + i, 0),
            // 9. Else, let first be min(relativeStart, len).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 10. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
        let end = args.get_or_undefined(1);
        let relative_end = if end.is_undefined() {
            IntegerOrInfinity::Integer(len)
        } else {
            end.to_integer_or_infinity(context)?
        };

        let r#final = match relative_end {
            // 11. If relativeEnd is -∞, let final be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 12. Else if relativeEnd < 0, let final be max(len + relativeEnd, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => std::cmp::max(len + i, 0),
            // 13. Else, let final be min(relativeEnd, len).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i, len),
            IntegerOrInfinity::PositiveInfinity => len,
        };

        // 14. Let newLen be max(final - first, 0).
        let new_len = std::cmp::max(r#final - first, 0) as u64;

        // 15. Let ctor be ? SpeciesConstructor(O, %ArrayBuffer%).
        let ctor = obj.species_constructor(StandardConstructors::array_buffer, context)?;

        // 16. Let new be ? Construct(ctor, « 𝔽(newLen) »).
        let new = ctor.construct(&[new_len.into()], Some(&ctor), context)?;

        {
            let new_obj = new.borrow();
            // 17. Perform ? RequireInternalSlot(new, [[ArrayBufferData]]).
            let new_array_buffer = new_obj.as_array_buffer().ok_or_else(|| {
                JsNativeError::typ().with_message("ArrayBuffer constructor returned invalid object")
            })?;

            // TODO: Shared Array Buffer
            // 18. If IsSharedArrayBuffer(new) is true, throw a TypeError exception.

            // 19. If IsDetachedBuffer(new) is true, throw a TypeError exception.
            if new_array_buffer.is_detached_buffer() {
                return Err(JsNativeError::typ()
                    .with_message("ArrayBuffer constructor returned detached ArrayBuffer")
                    .into());
            }
        }
        // 20. If SameValue(new, O) is true, throw a TypeError exception.
        if this
            .as_object()
            .map(|obj| JsObject::equals(obj, &new))
            .unwrap_or_default()
        {
            return Err(JsNativeError::typ()
                .with_message("New ArrayBuffer is the same as this ArrayBuffer")
                .into());
        }

        {
            let mut new_obj_borrow = new.borrow_mut();
            let new_array_buffer = new_obj_borrow
                .as_array_buffer_mut()
                .expect("Already checked that `new_obj` was an `ArrayBuffer`");

            // 21. If new.[[ArrayBufferByteLength]] < newLen, throw a TypeError exception.
            if new_array_buffer.array_buffer_byte_length < new_len {
                return Err(JsNativeError::typ()
                    .with_message("New ArrayBuffer length too small")
                    .into());
            }

            // 22. NOTE: Side-effects of the above steps may have detached O.
            // 23. If IsDetachedBuffer(O) is true, throw a TypeError exception.
            if Self::is_detached_buffer(buf) {
                return Err(JsNativeError::typ()
                    .with_message("ArrayBuffer detached while ArrayBuffer.slice was running")
                    .into());
            }

            // 24. Let fromBuf be O.[[ArrayBufferData]].
            let from_buf = buf
                .array_buffer_data
                .as_ref()
                .expect("ArrayBuffer cannot be detached here");

            // 25. Let toBuf be new.[[ArrayBufferData]].
            let to_buf = new_array_buffer
                .array_buffer_data
                .as_mut()
                .expect("ArrayBuffer cannot be detached here");

            // 26. Perform CopyDataBlockBytes(toBuf, 0, fromBuf, first, newLen).
            copy_data_block_bytes(to_buf, 0, from_buf, first as usize, new_len as usize);
        }

        // 27. Return new.
        Ok(new.into())
    }

    /// `25.1.2.1 AllocateArrayBuffer ( constructor, byteLength )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-allocatearraybuffer
    pub(crate) fn allocate(
        constructor: &JsValue,
        byte_length: u64,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%ArrayBuffer.prototype%", « [[ArrayBufferData]], [[ArrayBufferByteLength]], [[ArrayBufferDetachKey]] »).
        let prototype = get_prototype_from_constructor(
            constructor,
            StandardConstructors::array_buffer,
            context,
        )?;
        let obj = context.construct_object();
        obj.set_prototype(prototype.into());

        // 2. Let block be ? CreateByteDataBlock(byteLength).
        let block = create_byte_data_block(byte_length)?;

        // 3. Set obj.[[ArrayBufferData]] to block.
        // 4. Set obj.[[ArrayBufferByteLength]] to byteLength.
        obj.borrow_mut().data = ObjectData::array_buffer(Self {
            array_buffer_data: Some(block),
            array_buffer_byte_length: byte_length,
            array_buffer_detach_key: JsValue::Undefined,
        });

        // 5. Return obj.
        Ok(obj)
    }

    /// `25.1.2.2 IsDetachedBuffer ( arrayBuffer )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isdetachedbuffer
    pub(crate) fn is_detached_buffer(&self) -> bool {
        // 1. If arrayBuffer.[[ArrayBufferData]] is null, return true.
        // 2. Return false.
        self.array_buffer_data.is_none()
    }

    /// `25.1.2.4 CloneArrayBuffer ( srcBuffer, srcByteOffset, srcLength, cloneConstructor )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-clonearraybuffer
    pub(crate) fn clone_array_buffer(
        &self,
        src_byte_offset: u64,
        src_length: u64,
        clone_constructor: &JsValue,
        context: &mut Context,
    ) -> JsResult<JsObject> {
        // 1. Let targetBuffer be ? AllocateArrayBuffer(cloneConstructor, srcLength).
        let target_buffer = Self::allocate(clone_constructor, src_length, context)?;

        // 2. If IsDetachedBuffer(srcBuffer) is true, throw a TypeError exception.
        // 3. Let srcBlock be srcBuffer.[[ArrayBufferData]].
        let src_block = self.array_buffer_data.as_deref().ok_or_else(|| {
            JsNativeError::syntax().with_message("Cannot clone detached array buffer")
        })?;

        {
            // 4. Let targetBlock be targetBuffer.[[ArrayBufferData]].
            let mut target_buffer_mut = target_buffer.borrow_mut();
            let target_block = target_buffer_mut
                .as_array_buffer_mut()
                .expect("This must be an ArrayBuffer");

            // 5. Perform CopyDataBlockBytes(targetBlock, 0, srcBlock, srcByteOffset, srcLength).
            copy_data_block_bytes(
                target_block
                    .array_buffer_data
                    .as_mut()
                    .expect("ArrayBuffer cannot me detached here"),
                0,
                src_block,
                src_byte_offset as usize,
                src_length as usize,
            );
        }

        // 6. Return targetBuffer.
        Ok(target_buffer)
    }

    /// `25.1.2.6 IsUnclampedIntegerElementType ( type )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isunclampedintegerelementtype
    fn is_unclamped_integer_element_type(t: TypedArrayKind) -> bool {
        // 1. If type is Int8, Uint8, Int16, Uint16, Int32, or Uint32, return true.
        // 2. Return false.
        matches!(
            t,
            TypedArrayKind::Int8
                | TypedArrayKind::Uint8
                | TypedArrayKind::Int16
                | TypedArrayKind::Uint16
                | TypedArrayKind::Int32
                | TypedArrayKind::Uint32
        )
    }

    /// `25.1.2.7 IsBigIntElementType ( type )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isbigintelementtype
    fn is_big_int_element_type(t: TypedArrayKind) -> bool {
        // 1. If type is BigUint64 or BigInt64, return true.
        // 2. Return false.
        matches!(t, TypedArrayKind::BigUint64 | TypedArrayKind::BigInt64)
    }

    /// `25.1.2.8 IsNoTearConfiguration ( type, order )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isnotearconfiguration
    // TODO: Allow unused function until shared array buffers are implemented.
    #[allow(dead_code)]
    fn is_no_tear_configuration(t: TypedArrayKind, order: SharedMemoryOrder) -> bool {
        // 1. If ! IsUnclampedIntegerElementType(type) is true, return true.
        if Self::is_unclamped_integer_element_type(t) {
            return true;
        }

        // 2. If ! IsBigIntElementType(type) is true and order is not Init or Unordered, return true.
        if Self::is_big_int_element_type(t)
            && !matches!(
                order,
                SharedMemoryOrder::Init | SharedMemoryOrder::Unordered
            )
        {
            return true;
        }

        // 3. Return false.
        false
    }

    /// `25.1.2.9 RawBytesToNumeric ( type, rawBytes, isLittleEndian )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-rawbytestonumeric
    fn raw_bytes_to_numeric(t: TypedArrayKind, bytes: &[u8], is_little_endian: bool) -> JsValue {
        let n: Numeric = match t {
            TypedArrayKind::Int8 => {
                if is_little_endian {
                    i8::from_le_bytes(bytes.try_into().expect("slice with incorrect length")).into()
                } else {
                    i8::from_be_bytes(bytes.try_into().expect("slice with incorrect length")).into()
                }
            }
            TypedArrayKind::Uint8 | TypedArrayKind::Uint8Clamped => {
                if is_little_endian {
                    u8::from_le_bytes(bytes.try_into().expect("slice with incorrect length")).into()
                } else {
                    u8::from_be_bytes(bytes.try_into().expect("slice with incorrect length")).into()
                }
            }
            TypedArrayKind::Int16 => {
                if is_little_endian {
                    i16::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                } else {
                    i16::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                }
            }
            TypedArrayKind::Uint16 => {
                if is_little_endian {
                    u16::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                } else {
                    u16::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                }
            }
            TypedArrayKind::Int32 => {
                if is_little_endian {
                    i32::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                } else {
                    i32::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                }
            }
            TypedArrayKind::Uint32 => {
                if is_little_endian {
                    u32::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                } else {
                    u32::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                }
            }
            TypedArrayKind::BigInt64 => {
                if is_little_endian {
                    i64::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                } else {
                    i64::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                }
            }
            TypedArrayKind::BigUint64 => {
                if is_little_endian {
                    u64::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                } else {
                    u64::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                }
            }
            TypedArrayKind::Float32 => {
                if is_little_endian {
                    f32::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                } else {
                    f32::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                }
            }
            TypedArrayKind::Float64 => {
                if is_little_endian {
                    f64::from_le_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                } else {
                    f64::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
                        .into()
                }
            }
        };

        n.into()
    }

    /// `25.1.2.10 GetValueFromBuffer ( arrayBuffer, byteIndex, type, isTypedArray, order [ , isLittleEndian ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getvaluefrombuffer
    pub(crate) fn get_value_from_buffer(
        &self,
        byte_index: u64,
        t: TypedArrayKind,
        _is_typed_array: bool,
        _order: SharedMemoryOrder,
        is_little_endian: Option<bool>,
    ) -> JsValue {
        // 1. Assert: IsDetachedBuffer(arrayBuffer) is false.
        // 2. Assert: There are sufficient bytes in arrayBuffer starting at byteIndex to represent a value of type.
        // 3. Let block be arrayBuffer.[[ArrayBufferData]].
        let block = self
            .array_buffer_data
            .as_ref()
            .expect("ArrayBuffer cannot be detached here");

        // 4. Let elementSize be the Element Size value specified in Table 73 for Element Type type.
        let element_size = t.element_size() as usize;

        // TODO: Shared Array Buffer
        // 5. If IsSharedArrayBuffer(arrayBuffer) is true, then

        // 6. Else, let rawValue be a List whose elements are bytes from block at indices byteIndex (inclusive) through byteIndex + elementSize (exclusive).
        // 7. Assert: The number of elements in rawValue is elementSize.
        let byte_index = byte_index as usize;
        let raw_value = &block[byte_index..byte_index + element_size];

        // TODO: Agent Record [[LittleEndian]] filed
        // 8. If isLittleEndian is not present, set isLittleEndian to the value of the [[LittleEndian]] field of the surrounding agent's Agent Record.
        let is_little_endian = is_little_endian.unwrap_or(true);

        // 9. Return RawBytesToNumeric(type, rawValue, isLittleEndian).
        Self::raw_bytes_to_numeric(t, raw_value, is_little_endian)
    }

    /// `25.1.2.11 NumericToRawBytes ( type, value, isLittleEndian )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-numerictorawbytes
    fn numeric_to_raw_bytes(
        t: TypedArrayKind,
        value: &JsValue,
        is_little_endian: bool,
        context: &mut Context,
    ) -> JsResult<Vec<u8>> {
        Ok(match t {
            TypedArrayKind::Int8 if is_little_endian => {
                value.to_int8(context)?.to_le_bytes().to_vec()
            }
            TypedArrayKind::Int8 => value.to_int8(context)?.to_be_bytes().to_vec(),
            TypedArrayKind::Uint8 if is_little_endian => {
                value.to_uint8(context)?.to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint8 => value.to_uint8(context)?.to_be_bytes().to_vec(),
            TypedArrayKind::Uint8Clamped if is_little_endian => {
                value.to_uint8_clamp(context)?.to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint8Clamped => value.to_uint8_clamp(context)?.to_be_bytes().to_vec(),
            TypedArrayKind::Int16 if is_little_endian => {
                value.to_int16(context)?.to_le_bytes().to_vec()
            }
            TypedArrayKind::Int16 => value.to_int16(context)?.to_be_bytes().to_vec(),
            TypedArrayKind::Uint16 if is_little_endian => {
                value.to_uint16(context)?.to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint16 => value.to_uint16(context)?.to_be_bytes().to_vec(),
            TypedArrayKind::Int32 if is_little_endian => {
                value.to_i32(context)?.to_le_bytes().to_vec()
            }
            TypedArrayKind::Int32 => value.to_i32(context)?.to_be_bytes().to_vec(),
            TypedArrayKind::Uint32 if is_little_endian => {
                value.to_u32(context)?.to_le_bytes().to_vec()
            }
            TypedArrayKind::Uint32 => value.to_u32(context)?.to_be_bytes().to_vec(),
            TypedArrayKind::BigInt64 if is_little_endian => {
                let big_int = value.to_big_int64(context)?;
                big_int
                    .to_i64()
                    .unwrap_or_else(|| {
                        if big_int.is_positive() {
                            i64::MAX
                        } else {
                            i64::MIN
                        }
                    })
                    .to_le_bytes()
                    .to_vec()
            }
            TypedArrayKind::BigInt64 => {
                let big_int = value.to_big_int64(context)?;
                big_int
                    .to_i64()
                    .unwrap_or_else(|| {
                        if big_int.is_positive() {
                            i64::MAX
                        } else {
                            i64::MIN
                        }
                    })
                    .to_be_bytes()
                    .to_vec()
            }
            TypedArrayKind::BigUint64 if is_little_endian => value
                .to_big_uint64(context)?
                .to_u64()
                .unwrap_or(u64::MAX)
                .to_le_bytes()
                .to_vec(),
            TypedArrayKind::BigUint64 => value
                .to_big_uint64(context)?
                .to_u64()
                .unwrap_or(u64::MAX)
                .to_be_bytes()
                .to_vec(),
            TypedArrayKind::Float32 => match value.to_number(context)? {
                f if is_little_endian => (f as f32).to_le_bytes().to_vec(),
                f => (f as f32).to_be_bytes().to_vec(),
            },
            TypedArrayKind::Float64 => match value.to_number(context)? {
                f if is_little_endian => f.to_le_bytes().to_vec(),
                f => f.to_be_bytes().to_vec(),
            },
        })
    }

    /// `25.1.2.12 SetValueInBuffer ( arrayBuffer, byteIndex, type, value, isTypedArray, order [ , isLittleEndian ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-setvalueinbuffer
    pub(crate) fn set_value_in_buffer(
        &mut self,
        byte_index: u64,
        t: TypedArrayKind,
        value: &JsValue,
        _order: SharedMemoryOrder,
        is_little_endian: Option<bool>,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Assert: IsDetachedBuffer(arrayBuffer) is false.
        // 2. Assert: There are sufficient bytes in arrayBuffer starting at byteIndex to represent a value of type.
        // 3. Assert: Type(value) is BigInt if ! IsBigIntElementType(type) is true; otherwise, Type(value) is Number.
        // 4. Let block be arrayBuffer.[[ArrayBufferData]].
        let block = self
            .array_buffer_data
            .as_mut()
            .expect("ArrayBuffer cannot be detached here");

        // 5. Let elementSize be the Element Size value specified in Table 73 for Element Type type.

        // TODO: Agent Record [[LittleEndian]] filed
        // 6. If isLittleEndian is not present, set isLittleEndian to the value of the [[LittleEndian]] field of the surrounding agent's Agent Record.
        let is_little_endian = is_little_endian.unwrap_or(true);

        // 7. Let rawBytes be NumericToRawBytes(type, value, isLittleEndian).
        let raw_bytes = Self::numeric_to_raw_bytes(t, value, is_little_endian, context)?;

        // TODO: Shared Array Buffer
        // 8. If IsSharedArrayBuffer(arrayBuffer) is true, then

        // 9. Else, store the individual bytes of rawBytes into block, starting at block[byteIndex].
        for (i, raw_byte) in raw_bytes.iter().enumerate() {
            block[byte_index as usize + i] = *raw_byte;
        }

        // 10. Return NormalCompletion(undefined).
        Ok(JsValue::undefined())
    }
}

/// `CreateByteDataBlock ( size )` abstract operation.
///
/// The abstract operation `CreateByteDataBlock` takes argument `size` (a non-negative
/// integer). For more information, check the [spec][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-createbytedatablock
pub fn create_byte_data_block(size: u64) -> JsResult<Vec<u8>> {
    // 1. Let db be a new Data Block value consisting of size bytes. If it is impossible to
    //    create such a Data Block, throw a RangeError exception.
    let size = size.try_into().map_err(|e| {
        JsNativeError::range().with_message(format!("couldn't allocate the data block: {e}"))
    })?;

    let mut data_block = Vec::new();
    data_block.try_reserve(size).map_err(|e| {
        JsNativeError::range().with_message(format!("couldn't allocate the data block: {e}"))
    })?;

    // 2. Set all of the bytes of db to 0.
    data_block.resize(size, 0);

    // 3. Return db.
    Ok(data_block)
}

/// `6.2.8.3 CopyDataBlockBytes ( toBlock, toIndex, fromBlock, fromIndex, count )`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-copydatablockbytes
fn copy_data_block_bytes(
    to_block: &mut [u8],
    mut to_index: usize,
    from_block: &[u8],
    mut from_index: usize,
    mut count: usize,
) {
    // 1. Assert: fromBlock and toBlock are distinct values.
    // 2. Let fromSize be the number of bytes in fromBlock.
    let from_size = from_block.len();

    // 3. Assert: fromIndex + count ≤ fromSize.
    assert!(from_index + count <= from_size);

    // 4. Let toSize be the number of bytes in toBlock.
    let to_size = to_block.len();

    // 5. Assert: toIndex + count ≤ toSize.
    assert!(to_index + count <= to_size);

    // 6. Repeat, while count > 0,
    while count > 0 {
        // a. If fromBlock is a Shared Data Block, then
        // TODO: Shared Data Block

        // b. Else,
        // i. Assert: toBlock is not a Shared Data Block.
        // ii. Set toBlock[toIndex] to fromBlock[fromIndex].
        to_block[to_index] = from_block[from_index];

        // c. Set toIndex to toIndex + 1.
        to_index += 1;

        // d. Set fromIndex to fromIndex + 1.
        from_index += 1;

        // e. Set count to count - 1.
        count -= 1;
    }

    // 7. Return NormalCompletion(empty).
}

// TODO: Allow unused variants until shared array buffers are implemented.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum SharedMemoryOrder {
    Init,
    SeqCst,
    Unordered,
}
