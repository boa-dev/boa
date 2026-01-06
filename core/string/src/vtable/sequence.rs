//! `VTable` implementations for [`SequenceString`].
use crate::iter::CodePointsIter;
use crate::r#type::InternalStringType;
use crate::vtable::JsStringVTable;
use crate::{JsStr, JsString, alloc_overflow};
use std::alloc::{Layout, alloc, dealloc};
use std::cell::Cell;
use std::marker::PhantomData;
use std::process::abort;
use std::ptr;
use std::ptr::NonNull;

/// A sequential memory array of `T::Char` elements.
///
/// # Notes
/// A [`SequenceString`] is `!Sync` (using [`Cell`]) and invariant over `T` (strings
/// of various types cannot be used interchangeably). The string, however, could be
/// `Send`, although within Boa this does not make sense.
#[repr(C)]
pub(crate) struct SequenceString<T: InternalStringType> {
    /// Embedded `VTable` - must be the first field for vtable dispatch.
    vtable: JsStringVTable,
    refcount: Cell<usize>,
    // Forces invariant contract.
    _marker: PhantomData<fn() -> T>,
    pub(crate) data: [u8; 0],
}

impl<T: InternalStringType> SequenceString<T> {
    /// Creates a [`SequenceString`] without data. This should only be used to write to
    /// an allocation which contains all the information.
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

    /// Allocates a new [`SequenceString`] with an internal capacity of `len` characters.
    ///
    /// # Panics
    ///
    /// Panics if `try_allocate_seq` returns `Err`.
    pub(crate) fn allocate(len: usize) -> NonNull<SequenceString<T>> {
        match Self::try_allocate(len) {
            Ok(v) => v,
            Err(None) => alloc_overflow(),
            Err(Some(layout)) => std::alloc::handle_alloc_error(layout),
        }
    }

    /// Allocates a new [`SequenceString`] with an internal capacity of `len` characters.
    ///
    /// # Errors
    ///
    /// Returns `Err(None)` on integer overflows `usize::MAX`.
    /// Returns `Err(Some(Layout))` on allocation error.
    pub(crate) fn try_allocate(len: usize) -> Result<NonNull<Self>, Option<Layout>> {
        let (layout, offset) = Layout::array::<T::Byte>(len)
            .and_then(|arr| T::base_layout().extend(arr))
            .map(|(layout, offset)| (layout.pad_to_align(), offset))
            .map_err(|_| None)?;

        debug_assert_eq!(offset, T::DATA_OFFSET);
        debug_assert_eq!(layout.align(), align_of::<Self>());

        #[allow(clippy::cast_ptr_alignment)]
        // SAFETY:
        // The layout size of `SequenceString` is never zero, since it has to store
        // the length of the string and the reference count.
        let inner = unsafe { alloc(layout).cast::<Self>() };

        // We need to verify that the pointer returned by `alloc` is not null, otherwise
        // we should abort, since an allocation error is pretty unrecoverable for us
        // right now.
        let inner = NonNull::new(inner).ok_or(Some(layout))?;

        // SAFETY:
        // `NonNull` verified for us that the pointer returned by `alloc` is valid,
        // meaning we can write to its pointed memory.
        unsafe {
            // Write the first part, the `SequenceString`.
            inner.as_ptr().write(Self::new(len));
        }

        debug_assert!({
            let inner = inner.as_ptr();
            // SAFETY:
            // - `inner` must be a valid pointer, since it comes from a `NonNull`,
            // meaning we can safely dereference it to `SequenceString`.
            // - `offset` should point us to the beginning of the array,
            // and since we requested a `SequenceString` layout with a trailing
            // `[T::Byte; str_len]`, the memory of the array must be in the `usize`
            // range for the allocation to succeed.
            unsafe {
                // This is `<u8>` as the offset is in bytes.
                ptr::eq(
                    inner.cast::<u8>().add(offset).cast(),
                    (*inner).data().cast_mut(),
                )
            }
        });

        Ok(inner)
    }

    /// Returns the pointer to the data.
    #[inline]
    #[must_use]
    pub(crate) const fn data(&self) -> *const u8 {
        self.data.as_ptr()
    }
}

#[inline]
fn seq_clone<T: InternalStringType>(vtable: NonNull<JsStringVTable>) -> JsString {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };
    let Some(strong) = this.refcount.get().checked_add(1) else {
        abort();
    };
    this.refcount.set(strong);
    // SAFETY: validated the string outside this function.
    unsafe { JsString::from_ptr(vtable) }
}

#[inline]
fn seq_drop<T: InternalStringType>(vtable: NonNull<JsStringVTable>) {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };
    let Some(new) = this.refcount.get().checked_sub(1) else {
        abort();
    };
    this.refcount.set(new);
    if new != 0 {
        return;
    }

    // SAFETY: All the checks for the validity of the layout have already been made on allocation.
    let layout = unsafe {
        Layout::for_value(this)
            .extend(Layout::array::<T::Byte>(this.vtable.len).unwrap_unchecked())
            .unwrap_unchecked()
            .0
            .pad_to_align()
    };

    // SAFETY: If refcount is 0, this is the last reference, so deallocating is safe.
    unsafe {
        dealloc(vtable.as_ptr().cast(), layout);
    }
}

#[inline]
fn seq_as_str<T: InternalStringType>(vtable: NonNull<JsStringVTable>) -> JsStr<'static> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };
    let len = this.vtable.len;
    let data_ptr = (&raw const this.data).cast::<T::Byte>();

    // SAFETY: SequenceString data is always valid and properly aligned.
    let slice = unsafe { std::slice::from_raw_parts(data_ptr, len) };
    T::str_ctor(slice)
}

#[inline]
fn seq_code_points<T: InternalStringType>(
    vtable: NonNull<JsStringVTable>,
) -> CodePointsIter<'static> {
    CodePointsIter::new(seq_as_str::<T>(vtable))
}

/// `VTable` function for refcount, need to return an `Option<usize>`.
#[inline]
#[allow(clippy::unnecessary_wraps)]
fn seq_refcount<T: InternalStringType>(vtable: NonNull<JsStringVTable>) -> Option<usize> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };
    Some(this.refcount.get())
}
