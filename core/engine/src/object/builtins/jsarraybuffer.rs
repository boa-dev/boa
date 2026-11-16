//! A Rust API wrapper for Boa's `ArrayBuffer` Builtin ECMAScript Object
use crate::{
    Context, JsResult, JsValue,
    builtins::array_buffer::ArrayBuffer,
    context::intrinsics::StandardConstructors,
    error::JsNativeError,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    value::TryFromJs,
};
use boa_gc::{Finalize, GcRef, GcRefMut, Trace};
use std::ops::Deref;

#[doc(inline)]
pub use crate::builtins::array_buffer::{AlignedVec, AtomicU8};

/// `JsArrayBuffer` provides a wrapper for Boa's implementation of the ECMAScript `ArrayBuffer` object
#[derive(Debug, Clone, Trace, Finalize)]
#[boa_gc(unsafe_no_drop)]
pub struct JsArrayBuffer {
    inner: JsObject<ArrayBuffer>,
}

impl From<JsArrayBuffer> for JsObject<ArrayBuffer> {
    #[inline]
    fn from(value: JsArrayBuffer) -> Self {
        value.inner
    }
}

impl From<JsObject<ArrayBuffer>> for JsArrayBuffer {
    #[inline]
    fn from(value: JsObject<ArrayBuffer>) -> Self {
        Self { inner: value }
    }
}

// TODO: Add constructors that also take the `detach_key` as argument.
impl JsArrayBuffer {
    /// Create a new array buffer with byte length.
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::JsArrayBuffer,
    /// # Context, JsResult, JsValue
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::default();
    /// // Creates a blank array buffer of n bytes
    /// let array_buffer = JsArrayBuffer::new(4, context)?;
    ///
    /// assert_eq!(
    ///     array_buffer.detach(&JsValue::undefined())?.as_slice(),
    ///     &[0u8, 0, 0, 0]
    /// );
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new(byte_length: usize, context: &mut Context) -> JsResult<Self> {
        let inner = ArrayBuffer::allocate(
            &context
                .intrinsics()
                .constructors()
                .array_buffer()
                .constructor()
                .into(),
            byte_length as u64,
            None,
            context,
        )?;

        Ok(Self { inner })
    }

    /// Create a new array buffer from byte block.
    ///
    /// This uses the passed byte block as the internal storage, it does not clone it!
    ///
    /// The `byte_length` will be set to `byte_block.len()`.
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::{JsArrayBuffer, AlignedVec},
    /// # Context, JsResult, JsValue,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::default();
    ///
    /// // Create a buffer from a chunk of data
    /// let data_block = AlignedVec::from_iter(0, 0..5);
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context)?;
    ///
    /// assert_eq!(
    ///     array_buffer.detach(&JsValue::undefined())?.as_slice(),
    ///     &[0u8, 1, 2, 3, 4]
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_byte_block(byte_block: AlignedVec<u8>, context: &mut Context) -> JsResult<Self> {
        let constructor = context
            .intrinsics()
            .constructors()
            .array_buffer()
            .constructor()
            .into();

        // 1. Let obj be ? OrdinaryCreateFromConstructor(constructor, "%ArrayBuffer.prototype%", « [[ArrayBufferData]], [[ArrayBufferByteLength]], [[ArrayBufferDetachKey]] »).
        let prototype = get_prototype_from_constructor(
            &constructor,
            StandardConstructors::array_buffer,
            context,
        )?;

        // 2. Let block be ? CreateByteDataBlock(byteLength).
        //
        // NOTE: We skip step 2. because we already have the block
        // that is passed to us as a function argument.
        let block = byte_block;

        // 3. Set obj.[[ArrayBufferData]] to block.
        // 4. Set obj.[[ArrayBufferByteLength]] to byteLength.
        let obj = JsObject::new(
            context.root_shape(),
            prototype,
            ArrayBuffer::from_data(block, JsValue::undefined()),
        );

        Ok(Self { inner: obj })
    }

    /// Creates an `ArrayBuffer` that aliases a region of embedder-owned memory.
    ///
    /// Unlike [`JsArrayBuffer::from_byte_block`], this does **not** copy nor take
    /// ownership of the memory: the bytes of the resulting `ArrayBuffer` are the
    /// provided region itself. Writes performed by JavaScript code are immediately
    /// visible to the embedder and vice versa, enabling zero-copy sharing of memory
    /// regions like `WebAssembly` linear memories, memory-mapped files or GPU-mapped
    /// buffers.
    ///
    /// The engine accesses the region with normal (non-atomic) loads, stores and
    /// copies, exactly like Boa-owned memory. The exclusive `&'static mut` reference
    /// is what makes this sound: by passing it, the embedder gives up its own access
    /// to the region for as long as the buffer (or a view over it) can reach it.
    /// Embedders that need to keep accessing the region while the buffer is alive
    /// must either synchronize every access with the JavaScript code that may access
    /// the buffer, or use [`JsArrayBuffer::from_external_ptr`]. For regions that are
    /// concurrently accessed from other threads, create a
    /// [`JsSharedArrayBuffer`](crate::object::builtins::JsSharedArrayBuffer) instead,
    /// which accesses its region with atomic operations.
    ///
    /// The resulting buffer is always fixed-length and cannot be resized nor
    /// transferred; those operations throw a `TypeError`. It **can** be detached,
    /// which is the way for an embedder to guarantee that the engine can no longer
    /// access the region; see [`JsArrayBuffer::detach`].
    ///
    /// [`JsArrayBuffer::data`] and [`JsArrayBuffer::data_mut`] return `None` for
    /// externally-backed buffers; use [`JsArrayBuffer::to_vec`] to copy the contents
    /// out, or read the region directly since the embedder owns it.
    ///
    /// # Example
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::JsArrayBuffer,
    /// # property::Attribute,
    /// # Context, JsResult, Source, js_string,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::default();
    /// // The backing region must be 8-byte aligned.
    /// #[repr(align(8))]
    /// struct Backing([u8; 4]);
    /// let backing: &'static mut Backing = Box::leak(Box::new(Backing([0; 4])));
    ///
    /// let array_buffer = JsArrayBuffer::from_external_data(&mut backing.0, context);
    /// assert!(array_buffer.is_external());
    ///
    /// context.register_global_property(js_string!("buf"), array_buffer.clone(), Attribute::all())?;
    /// context.eval(Source::from_bytes("new Uint8Array(buf)[1] = 42;"))?;
    ///
    /// // The embedder gave up its reference to the region, so the contents are
    /// // read back through the buffer itself.
    /// assert_eq!(array_buffer.to_vec().as_deref(), Some(&[0u8, 42, 0, 0][..]));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the region is non-empty and its base address is not aligned to 8
    /// bytes. Typed array views perform aligned accesses of up to 8 bytes on the
    /// backing memory, so the base address must satisfy the largest alignment those
    /// accesses need.
    #[must_use]
    pub fn from_external_data(data: &'static mut [u8], context: &mut Context) -> Self {
        let prototype = context
            .intrinsics()
            .constructors()
            .array_buffer()
            .prototype();

        let data = ArrayBuffer::from_external_data(data);

        let obj = JsObject::new(context.root_shape(), prototype, data);

        Self { inner: obj }
    }

    /// Creates an `ArrayBuffer` that aliases `len` bytes of embedder-owned memory
    /// starting at `ptr`.
    ///
    /// This is a convenience wrapper that builds the `&'static mut [u8]` slice from
    /// its raw parts and delegates to [`JsArrayBuffer::from_external_data`]; see that
    /// method for the aliasing and threading guarantees of the returned buffer.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that:
    ///
    /// - `ptr` is valid for reads and writes of `len` bytes, and the region stays
    ///   valid **and unmoved** at the same address for the whole lifetime of the
    ///   returned buffer (and of every object that shares its data, e.g. typed arrays
    ///   or `DataView`s constructed over it). Note that the garbage collector may keep
    ///   the buffer alive for an unbounded amount of time after it becomes
    ///   unreachable, and that regions that can relocate, like a growable
    ///   `WebAssembly` linear memory that moves its base address on `memory.grow`,
    ///   silently invalidate the buffer unless the embedder guarantees that no
    ///   relocation happens while the buffer is alive.
    /// - The region is not accessed from other threads while JavaScript code that may
    ///   access the buffer is executing, unless all accesses (the engine's and the
    ///   embedder's) are synchronized with a happens-before relationship (e.g. a
    ///   mutex). Regions that are concurrently accessed from other threads, like
    ///   shared `WebAssembly` linear memories, must instead back a
    ///   [`JsSharedArrayBuffer`](crate::object::builtins::JsSharedArrayBuffer); see
    ///   [`JsSharedArrayBuffer::from_external_ptr`](crate::object::builtins::JsSharedArrayBuffer::from_external_ptr).
    ///
    /// # Example
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::{AlignedVec, JsArrayBuffer},
    /// # property::Attribute,
    /// # Context, JsResult, Source, js_string,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # let context = &mut Context::default();
    /// // `AlignedVec` allocations are 64-byte aligned, which satisfies the required
    /// // 8-byte alignment of external regions.
    /// let mut backing: AlignedVec<u8> = AlignedVec::from_iter(0, [0u8; 8]);
    ///
    /// // SAFETY: `backing` stays alive and unmoved for the whole lifetime of the
    /// // context that can reach the buffer.
    /// let array_buffer = unsafe {
    ///     JsArrayBuffer::from_external_ptr(backing.as_mut_ptr(), backing.len(), context)
    /// };
    ///
    /// context.register_global_property(js_string!("buf"), array_buffer, Attribute::all())?;
    /// context.eval(Source::from_bytes("new Uint8Array(buf).fill(42);"))?;
    ///
    /// assert_eq!(&backing[..], &[42u8; 8]);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `ptr` is null, if `len` is bigger than `isize::MAX`, or if the region
    /// is non-empty and `ptr` is not aligned to 8 bytes.
    #[must_use]
    pub unsafe fn from_external_ptr(ptr: *mut u8, len: usize, context: &mut Context) -> Self {
        let prototype = context
            .intrinsics()
            .constructors()
            .array_buffer()
            .prototype();

        // SAFETY: The caller upholds the invariants of `ArrayBuffer::from_external_ptr`.
        let data = unsafe { ArrayBuffer::from_external_ptr(ptr, len) };

        let obj = JsObject::new(context.root_shape(), prototype, data);

        Self { inner: obj }
    }

    /// Returns `true` if this buffer is backed by embedder-owned memory.
    ///
    /// See [`JsArrayBuffer::from_external_data`].
    #[inline]
    #[must_use]
    pub fn is_external(&self) -> bool {
        self.inner.borrow().data().is_external()
    }

    /// Set a maximum length for the underlying array buffer.
    #[inline]
    #[must_use]
    pub fn with_max_byte_length(self, max_byte_len: u64) -> Self {
        self.inner
            .borrow_mut()
            .data_mut()
            .set_max_byte_length(max_byte_len);
        self
    }

    /// Create a [`JsArrayBuffer`] from a [`JsObject`], if the object is not an array buffer throw a `TypeError`.
    ///
    /// This does not clone the fields of the array buffer, it only does a shallow clone of the object.
    #[inline]
    pub fn from_object(object: JsObject) -> JsResult<Self> {
        object
            .downcast::<ArrayBuffer>()
            .map(|inner| Self { inner })
            .map_err(|_| {
                JsNativeError::typ()
                    .with_message("object is not an ArrayBuffer")
                    .into()
            })
    }

    /// Returns the byte length of the array buffer.
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::{JsArrayBuffer, AlignedVec},
    /// # Context, JsResult,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::default();
    /// // Create a buffer from a chunk of data
    /// let data_block = AlignedVec::from_iter(0, 0..5);
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context)?;
    ///
    /// // Take the inner buffer
    /// let buffer_length = array_buffer.byte_length();
    ///
    /// assert_eq!(buffer_length, 5);
    /// # Ok(())
    /// # }
    ///  ```
    #[inline]
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.inner.borrow().data().len()
    }

    /// Take the inner `ArrayBuffer`'s `array_buffer_data` field and replace it with `None`
    ///
    /// # Note
    ///
    /// This tries to detach the pre-existing `JsArrayBuffer`, meaning the original detach
    /// key is required. By default, the key is set to `undefined`.
    ///
    /// For a buffer backed by embedder-owned memory (see
    /// [`JsArrayBuffer::from_external_data`]), this returns a copy of the region's
    /// contents and drops the engine's reference into the region; the embedder remains
    /// the owner of the region itself. This is the way for an embedder to guarantee
    /// that the engine can no longer access the region, e.g. before unmapping or
    /// freeing it.
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::{JsArrayBuffer, AlignedVec},
    /// # Context, JsResult, JsValue
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::default();
    /// // Create a buffer from a chunk of data
    /// let data_block = AlignedVec::from_iter(0, 0..5);
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context)?;
    ///
    /// // Take the inner buffer
    /// let internal_buffer = array_buffer.detach(&JsValue::undefined())?;
    ///
    /// assert_eq!(internal_buffer.as_slice(), &[0u8, 1, 2, 3, 4]);
    ///
    /// // Anymore interaction with the buffer will return an error
    /// let detached_err = array_buffer.detach(&JsValue::undefined());
    /// assert!(detached_err.is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn detach(&self, detach_key: &JsValue) -> JsResult<AlignedVec<u8>> {
        self.inner
            .borrow_mut()
            .data_mut()
            .detach(detach_key)?
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("ArrayBuffer was already detached")
                    .into()
            })
    }

    /// Get an immutable reference to the [`JsArrayBuffer`]'s data.
    ///
    /// Returns `None` if the buffer is detached or backed by embedder-owned memory
    /// (see [`JsArrayBuffer::from_external_data`]); the embedder already owns an
    /// externally-backed region and can read it directly, or copy it out with
    /// [`JsArrayBuffer::to_vec`].
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::{JsArrayBuffer, AlignedVec},
    /// # Context, JsResult, JsValue,
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::default();
    /// // Create a buffer from a chunk of data
    /// let data_block = AlignedVec::from_iter(0, 0..5);
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context)?;
    ///
    /// // Get a reference to the data.
    /// let internal_buffer = array_buffer.data();
    ///
    /// assert_eq!(internal_buffer.as_deref(), Some(&[0u8, 1, 2, 3, 4][..]));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn data(&self) -> Option<GcRef<'_, [u8]>> {
        GcRef::try_map(self.inner.borrow(), |o| o.data().bytes())
    }

    /// Copies the contents of this [`JsArrayBuffer`] into a new [`Vec<u8>`].
    ///
    /// Returns `None` if the buffer has been detached. This works for both Boa-owned
    /// and externally-backed buffers.
    ///
    /// See also [`crate::object::builtins::JsUint8Array::to_vec`] and
    /// [`crate::object::builtins::JsSharedArrayBuffer::to_vec`].
    ///
    /// # Example
    ///
    /// ```
    /// # use boa_engine::object::builtins::{AlignedVec, JsArrayBuffer};
    /// # use boa_engine::{Context, JsResult, JsValue};
    /// # fn main() -> JsResult<()> {
    /// let context = &mut Context::default();
    /// let data = AlignedVec::from_iter(0, [1u8, 2, 3, 4]);
    /// let buffer = JsArrayBuffer::from_byte_block(data, context)?;
    /// assert_eq!(buffer.to_vec(), Some(vec![1u8, 2, 3, 4]));
    ///
    /// buffer.detach(&JsValue::undefined())?;
    /// assert_eq!(buffer.to_vec(), None);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn to_vec(&self) -> Option<Vec<u8>> {
        self.inner
            .borrow()
            .data()
            .slice_ref()
            .map(crate::builtins::array_buffer::utils::SliceRef::to_vec)
    }

    /// Get a mutable reference to the [`JsArrayBuffer`]'s data.
    ///
    /// Returns `None` if the buffer is detached or backed by embedder-owned memory
    /// (see [`JsArrayBuffer::from_external_data`]).
    ///
    /// ```
    /// # use boa_engine::{
    /// # object::builtins::{JsArrayBuffer, AlignedVec},
    /// # Context, JsResult, JsValue
    /// # };
    /// # fn main() -> JsResult<()> {
    /// # // Initialize context
    /// # let context = &mut Context::default();
    /// // Create a buffer from a chunk of data
    /// let data_block = AlignedVec::from_iter(0, 0..5);
    /// let array_buffer = JsArrayBuffer::from_byte_block(data_block, context)?;
    ///
    /// // Get a reference to the data.
    /// let mut internal_buffer = array_buffer
    ///     .data_mut()
    ///     .expect("the buffer should not be detached");
    ///
    /// internal_buffer.fill(10);
    ///
    /// assert_eq!(&*internal_buffer, &[10u8, 10, 10, 10, 10]);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn data_mut(&self) -> Option<GcRefMut<'_, [u8]>> {
        GcRefMut::try_map(self.inner.borrow_mut(), |o| o.data_mut().bytes_mut())
    }
}

impl From<JsArrayBuffer> for JsObject {
    #[inline]
    fn from(o: JsArrayBuffer) -> Self {
        o.inner.upcast()
    }
}

impl From<JsArrayBuffer> for JsValue {
    #[inline]
    fn from(o: JsArrayBuffer) -> Self {
        o.inner.upcast().into()
    }
}

impl Deref for JsArrayBuffer {
    type Target = JsObject<ArrayBuffer>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TryFromJs for JsArrayBuffer {
    fn try_from_js(value: &JsValue, _context: &mut Context) -> JsResult<Self> {
        if let Some(o) = value.as_object() {
            Self::from_object(o.clone())
        } else {
            Err(JsNativeError::typ()
                .with_message("value is not an ArrayBuffer object")
                .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, JsValue};

    #[test]
    fn array_buffer_to_vec_roundtrip_and_detach() {
        let context = &mut Context::default();

        let data = AlignedVec::from_iter(0, [1u8, 2, 3, 4, 5]);
        let buffer = JsArrayBuffer::from_byte_block(data, context).unwrap();

        assert_eq!(buffer.to_vec(), Some(vec![1u8, 2, 3, 4, 5]));

        buffer.detach(&JsValue::undefined()).unwrap();
        assert_eq!(buffer.to_vec(), None);
    }

    #[test]
    fn array_buffer_to_vec_empty() {
        let context = &mut Context::default();

        let data = AlignedVec::from_iter(0, []);
        let buffer = JsArrayBuffer::from_byte_block(data, context).unwrap();

        assert_eq!(buffer.to_vec(), Some(Vec::new()));
    }
}
