//! Boa's implementation of ECMAScript's global `ArrayBuffer` and `SharedArrayBuffer` objects
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-arraybuffer-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer

pub(crate) mod shared;
pub(crate) mod utils;

#[cfg(test)]
mod tests;

pub use shared::SharedArrayBuffer;

use crate::{
    builtins::BuiltInObject,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData},
    property::Attribute,
    realm::Realm,
    string::common::StaticJsStrings,
    symbol::JsSymbol,
    value::IntegerOrInfinity,
    Context, JsArgs, JsResult, JsString, JsValue,
};
use boa_gc::{Finalize, Trace};
use boa_profiler::Profiler;

use self::utils::{SliceRef, SliceRefMut};

use super::{BuiltInBuilder, BuiltInConstructor, IntrinsicObject};

#[derive(Debug, Clone, Copy)]
pub(crate) enum BufferRef<'a> {
    Common(&'a ArrayBuffer),
    Shared(&'a SharedArrayBuffer),
}

impl BufferRef<'_> {
    pub(crate) fn data(&self) -> Option<SliceRef<'_>> {
        match self {
            Self::Common(buf) => buf.data().map(SliceRef::Common),
            Self::Shared(buf) => Some(SliceRef::Atomic(buf.data())),
        }
    }

    pub(crate) fn is_detached(&self) -> bool {
        self.data().is_none()
    }
}

#[derive(Debug)]
pub(crate) enum BufferRefMut<'a> {
    Common(&'a mut ArrayBuffer),
    Shared(&'a mut SharedArrayBuffer),
}

impl BufferRefMut<'_> {
    pub(crate) fn data_mut(&mut self) -> Option<SliceRefMut<'_>> {
        match self {
            Self::Common(buf) => buf.data_mut().map(SliceRefMut::Common),
            Self::Shared(buf) => Some(SliceRefMut::Atomic(buf.data())),
        }
    }
}

/// The internal representation of an `ArrayBuffer` object.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct ArrayBuffer {
    /// The `[[ArrayBufferData]]` internal slot.
    data: Option<Vec<u8>>,

    /// The `[[ArrayBufferDetachKey]]` internal slot.
    detach_key: JsValue,
}

impl ArrayBuffer {
    pub(crate) fn from_data(data: Vec<u8>, detach_key: JsValue) -> Self {
        Self {
            data: Some(data),
            detach_key,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.data.as_ref().map_or(0, Vec::len)
    }

    pub(crate) fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    pub(crate) fn data_mut(&mut self) -> Option<&mut [u8]> {
        self.data.as_deref_mut()
    }

    /// Detaches the inner data of this `ArrayBuffer`, returning the original buffer if still
    /// present.
    ///
    /// # Errors
    ///
    /// Throws an error if the provided detach key is invalid.
    pub fn detach(&mut self, key: &JsValue) -> JsResult<Option<Vec<u8>>> {
        if !JsValue::same_value(&self.detach_key, key) {
            return Err(JsNativeError::typ()
                .with_message("Cannot detach array buffer with different key")
                .into());
        }

        Ok(self.data.take())
    }

    /// `25.1.2.2 IsDetachedBuffer ( arrayBuffer )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-isdetachedbuffer
    pub(crate) const fn is_detached(&self) -> bool {
        // 1. If arrayBuffer.[[ArrayBufferData]] is null, return true.
        // 2. Return false.
        self.data.is_none()
    }
}

impl IntrinsicObject for ArrayBuffer {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let flag_attributes = Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE;

        let get_species = BuiltInBuilder::callable(realm, Self::get_species)
            .name(js_string!("get [Symbol.species]"))
            .build();

        let get_byte_length = BuiltInBuilder::callable(realm, Self::get_byte_length)
            .name(js_string!("get byteLength"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("byteLength"),
                Some(get_byte_length),
                None,
                flag_attributes,
            )
            .static_accessor(
                JsSymbol::species(),
                Some(get_species),
                None,
                Attribute::CONFIGURABLE,
            )
            .static_method(Self::is_view, js_string!("isView"), 1)
            .method(Self::slice, js_string!("slice"), 2)
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

impl BuiltInObject for ArrayBuffer {
    const NAME: JsString = StaticJsStrings::ARRAY_BUFFER;
}

impl BuiltInConstructor for ArrayBuffer {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::array_buffer;

    /// `25.1.3.1 ArrayBuffer ( length )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer-length
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
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
}

impl ArrayBuffer {
    /// `25.1.4.3 get ArrayBuffer [ @@species ]`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-arraybuffer-@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
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
    fn is_view(_: &JsValue, args: &[JsValue], _context: &mut Context<'_>) -> JsResult<JsValue> {
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
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ArrayBuffer.byteLength called with non-object value")
        })?;
        let obj = obj.borrow();
        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
        let buf = obj.as_array_buffer().ok_or_else(|| {
            JsNativeError::typ().with_message("ArrayBuffer.byteLength called with invalid object")
        })?;

        // 4. If IsDetachedBuffer(O) is true, return +0𝔽.
        // 5. Let length be O.[[ArrayBufferByteLength]].
        // 6. Return 𝔽(length).
        Ok(buf.len().into())
    }

    /// `25.1.5.3 ArrayBuffer.prototype.slice ( start, end )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-arraybuffer.prototype.slice
    fn slice(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ArrayBuffer.slice called with non-object value")
        })?;
        let obj_borrow = obj.borrow();

        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
        let buf = obj_borrow.as_array_buffer().ok_or_else(|| {
            JsNativeError::typ().with_message("ArrayBuffer.slice called with invalid object")
        })?;

        // 4. If IsDetachedBuffer(O) is true, throw a TypeError exception.
        if buf.is_detached() {
            return Err(JsNativeError::typ()
                .with_message("ArrayBuffer.slice called with detached buffer")
                .into());
        }

        let SliceRange {
            start: first,
            length: new_len,
        } = get_slice_range(
            buf.len() as u64,
            args.get_or_undefined(0),
            args.get_or_undefined(1),
            context,
        )?;

        // 15. Let ctor be ? SpeciesConstructor(O, %ArrayBuffer%).
        let ctor = obj.species_constructor(StandardConstructors::array_buffer, context)?;

        // 16. Let new be ? Construct(ctor, « 𝔽(newLen) »).
        let new = ctor.construct(&[new_len.into()], Some(&ctor), context)?;

        {
            let new_obj = new.borrow();
            // 17. Perform ? RequireInternalSlot(new, [[ArrayBufferData]]).
            // 18. If IsSharedArrayBuffer(new) is true, throw a TypeError exception.
            let new_array_buffer = new_obj.as_array_buffer().ok_or_else(|| {
                JsNativeError::typ().with_message("ArrayBuffer constructor returned invalid object")
            })?;

            // 19. If IsDetachedBuffer(new) is true, throw a TypeError exception.
            if new_array_buffer.is_detached() {
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
                .with_message("new ArrayBuffer is the same as this ArrayBuffer")
                .into());
        }

        {
            let mut new_obj_borrow = new.borrow_mut();
            let new_array_buffer = new_obj_borrow
                .as_array_buffer_mut()
                .expect("Already checked that `new_obj` was an `ArrayBuffer`");

            // 21. If new.[[ArrayBufferByteLength]] < newLen, throw a TypeError exception.
            if (new_array_buffer.len() as u64) < new_len {
                return Err(JsNativeError::typ()
                    .with_message("new ArrayBuffer length too small")
                    .into());
            }

            // 22. NOTE: Side-effects of the above steps may have detached O.
            // 24. Let fromBuf be O.[[ArrayBufferData]].
            let Some(from_buf) = buf.data() else {
                // 23. If IsDetachedBuffer(O) is true, throw a TypeError exception.
                return Err(JsNativeError::typ()
                    .with_message("ArrayBuffer detached while ArrayBuffer.slice was running")
                    .into());
            };

            // 25. Let toBuf be new.[[ArrayBufferData]].
            let to_buf = new_array_buffer
                .data
                .as_mut()
                .expect("ArrayBuffer cannot be detached here");

            // 26. Perform CopyDataBlockBytes(toBuf, 0, fromBuf, first, newLen).
            let first = first as usize;
            let new_len = new_len as usize;
            to_buf[..new_len].copy_from_slice(&from_buf[first..first + new_len]);
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
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // 1. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%ArrayBuffer.prototype%", « [[ArrayBufferData]], [[ArrayBufferByteLength]], [[ArrayBufferDetachKey]] »).
        let prototype = get_prototype_from_constructor(
            constructor,
            StandardConstructors::array_buffer,
            context,
        )?;

        // 2. Let block be ? CreateByteDataBlock(byteLength).
        let block = create_byte_data_block(byte_length, context)?;

        // 3. Set obj.[[ArrayBufferData]] to block.
        // 4. Set obj.[[ArrayBufferByteLength]] to byteLength.
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ObjectData::array_buffer(Self {
                data: Some(block),
                detach_key: JsValue::Undefined,
            }),
        );

        // 5. Return obj.
        Ok(obj)
    }
}

/// Utility struct to return the result of the [`get_slice_range`] function.
#[derive(Debug, Clone, Copy)]
struct SliceRange {
    start: u64,
    length: u64,
}

/// Gets the slice copy range from the original length, the relative start and the end.
fn get_slice_range(
    len: u64,
    relative_start: &JsValue,
    end: &JsValue,
    context: &mut Context<'_>,
) -> JsResult<SliceRange> {
    // 5. Let len be O.[[ArrayBufferByteLength]].

    // 6. Let relativeStart be ? ToIntegerOrInfinity(start).
    let relative_start = relative_start.to_integer_or_infinity(context)?;

    let first = match relative_start {
        // 7. If relativeStart is -∞, let first be 0.
        IntegerOrInfinity::NegativeInfinity => 0,
        // 8. Else if relativeStart < 0, let first be max(len + relativeStart, 0).
        IntegerOrInfinity::Integer(i) if i < 0 => len.checked_add_signed(i).unwrap_or(0),
        // 9. Else, let first be min(relativeStart, len).
        IntegerOrInfinity::Integer(i) => std::cmp::min(i as u64, len),
        IntegerOrInfinity::PositiveInfinity => len,
    };

    // 10. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
    let r#final = if end.is_undefined() {
        len
    } else {
        match end.to_integer_or_infinity(context)? {
            // 11. If relativeEnd is -∞, let final be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 12. Else if relativeEnd < 0, let final be max(len + relativeEnd, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => len.checked_add_signed(i).unwrap_or(0),
            // 13. Else, let final be min(relativeEnd, len).
            IntegerOrInfinity::Integer(i) => std::cmp::min(i as u64, len),
            IntegerOrInfinity::PositiveInfinity => len,
        }
    };

    // 14. Let newLen be max(final - first, 0).
    let new_len = r#final.saturating_sub(first);

    Ok(SliceRange {
        start: first,
        length: new_len,
    })
}

/// `CreateByteDataBlock ( size )` abstract operation.
///
/// The abstract operation `CreateByteDataBlock` takes argument `size` (a non-negative
/// integer). For more information, check the [spec][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-createbytedatablock
pub(crate) fn create_byte_data_block(size: u64, context: &mut Context<'_>) -> JsResult<Vec<u8>> {
    if size > context.host_hooks().max_buffer_size() {
        return Err(JsNativeError::range()
            .with_message(
                "cannot allocate a buffer that exceeds the maximum buffer size".to_string(),
            )
            .into());
    }
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
