#![allow(unstable_name_collisions)]

use std::{alloc, sync::Arc};

use boa_profiler::Profiler;
use portable_atomic::AtomicU8;

use boa_gc::{Finalize, Trace};
use sptr::Strict;

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{internal_methods::get_prototype_from_constructor, ObjectData},
    property::Attribute,
    realm::Realm,
    string::common::StaticJsStrings,
    Context, JsArgs, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
};

use super::{get_slice_range, utils::copy_shared_to_shared, SliceRange};

/// The internal representation of a `SharedArrayBuffer` object.
///
/// This struct implements `Send` and `Sync`, meaning it can be shared between threads
/// running different JS code at the same time.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct SharedArrayBuffer {
    /// The `[[ArrayBufferData]]` internal slot.
    // Shared buffers cannot be detached.
    #[unsafe_ignore_trace]
    data: Arc<Box<[AtomicU8]>>,
}

impl SharedArrayBuffer {
    /// Gets the length of this `SharedArrayBuffer`.
    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }

    /// Gets the inner bytes of this `SharedArrayBuffer`.
    pub(crate) fn data(&self) -> &[AtomicU8] {
        &self.data
    }
}

impl IntrinsicObject for SharedArrayBuffer {
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

impl BuiltInObject for SharedArrayBuffer {
    const NAME: JsString = StaticJsStrings::SHARED_ARRAY_BUFFER;
}

impl BuiltInConstructor for SharedArrayBuffer {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::shared_array_buffer;

    /// `25.1.3.1 SharedArrayBuffer ( length [ , options ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-sharedarraybuffer-constructor
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

        // 3. Return ? AllocateSharedArrayBuffer(NewTarget, byteLength, requestedMaxByteLength).
        Ok(Self::allocate(new_target, byte_length, context)?.into())
    }
}

impl SharedArrayBuffer {
    /// `get SharedArrayBuffer [ @@species ]`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-sharedarraybuffer-@@species
    #[allow(clippy::unnecessary_wraps)]
    fn get_species(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Return the this value.
        Ok(this.clone())
    }

    /// `get SharedArrayBuffer.prototype.byteLength`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-get-sharedarraybuffer.prototype.bytelength
    pub(crate) fn get_byte_length(
        this: &JsValue,
        _args: &[JsValue],
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("SharedArrayBuffer.byteLength called with non-object value")
        })?;
        let obj = obj.borrow();
        // 3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
        let buf = obj.as_shared_array_buffer().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("SharedArrayBuffer.byteLength called with invalid object")
        })?;

        // TODO: 4. Let length be ArrayBufferByteLength(O, seq-cst).
        // 5. Return 𝔽(length).
        let len = buf.data().len() as u64;
        Ok(len.into())
    }

    /// `SharedArrayBuffer.prototype.slice ( start, end )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-sharedarraybuffer.prototype.slice
    fn slice(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ArrayBuffer.slice called with non-object value")
        })?;
        let obj_borrow = obj.borrow();

        // 3. If IsSharedArrayBuffer(O) is false, throw a TypeError exception.
        let buf = obj_borrow.as_shared_array_buffer().ok_or_else(|| {
            JsNativeError::typ().with_message("ArrayBuffer.slice called with invalid object")
        })?;

        let SliceRange {
            start: first,
            length: new_len,
        } = get_slice_range(
            buf.len() as u64,
            args.get_or_undefined(0),
            args.get_or_undefined(1),
            context,
        )?;

        // 14. Let ctor be ? SpeciesConstructor(O, %SharedArrayBuffer%).

        let ctor = obj.species_constructor(StandardConstructors::shared_array_buffer, context)?;

        // 15. Let new be ? Construct(ctor, « 𝔽(newLen) »).
        let new = ctor.construct(&[new_len.into()], Some(&ctor), context)?;

        {
            // 16. Perform ? RequireInternalSlot(new, [[ArrayBufferData]]).
            // 17. If IsSharedArrayBuffer(new) is false, throw a TypeError exception.
            let new_obj = new.borrow();
            let new_buf = new_obj.as_shared_array_buffer().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("SharedArrayBuffer constructor returned invalid object")
            })?;

            // 18. If new.[[ArrayBufferData]] is O.[[ArrayBufferData]], throw a TypeError exception.
            if std::ptr::eq(buf.data().as_ptr(), new_buf.data().as_ptr()) {
                return Err(JsNativeError::typ()
                    .with_message("cannot reuse the same `SharedArrayBuffer` for a slice operation")
                    .into());
            }

            // TODO: 19. If ArrayBufferByteLength(new, seq-cst) < newLen, throw a TypeError exception.
            if (new_buf.len() as u64) < new_len {
                return Err(JsNativeError::typ()
                    .with_message("invalid size of constructed shared array")
                    .into());
            }

            // 20. Let fromBuf be O.[[ArrayBufferData]].
            let from_buf = buf.data();
            // 21. Let toBuf be new.[[ArrayBufferData]].
            let to_buf = new_buf.data();

            // 22. Perform CopyDataBlockBytes(toBuf, 0, fromBuf, first, newLen).

            let first = first as usize;
            let new_len = new_len as usize;

            // SAFETY: `get_slice_range` will always return indices that are in-bounds.
            // This also means that the newly created buffer will have at least `new_len` elements
            // to write to.
            unsafe { copy_shared_to_shared(&from_buf[first..], to_buf, new_len) }
        }

        // 23. Return new.
        Ok(new.into())
    }

    /// `AllocateSharedArrayBuffer ( constructor, byteLength [ , maxByteLength ] )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-allocatesharedarraybuffer
    pub(crate) fn allocate(
        constructor: &JsValue,
        byte_length: u64,
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // TODO:
        // 1. Let slots be « [[ArrayBufferData]] ».
        // 2. If maxByteLength is present and maxByteLength is not empty, let allocatingGrowableBuffer
        //    be true; otherwise let allocatingGrowableBuffer be false.
        // 3. If allocatingGrowableBuffer is true, then
        //     a. If byteLength > maxByteLength, throw a RangeError exception.
        //     b. Append [[ArrayBufferByteLengthData]] and [[ArrayBufferMaxByteLength]] to slots.
        // 4. Else,
        //     a. Append [[ArrayBufferByteLength]] to slots.

        // 5. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%SharedArrayBuffer.prototype%", slots).
        let prototype = get_prototype_from_constructor(
            constructor,
            StandardConstructors::shared_array_buffer,
            context,
        )?;

        // TODO: 6. If allocatingGrowableBuffer is true, let allocLength be maxByteLength;
        // otherwise let allocLength be byteLength.

        // 7. Let block be ? CreateSharedByteDataBlock(allocLength).
        // 8. Set obj.[[ArrayBufferData]] to block.
        let data = create_shared_byte_data_block(byte_length, context)?;

        // TODO:
        // 9. If allocatingGrowableBuffer is true, then
        //     a. Assert: byteLength ≤ maxByteLength.
        //     b. Let byteLengthBlock be ? CreateSharedByteDataBlock(8).
        //     c. Perform SetValueInBuffer(byteLengthBlock, 0, biguint64, ℤ(byteLength), true, seq-cst).
        //     d. Set obj.[[ArrayBufferByteLengthData]] to byteLengthBlock.
        //     e. Set obj.[[ArrayBufferMaxByteLength]] to maxByteLength.

        // 10. Else,
        //     a. Set obj.[[ArrayBufferByteLength]] to byteLength.
        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            ObjectData::shared_array_buffer(Self { data }),
        );

        // 11. Return obj.
        Ok(obj)
    }
}

/// [`CreateSharedByteDataBlock ( size )`][spec] abstract operation.
///
/// Creates a new `Arc<Vec<AtomicU8>>` that can be used as a backing buffer for a [`SharedArrayBuffer`].
///
/// For more information, check the [spec][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-createsharedbytedatablock
pub(crate) fn create_shared_byte_data_block(
    size: u64,
    context: &mut Context<'_>,
) -> JsResult<Arc<Box<[AtomicU8]>>> {
    if size > context.host_hooks().max_buffer_size() {
        return Err(JsNativeError::range()
            .with_message(
                "cannot allocate a buffer that exceeds the maximum buffer size".to_string(),
            )
            .into());
    }

    // 1. Let db be a new Shared Data Block value consisting of size bytes. If it is impossible to
    //    create such a Shared Data Block, throw a RangeError exception.
    let size = size.try_into().map_err(|e| {
        JsNativeError::range().with_message(format!("couldn't allocate the data block: {e}"))
    })?;

    if size == 0 {
        // Must ensure we don't allocate a zero-sized buffer.
        return Ok(Arc::new(Box::new([])));
    }

    // 2. Let execution be the [[CandidateExecution]] field of the surrounding agent's Agent Record.
    // 3. Let eventsRecord be the Agent Events Record of execution.[[EventsRecords]] whose
    //    [[AgentSignifier]] is AgentSignifier().
    // 4. Let zero be « 0 ».
    // 5. For each index i of db, do
    //     a. Append WriteSharedMemory { [[Order]]: init, [[NoTear]]: true, [[Block]]: db,
    //        [[ByteIndex]]: i, [[ElementSize]]: 1, [[Payload]]: zero } to eventsRecord.[[EventList]].
    // 6. Return db.
    let layout = alloc::Layout::array::<AtomicU8>(size).map_err(|e| {
        JsNativeError::range().with_message(format!("couldn't allocate the data block: {e}"))
    })?;

    // SAFETY: We already returned if `size == 0`, making this safe.
    let ptr: *mut AtomicU8 = unsafe { alloc::alloc_zeroed(layout).cast() };

    if ptr.is_null() {
        return Err(JsNativeError::range()
            .with_message("memory allocation failed to allocate buffer")
            .into());
    }

    // SAFETY:
    // - It is ensured by the layout that `buffer` has `size` contiguous elements
    // on its allocation.
    // - The original `ptr` doesn't escape outside this function.
    // - `buffer` is a valid pointer by the null check above.
    let buffer = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr, size)) };

    // Just for good measure.
    assert_eq!(buffer.as_ptr().addr() % std::mem::align_of::<u64>(), 0);

    // 3. Return db.
    Ok(Arc::new(buffer))
}
