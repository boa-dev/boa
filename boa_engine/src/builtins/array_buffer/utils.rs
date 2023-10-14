#![allow(unstable_name_collisions)]

use std::{ptr, slice::SliceIndex, sync::atomic};

use portable_atomic::AtomicU8;
use sptr::Strict;

use crate::{
    builtins::typed_array::{ClampedU8, Element, TypedArrayElement, TypedArrayKind},
    Context, JsObject, JsResult,
};

use super::ArrayBuffer;

#[derive(Debug, Clone, Copy)]
pub(crate) enum SliceRef<'a> {
    Common(&'a [u8]),
    Atomic(&'a [AtomicU8]),
}

impl SliceRef<'_> {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Common(buf) => buf.len(),
            Self::Atomic(buf) => buf.len(),
        }
    }

    pub(crate) fn subslice<I>(&self, index: I) -> SliceRef<'_>
    where
        I: SliceIndex<[u8], Output = [u8]> + SliceIndex<[AtomicU8], Output = [AtomicU8]>,
    {
        match self {
            Self::Common(buffer) => {
                SliceRef::Common(buffer.get(index).expect("index out of bounds"))
            }
            Self::Atomic(buffer) => {
                SliceRef::Atomic(buffer.get(index).expect("index out of bounds"))
            }
        }
    }

    pub(crate) fn addr(&self) -> usize {
        match self {
            Self::Common(buf) => buf.as_ptr().addr(),
            Self::Atomic(buf) => buf.as_ptr().addr(),
        }
    }

    /// [`GetValueFromBuffer ( arrayBuffer, byteIndex, type, isTypedArray, order [ , isLittleEndian ] )`][spec]
    ///
    /// The start offset is determined by the input buffer instead of a `byteIndex` parameter.
    ///
    /// # Safety
    ///
    /// - There must be enough bytes in `buffer` to read an element from an array with type `TypedArrayKind`.
    /// - `buffer` must be aligned to the alignment of said element.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getvaluefrombuffer
    pub(crate) unsafe fn get_value(
        &self,
        kind: TypedArrayKind,
        order: atomic::Ordering,
    ) -> TypedArrayElement {
        unsafe fn read_elem<T: Element>(buffer: SliceRef<'_>, order: atomic::Ordering) -> T {
            // <https://tc39.es/ecma262/#sec-getvaluefrombuffer>

            // 1. Assert: IsDetachedBuffer(arrayBuffer) is false.
            // 2. Assert: There are sufficient bytes in arrayBuffer starting at byteIndex to represent a value of type.
            if cfg!(debug_assertions) {
                assert!(buffer.len() >= std::mem::size_of::<T>());
                assert_eq!(buffer.len() % std::mem::align_of::<T>(), 0);
            }

            // 3. Let block be arrayBuffer.[[ArrayBufferData]].
            // 4. Let elementSize be the Element Size value specified in Table 70 for Element Type type.
            // 5. If IsSharedArrayBuffer(arrayBuffer) is true, then
            //     a. Let execution be the [[CandidateExecution]] field of the surrounding agent's Agent Record.
            //     b. Let eventsRecord be the Agent Events Record of execution.[[EventsRecords]] whose [[AgentSignifier]] is AgentSignifier().
            //     c. If isTypedArray is true and IsNoTearConfiguration(type, order) is true, let noTear be true; otherwise let noTear be false.
            //     d. Let rawValue be a List of length elementSize whose elements are nondeterministically chosen byte values.
            //     e. NOTE: In implementations, rawValue is the result of a non-atomic or atomic read instruction on the underlying hardware. The nondeterminism is a semantic prescription of the memory model to describe observable behaviour of hardware with weak consistency.
            //     f. Let readEvent be ReadSharedMemory { [[Order]]: order, [[NoTear]]: noTear, [[Block]]: block, [[ByteIndex]]: byteIndex, [[ElementSize]]: elementSize }.
            //     g. Append readEvent to eventsRecord.[[EventList]].
            //     h. Append Chosen Value Record { [[Event]]: readEvent, [[ChosenValue]]: rawValue } to execution.[[ChosenValues]].
            // 6. Else,
            //     a. Let rawValue be a List whose elements are bytes from block at indices in the interval from byteIndex (inclusive) to byteIndex + elementSize (exclusive).
            // 7. Assert: The number of elements in rawValue is elementSize.
            // 8. If isLittleEndian is not present, set isLittleEndian to the value of the [[LittleEndian]] field of the surrounding agent's Agent Record.
            // 9. Return RawBytesToNumeric(type, rawValue, isLittleEndian).

            // SAFETY: The invariants of this operation are ensured by the caller.
            unsafe { T::read_from_buffer(buffer, order) }
        }

        let buffer = *self;

        // SAFETY: The invariants of this operation are ensured by the caller.
        unsafe {
            match kind {
                TypedArrayKind::Int8 => read_elem::<i8>(buffer, order).into(),
                TypedArrayKind::Uint8 => read_elem::<u8>(buffer, order).into(),
                TypedArrayKind::Uint8Clamped => read_elem::<ClampedU8>(buffer, order).into(),
                TypedArrayKind::Int16 => read_elem::<i16>(buffer, order).into(),
                TypedArrayKind::Uint16 => read_elem::<u16>(buffer, order).into(),
                TypedArrayKind::Int32 => read_elem::<i32>(buffer, order).into(),
                TypedArrayKind::Uint32 => read_elem::<u32>(buffer, order).into(),
                TypedArrayKind::BigInt64 => read_elem::<i64>(buffer, order).into(),
                TypedArrayKind::BigUint64 => read_elem::<u64>(buffer, order).into(),
                TypedArrayKind::Float32 => read_elem::<f32>(buffer, order).into(),
                TypedArrayKind::Float64 => read_elem::<f64>(buffer, order).into(),
            }
        }
    }

    /// `25.1.2.4 CloneArrayBuffer ( srcBuffer, srcByteOffset, srcLength )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-clonearraybuffer
    pub(crate) fn clone(&self, context: &mut Context<'_>) -> JsResult<JsObject> {
        // 1. Assert: IsDetachedBuffer(srcBuffer) is false.

        // 2. Let targetBuffer be ? AllocateArrayBuffer(%ArrayBuffer%, srcLength).
        let target_buffer = ArrayBuffer::allocate(
            &context
                .realm()
                .intrinsics()
                .constructors()
                .array_buffer()
                .constructor()
                .into(),
            self.len() as u64,
            context,
        )?;

        // 3. Let srcBlock be srcBuffer.[[ArrayBufferData]].

        // 4. Let targetBlock be targetBuffer.[[ArrayBufferData]].
        {
            let mut target_buffer_mut = target_buffer.borrow_mut();
            let target_array_buffer = target_buffer_mut
                .as_array_buffer_mut()
                .expect("This must be an ArrayBuffer");
            let target_block = target_array_buffer
                .data
                .as_deref_mut()
                .expect("ArrayBuffer cannot be detached here");

            // 5. Perform CopyDataBlockBytes(targetBlock, 0, srcBlock, srcByteOffset, srcLength).

            // SAFETY: Both buffers are of the same length, `buffer.len()`, which makes this operation
            // safe.
            unsafe { memcpy(*self, SliceRefMut::Common(target_block), self.len()) }
        }

        // 6. Return targetBuffer.
        Ok(target_buffer)
    }
}

impl<'a> From<&'a [u8]> for SliceRef<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self::Common(value)
    }
}

impl<'a> From<&'a [AtomicU8]> for SliceRef<'a> {
    fn from(value: &'a [AtomicU8]) -> Self {
        Self::Atomic(value)
    }
}

#[derive(Debug)]
pub(crate) enum SliceRefMut<'a> {
    Common(&'a mut [u8]),
    Atomic(&'a [AtomicU8]),
}

impl SliceRefMut<'_> {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Common(buf) => buf.len(),
            Self::Atomic(buf) => buf.len(),
        }
    }

    pub(crate) fn subslice_mut<I>(&mut self, index: I) -> SliceRefMut<'_>
    where
        I: SliceIndex<[u8], Output = [u8]> + SliceIndex<[AtomicU8], Output = [AtomicU8]>,
    {
        match self {
            Self::Common(buffer) => {
                SliceRefMut::Common(buffer.get_mut(index).expect("index out of bounds"))
            }
            Self::Atomic(buffer) => {
                SliceRefMut::Atomic(buffer.get(index).expect("index out of bounds"))
            }
        }
    }

    pub(crate) fn addr(&self) -> usize {
        match self {
            Self::Common(buf) => buf.as_ptr().addr(),
            Self::Atomic(buf) => buf.as_ptr().addr(),
        }
    }

    /// `25.1.2.12 SetValueInBuffer ( arrayBuffer, byteIndex, type, value, isTypedArray, order [ , isLittleEndian ] )`
    ///
    /// The start offset is determined by the input buffer instead of a `byteIndex` parameter.
    ///
    /// # Safety
    ///
    /// - There must be enough bytes in `buffer` to write the `TypedArrayElement`.
    /// - `buffer` must be aligned to the alignment of the `TypedArrayElement`.
    ///
    /// # Panics
    ///
    /// - Panics if the type of `value` is not equal to the content of `kind`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-setvalueinbuffer
    pub(crate) unsafe fn set_value(&mut self, value: TypedArrayElement, order: atomic::Ordering) {
        pub(crate) unsafe fn write_to_buffer<T: Element>(
            buffer: SliceRefMut<'_>,
            value: T,
            order: atomic::Ordering,
        ) {
            // <https://tc39.es/ecma262/#sec-setvalueinbuffer>

            // 1. Assert: IsDetachedBuffer(arrayBuffer) is false.
            // 2. Assert: There are sufficient bytes in arrayBuffer starting at byteIndex to represent a value of type.
            // 3. Assert: value is a BigInt if IsBigIntElementType(type) is true; otherwise, value is a Number.
            if cfg!(debug_assertions) {
                assert!(buffer.len() >= std::mem::size_of::<T>());
                assert_eq!(buffer.len() % std::mem::align_of::<T>(), 0);
            }

            // 4. Let block be arrayBuffer.[[ArrayBufferData]].
            // 5. Let elementSize be the Element Size value specified in Table 70 for Element Type type.
            // 6. If isLittleEndian is not present, set isLittleEndian to the value of the [[LittleEndian]] field of the surrounding agent's Agent Record.
            // 7. Let rawBytes be NumericToRawBytes(type, value, isLittleEndian).
            // 8. If IsSharedArrayBuffer(arrayBuffer) is true, then
            //     a. Let execution be the [[CandidateExecution]] field of the surrounding agent's Agent Record.
            //     b. Let eventsRecord be the Agent Events Record of execution.[[EventsRecords]] whose [[AgentSignifier]] is AgentSignifier().
            //     c. If isTypedArray is true and IsNoTearConfiguration(type, order) is true, let noTear be true; otherwise let noTear be false.
            //     d. Append WriteSharedMemory { [[Order]]: order, [[NoTear]]: noTear, [[Block]]: block, [[ByteIndex]]: byteIndex, [[ElementSize]]: elementSize, [[Payload]]: rawBytes } to eventsRecord.[[EventList]].
            // 9. Else,
            //     a. Store the individual bytes of rawBytes into block, starting at block[byteIndex].
            // 10. Return unused.

            // SAFETY: The invariants of this operation are ensured by the caller.
            unsafe {
                T::write_to_buffer(buffer, value, order);
            }
        }

        // Have to rebind in order to remove the outer `&mut` ref.
        let buffer = match self {
            SliceRefMut::Common(buf) => SliceRefMut::Common(buf),
            SliceRefMut::Atomic(buf) => SliceRefMut::Atomic(buf),
        };

        // SAFETY: The invariants of this operation are ensured by the caller.
        unsafe {
            match value {
                TypedArrayElement::Int8(e) => write_to_buffer(buffer, e, order),
                TypedArrayElement::Uint8(e) => write_to_buffer(buffer, e, order),
                TypedArrayElement::Uint8Clamped(e) => write_to_buffer(buffer, e, order),
                TypedArrayElement::Int16(e) => write_to_buffer(buffer, e, order),
                TypedArrayElement::Uint16(e) => write_to_buffer(buffer, e, order),
                TypedArrayElement::Int32(e) => write_to_buffer(buffer, e, order),
                TypedArrayElement::Uint32(e) => write_to_buffer(buffer, e, order),
                TypedArrayElement::BigInt64(e) => write_to_buffer(buffer, e, order),
                TypedArrayElement::BigUint64(e) => write_to_buffer(buffer, e, order),
                TypedArrayElement::Float32(e) => write_to_buffer(buffer, e, order),
                TypedArrayElement::Float64(e) => write_to_buffer(buffer, e, order),
            }
        }
    }
}

impl<'a> From<&'a mut [u8]> for SliceRefMut<'a> {
    fn from(value: &'a mut [u8]) -> Self {
        Self::Common(value)
    }
}

impl<'a> From<&'a [AtomicU8]> for SliceRefMut<'a> {
    fn from(value: &'a [AtomicU8]) -> Self {
        Self::Atomic(value)
    }
}

/// Copies `count` bytes from `src` into `dest` using atomic relaxed loads and stores.
pub(super) unsafe fn copy_shared_to_shared(src: &[AtomicU8], dest: &[AtomicU8], count: usize) {
    // TODO: this could be optimized with batches of writes using `u32/u64` stores instead.
    for i in 0..count {
        // SAFETY: The invariants of this operation are ensured by the caller of the function.
        unsafe {
            dest.get_unchecked(i).store(
                src.get_unchecked(i).load(atomic::Ordering::Relaxed),
                atomic::Ordering::Relaxed,
            );
        }
    }
}

unsafe fn copy_shared_to_shared_backwards(src: &[AtomicU8], dest: &[AtomicU8], count: usize) {
    for i in (0..count).rev() {
        // SAFETY: The invariants of this operation are ensured by the caller of the function.
        unsafe {
            dest.get_unchecked(i).store(
                src.get_unchecked(i).load(atomic::Ordering::Relaxed),
                atomic::Ordering::Relaxed,
            );
        }
    }
}

/// Copies `count` bytes from the buffer `src` into the buffer `dest`, using the atomic ordering `order`
/// if any of the buffers are atomic.
///
/// # Safety
///
/// - Both `src.len()` and `dest.len()` must have at least `count` bytes to read and write,
/// respectively.
pub(crate) unsafe fn memcpy(src: SliceRef<'_>, dest: SliceRefMut<'_>, count: usize) {
    if cfg!(debug_assertions) {
        assert!(src.len() >= count);
        assert!(dest.len() >= count);
    }

    // TODO: this could be optimized with batches of writes using `u32/u64` stores instead.
    match (src, dest) {
        // SAFETY: The invariants of this operation are ensured by the caller of the function.
        (SliceRef::Common(src), SliceRefMut::Common(dest)) => unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), count);
        },
        // SAFETY: The invariants of this operation are ensured by the caller of the function.
        (SliceRef::Common(src), SliceRefMut::Atomic(dest)) => unsafe {
            for i in 0..count {
                dest.get_unchecked(i)
                    .store(*src.get_unchecked(i), atomic::Ordering::Relaxed);
            }
        },
        // SAFETY: The invariants of this operation are ensured by the caller of the function.
        (SliceRef::Atomic(src), SliceRefMut::Common(dest)) => unsafe {
            for i in 0..count {
                *dest.get_unchecked_mut(i) = src.get_unchecked(i).load(atomic::Ordering::Relaxed);
            }
        },
        // SAFETY: The invariants of this operation are ensured by the caller of the function.
        (SliceRef::Atomic(src), SliceRefMut::Atomic(dest)) => unsafe {
            copy_shared_to_shared(src, dest, count);
        },
    }
}

/// Copies `count` bytes from the position `from` to the position `to` in `buffer`.
///
/// # Safety
///
/// - `from + count <= buffer.len()`
/// - `to + count <= buffer.len()`
pub(crate) unsafe fn memmove(buffer: SliceRefMut<'_>, from: usize, to: usize, count: usize) {
    if cfg!(debug_assertions) {
        assert!(from + count <= buffer.len());
        assert!(to + count <= buffer.len());
    }

    match buffer {
        // SAFETY: The invariants of this operation are ensured by the caller of the function.
        SliceRefMut::Common(buf) => unsafe {
            let ptr = buf.as_mut_ptr();
            let src_ptr = ptr.add(from);
            let dest_ptr = ptr.add(to);
            ptr::copy(src_ptr, dest_ptr, count);
        },
        // SAFETY: The invariants of this operation are ensured by the caller of the function.
        SliceRefMut::Atomic(buf) => unsafe {
            let src = buf.get_unchecked(from..);
            let dest = buf.get_unchecked(to..);

            // Let's draw a simple array.
            //
            // | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
            //
            // Now let's define `from`, `to` and `count` such that the below condition is satisfied.
            // `from = 0`
            // `to = 2`
            // `count = 4`
            //
            // We can now imagine that the array is pointer to by our indices:
            //
            // | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
            //   ^       ^
            // from     to
            //
            // If we start copying bytes until `from + 2 = to`, we can see that the new array would be:
            //
            // | 0 | 1 | 0 | 1 | 0 | 5 | 6 | 7 | 8 |
            //           ^       ^
            //    from + 2       to + 2
            //
            // However, we've lost the data that was in the index 2! If this process
            // continues, this'll give the incorrect result:
            //
            // | 0 | 1 | 0 | 1 | 0 | 1 | 6 | 7 | 8 |
            //
            // To solve this, we just need to copy backwards to ensure we never override data that
            // we need in next iterations:
            //
            // | 0 | 1 | 2 | 3 | 4 | 3 | 6 | 7 | 8 |
            //               ^       ^
            //            from      to
            //
            // | 0 | 1 | 2 | 3 | 2 | 3 | 6 | 7 | 8 |
            //           ^       ^
            //        from      to
            //
            // | 0 | 1 | 0 | 1 | 2 | 3 | 6 | 7 | 8 |
            //   ^       ^
            // from     to
            if from < to && to < from + count {
                copy_shared_to_shared_backwards(src, dest, count);
            } else {
                copy_shared_to_shared(src, dest, count);
            }
        },
    }
}
