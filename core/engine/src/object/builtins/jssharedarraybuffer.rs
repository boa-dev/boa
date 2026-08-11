//! A Rust API wrapper for Boa's `SharedArrayBuffer` Builtin ECMAScript Object
use crate::{
    Context, JsResult, JsValue,
    builtins::array_buffer::{AtomicU8, SharedArrayBuffer, utils::SliceRef},
    error::JsNativeError,
    object::JsObject,
    value::TryFromJs,
};
use boa_gc::{Finalize, Trace};
use std::{ops::Deref, sync::atomic::Ordering};

/// `JsSharedArrayBuffer` provides a wrapper for Boa's implementation of the ECMAScript `ArrayBuffer` object
#[derive(Debug, Clone, Trace, Finalize)]
#[boa_gc(unsafe_no_drop)]
pub struct JsSharedArrayBuffer {
    inner: JsObject<SharedArrayBuffer>,
}

impl From<JsSharedArrayBuffer> for JsObject<SharedArrayBuffer> {
    #[inline]
    fn from(value: JsSharedArrayBuffer) -> Self {
        value.inner
    }
}

impl From<JsObject<SharedArrayBuffer>> for JsSharedArrayBuffer {
    #[inline]
    fn from(value: JsObject<SharedArrayBuffer>) -> Self {
        JsSharedArrayBuffer { inner: value }
    }
}

impl JsSharedArrayBuffer {
    /// Creates a new [`JsSharedArrayBuffer`] with `byte_length` bytes of allocated space.
    #[inline]
    pub fn new(byte_length: usize, context: &mut Context) -> JsResult<Self> {
        let inner = SharedArrayBuffer::allocate(
            &context
                .intrinsics()
                .constructors()
                .shared_array_buffer()
                .constructor()
                .into(),
            byte_length as u64,
            None,
            context,
        )?;

        Ok(Self { inner })
    }

    /// Creates a [`JsSharedArrayBuffer`] from a shared raw buffer.
    #[inline]
    pub fn from_buffer(buffer: SharedArrayBuffer, context: &mut Context) -> Self {
        let proto = context
            .intrinsics()
            .constructors()
            .shared_array_buffer()
            .prototype();

        let inner = JsObject::new(context.root_shape(), proto, buffer);

        Self { inner }
    }

    /// Creates a `SharedArrayBuffer` that aliases a region of embedder-owned memory.
    ///
    /// Unlike [`JsSharedArrayBuffer::new`], this does **not** allocate: the bytes of the
    /// resulting `SharedArrayBuffer` are the provided region itself. Writes performed by
    /// JavaScript code are immediately visible to the embedder and vice versa, enabling
    /// zero-copy sharing of memory regions like `WebAssembly` linear memories,
    /// memory-mapped files or GPU-mapped buffers.
    ///
    /// The engine only ever accesses the region with atomic operations. Accesses to the
    /// region from other threads must be synchronized with the JavaScript code that may
    /// access the buffer concurrently, exactly like for any other `SharedArrayBuffer`
    /// memory.
    ///
    /// The resulting buffer is always fixed-length and cannot be grown.
    ///
    /// # Panics
    ///
    /// Panics if the region is non-empty and its base address is not aligned to 8
    /// bytes. `Atomics` and typed array views perform aligned atomic accesses of up to
    /// 8 bytes on the backing memory, so the base address must satisfy the largest
    /// alignment those accesses need.
    #[inline]
    #[must_use]
    pub fn from_external_data(data: &'static [AtomicU8], context: &mut Context) -> Self {
        Self::from_buffer(SharedArrayBuffer::from_external_data(data), context)
    }

    /// Creates a `SharedArrayBuffer` that aliases `len` bytes of embedder-owned memory
    /// starting at `ptr`.
    ///
    /// This is a convenience wrapper that builds the `&'static [AtomicU8]` slice from
    /// its raw parts and delegates to [`JsSharedArrayBuffer::from_external_data`]; see
    /// that method for the aliasing and threading guarantees of the returned buffer.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that:
    ///
    /// - `ptr` is valid for reads and writes of `len` bytes, and the region stays
    ///   valid **and unmoved** at the same address for the whole lifetime of the
    ///   returned buffer and all of its clones (including clones sent to other
    ///   agents/threads). Note that the garbage collector may keep the buffer alive
    ///   for an unbounded amount of time after it becomes unreachable, and that
    ///   regions that can relocate, like a growable `WebAssembly` linear memory that
    ///   moves its base address on `memory.grow`, silently invalidate the buffer
    ///   unless the embedder guarantees that no relocation happens while the buffer
    ///   is alive.
    /// - All accesses to the region from outside the buffer are performed with atomic
    ///   operations, or are otherwise synchronized with any JavaScript code that may
    ///   access the buffer concurrently.
    ///
    /// # Panics
    ///
    /// Panics if `ptr` is null, if `len` is bigger than `isize::MAX`, or if the region
    /// is non-empty and `ptr` is not aligned to 8 bytes.
    #[inline]
    #[must_use]
    pub unsafe fn from_external_ptr(ptr: *mut u8, len: usize, context: &mut Context) -> Self {
        // SAFETY: The caller upholds the invariants of `SharedArrayBuffer::from_external_ptr`.
        let buffer = unsafe { SharedArrayBuffer::from_external_ptr(ptr, len) };
        Self::from_buffer(buffer, context)
    }

    /// Returns `true` if this buffer is backed by embedder-owned memory.
    ///
    /// See [`JsSharedArrayBuffer::from_external_data`].
    #[inline]
    #[must_use]
    pub fn is_external(&self) -> bool {
        self.borrow().data().is_external()
    }

    /// Creates a [`JsSharedArrayBuffer`] from a [`JsObject`], throwing a `TypeError` if the object
    /// is not a shared array buffer.
    ///
    /// This does not clone the fields of the shared array buffer, it only does a shallow clone of
    /// the object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        object
            .downcast::<SharedArrayBuffer>()
            .map(|inner| Self { inner })
            .map_err(|_| {
                JsNativeError::typ()
                    .with_message("object is not a SharedArrayBuffer")
                    .into()
            })
    }

    /// Returns the byte length of the array buffer.
    #[inline]
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.borrow().data().len(Ordering::SeqCst)
    }

    /// Copies the contents of this [`JsSharedArrayBuffer`] into a new [`Vec<u8>`].
    ///
    /// Each byte is loaded with `SeqCst` ordering into the returned buffer.
    /// GC-safe and safe for concurrent access within Boa's memory model.
    ///
    /// # Example
    ///
    /// ```
    /// # use boa_engine::{Context, JsResult, object::builtins::JsSharedArrayBuffer};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::default();
    /// let sab = JsSharedArrayBuffer::new(64, context)?;
    /// let bytes = sab.to_vec();
    /// assert_eq!(bytes.len(), 64);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn to_vec(&self) -> Vec<u8> {
        let obj = self.borrow();
        let src = obj.data().bytes(Ordering::SeqCst);
        let src = SliceRef::AtomicSlice(src);
        src.to_vec()
    }

    /// Gets the raw buffer of this `JsSharedArrayBuffer`.
    #[inline]
    #[must_use]
    pub fn inner(&self) -> SharedArrayBuffer {
        self.borrow().data().clone()
    }
}

impl From<JsSharedArrayBuffer> for JsObject {
    #[inline]
    fn from(o: JsSharedArrayBuffer) -> Self {
        o.inner.upcast()
    }
}

impl From<JsSharedArrayBuffer> for JsValue {
    #[inline]
    fn from(o: JsSharedArrayBuffer) -> Self {
        o.inner.upcast().into()
    }
}

impl Deref for JsSharedArrayBuffer {
    type Target = JsObject<SharedArrayBuffer>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsSharedArrayBuffer {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        if let Some(o) = value.as_object() {
            Self::from_object(o.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not a SharedArrayBuffer object")
                .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_array_buffer_to_vec_roundtrip() {
        let context = &mut Context::default();
        let len = 64;
        let sab = JsSharedArrayBuffer::new(len, context).unwrap();
        assert_eq!(sab.byte_length(), len);

        // Write a pattern at multiple indices and ensure `to_vec` observes it.
        let inner = sab.inner();
        let atoms = inner.bytes(Ordering::SeqCst);
        atoms[0].store(1, Ordering::SeqCst);
        atoms[1].store(2, Ordering::SeqCst);
        atoms[len - 1].store(255, Ordering::SeqCst);

        let bytes = sab.to_vec();
        assert_eq!(bytes.len(), len);
        assert_eq!(bytes[0], 1);
        assert_eq!(bytes[1], 2);
        assert_eq!(bytes[len - 1], 255);
    }

    #[test]
    fn shared_array_buffer_to_vec_zero_length() {
        let context = &mut Context::default();
        let sab = JsSharedArrayBuffer::new(0, context).unwrap();
        assert_eq!(sab.byte_length(), 0);

        let bytes = sab.to_vec();
        assert!(bytes.is_empty());
    }
}
