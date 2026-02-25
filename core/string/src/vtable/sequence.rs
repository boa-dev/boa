//! `VTable` implementations for [`SequenceString`].

use crate::iter::CodePointsIter;
use crate::r#type::InternalStringType;
use crate::vtable::JsStringVTable;
use crate::{JsStr, JsString, alloc_overflow};
use std::alloc::{Layout, alloc, dealloc};
use std::cell::Cell;
use std::marker::PhantomData;
use std::mem::align_of;
use std::process::abort;
use std::ptr;
use std::ptr::NonNull;

/// A sequential memory array of string elements.
///
/// This structure represents a dynamically allocated string stored as a contiguous
/// array of code units in memory.
///
/// # Type Safety
///
/// - Not thread-safe (`!Sync`) due to interior mutability via [`Cell`]
/// - Invariant over `T` to prevent mixing different string encodings
/// - Reference counting enables safe shared ownership
#[repr(C)]
pub(crate) struct SequenceString<T: InternalStringType> {
    /// Embedded vtable pointer for dynamic dispatch.
    /// Must be the first field to enable vtable-based polymorphism.
    vtable: JsStringVTable,

    /// Reference count for shared ownership tracking.
    refcount: Cell<usize>,

    /// Zero-sized marker to enforce type invariance.
    _marker: PhantomData<fn() -> T>,

    /// Flexible array member for string data storage.
    /// The actual data follows this structure in memory.
    pub(crate) data: [u8; 0],
}

impl<T: InternalStringType> SequenceString<T> {
    /// Creates a new [`SequenceString`] header without allocating data.
    ///
    /// This method only initializes the metadata. The caller is responsible
    /// for writing the actual string data to the allocated memory region.
    #[inline]
    #[must_use]
    pub(crate) fn new(len: usize) -> Self {
        SequenceString {
            vtable: JsStringVTable {
                clone: seq_clone::<T>,
                drop: seq_drop::<T>,
                as_str: seq_as_str::<T>,
                code_points: seq_code_points::<T>,
                refcount: seq_refcount::<T>,
                len,
                kind: T::KIND,
            },
            refcount: Cell::new(1),
            _marker: PhantomData,
            data: [0; 0],
        }
    }

    /// Allocates a new [`SequenceString`] with capacity for `len` code units.
    ///
    /// # Panics
    ///
    /// Panics if allocation fails or if `len` exceeds implementation limits.
    pub(crate) fn allocate(len: usize) -> NonNull<SequenceString<T>> {
        match Self::try_allocate(len) {
            Ok(v) => v,
            Err(None) => alloc_overflow(),
            Err(Some(layout)) => std::alloc::handle_alloc_error(layout),
        }
    }

    /// Attempts to allocate a new [`SequenceString`] with fallible allocation.
    ///
    /// # Errors
    ///
    /// Returns `Err(None)` if:
    /// - Layout calculation overflows
    ///
    /// Returns `Err(Some(layout))` if:
    /// - Memory allocation fails
    pub(crate) fn try_allocate(len: usize) -> Result<NonNull<Self>, Option<Layout>> {
        // Calculate layout using the original method
        let (layout, offset) = Layout::array::<T::Byte>(len)
            .and_then(|arr| Layout::new::<Self>().extend(arr))
            .map(|(layout, offset)| (layout.pad_to_align(), offset))
            .map_err(|_| None)?;

        debug_assert_eq!(offset, T::DATA_OFFSET);
        debug_assert_eq!(layout.align(), align_of::<Self>());

        // SAFETY:
        // The layout size of `SequenceString` is never zero, since it has to store
        // the length of the string and the reference count.        #[allow(clippy::cast_ptr_alignment)]
        #[allow(clippy::cast_ptr_alignment)]
        let inner = unsafe { alloc(layout).cast::<Self>() };

        // We need to verify that the pointer returned by `alloc` is not null, otherwise
        // we should abort, since an allocation error is pretty unrecoverable for us
        // right now.
        let inner = NonNull::new(inner).ok_or(Some(layout))?;

        // SAFETY
        //`NonNull` verified for us that the pointer returned by `alloc` is valid,
        // meaning we can write to its pointed memory.
        unsafe {
            inner.as_ptr().write(Self::new(len));
        }

        debug_assert!({
            let inner = inner.as_ptr();
            // SAFETY
            // - `inner` must be a valid pointer, since it comes from a `NonNull`,
            // meaning we can safely dereference it to `SequenceString`.
            // - `offset` should point us to the beginning of the array,
            // and since we requested a `SequenceString` layout with a trailing
            // `[T::Byte; str_len]`, the memory of the array must be in the `usize`
            // range for the allocation to succeed.
            unsafe {
                ptr::eq(
                    inner.cast::<u8>().add(offset).cast(),
                    (*inner).data().cast_mut(),
                )
            }
        });

        Ok(inner)
    }

    /// Returns a pointer to the string data.
    ///
    /// The returned pointer points to the flexible array member immediately
    /// following the structure header in memory.
    #[inline]
    #[must_use]
    #[allow(dead_code)] // May be used by external code or future features
    pub(crate) const fn data(&self) -> *const u8 {
        self.data.as_ptr()
    }
}

/// Clones a sequence string by incrementing its reference count.
///
/// # Safety
///
/// The vtable pointer must be valid and point to a properly initialized [`SequenceString<T>`].
#[inline]
fn seq_clone<T: InternalStringType>(vtable: NonNull<JsStringVTable>) -> JsString {
    // SAFETY: Caller guarantees vtable points to a valid SequenceString
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };

    let Some(strong) = this.refcount.get().checked_add(1) else {
        abort();
    };
    this.refcount.set(strong);

    // SAFETY: String validity guaranteed by caller
    unsafe { JsString::from_ptr(vtable) }
}

/// Decrements the reference count and deallocates if it reaches zero.
///
/// # Safety
///
/// The vtable pointer must be valid and point to a properly initialized [`SequenceString<T>`].
#[inline]
fn seq_drop<T: InternalStringType>(vtable: NonNull<JsStringVTable>) {
    // SAFETY: Caller guarantees vtable points to a valid SequenceString
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };

    let Some(new) = this.refcount.get().checked_sub(1) else {
        abort();
    };
    this.refcount.set(new);

    // Only deallocate when last reference is dropped
    if new != 0 {
        return;
    }

    // SAFETY: Layout matches the original allocation
    let layout = unsafe {
        Layout::for_value(this)
            .extend(Layout::array::<T::Byte>(this.vtable.len).unwrap_unchecked())
            .unwrap_unchecked()
            .0
            .pad_to_align()
    };

    // SAFETY: Refcount is zero, so this is the last reference
    unsafe {
        dealloc(vtable.as_ptr().cast(), layout);
    }
}

/// Returns a string slice view of the sequence string.
///
/// # Safety
///
/// The vtable pointer must be valid and point to a properly initialized [`SequenceString<T>`].
#[inline]
fn seq_as_str<T: InternalStringType>(vtable: NonNull<JsStringVTable>) -> JsStr<'static> {
    // SAFETY: Caller guarantees vtable points to a valid SequenceString
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };
    let len = this.vtable.len;
    let data_ptr = (&raw const this.data).cast::<T::Byte>();

    // SAFETY: Data pointer and length are valid by construction
    let slice = unsafe { std::slice::from_raw_parts(data_ptr, len) };
    T::str_ctor(slice)
}

/// Creates a code point iterator for the sequence string.
///
/// # Safety
///
/// The vtable pointer must be valid and point to a properly initialized [`SequenceString<T>`].
#[inline]
fn seq_code_points<T: InternalStringType>(
    vtable: NonNull<JsStringVTable>,
) -> CodePointsIter<'static> {
    CodePointsIter::new(seq_as_str::<T>(vtable))
}

/// Returns the current reference count of the sequence string.
///
/// # Safety
///
/// The vtable pointer must be valid and point to a properly initialized [`SequenceString<T>`].
#[inline]
#[allow(clippy::unnecessary_wraps)]
fn seq_refcount<T: InternalStringType>(vtable: NonNull<JsStringVTable>) -> Option<usize> {
    // SAFETY: Caller guarantees vtable points to a valid SequenceString
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };
    Some(this.refcount.get())
}
