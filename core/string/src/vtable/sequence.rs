use std::alloc::{Layout, alloc, dealloc};
use std::marker::PhantomData;
use std::ptr::{self, NonNull};

use crate::iter::CodePointsIter;
use crate::r#type::{InternalStringType, Latin1, Utf16};
use crate::vtable::{JsStringHeader, JsStringVTable};
use crate::{JsStr, JsStringKind, alloc_overflow};
pub(crate) static LATIN1_VTABLE: JsStringVTable = JsStringVTable {
    as_str: seq_as_str::<Latin1>,
    code_points: seq_code_points::<Latin1>,
    code_unit_at: seq_code_unit_at::<Latin1>,
    dealloc: seq_dealloc::<Latin1>,
    kind: JsStringKind::Latin1Sequence,
};

/// Static vtable for UTF-16 sequence strings.
pub(crate) static UTF16_VTABLE: JsStringVTable = JsStringVTable {
    as_str: seq_as_str::<Utf16>,
    code_points: seq_code_points::<Utf16>,
    code_unit_at: seq_code_unit_at::<Utf16>,
    dealloc: seq_dealloc::<Utf16>,
    kind: JsStringKind::Utf16Sequence,
};

/// A sequential memory array of `T::Char` elements.
///
/// # Notes
/// A [`SequenceString`] is `!Sync` (using [`std::cell::Cell`]) and invariant over `T` (strings
/// of various types cannot be used interchangeably). The string, however, could be
/// `Send`, although within Boa this does not make sense.
#[repr(C)]
#[derive(Debug)]
pub struct SequenceString<T: InternalStringType> {
    /// Standardized header for all strings.
    pub(crate) header: JsStringHeader,
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
            header: JsStringHeader::new(T::VTABLE, len, 1),
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
        // SAFETY: The layout size of `SequenceString` is never zero.
        let inner = unsafe { alloc(layout).cast::<Self>() };

        // We need to verify that the pointer returned by `alloc` is not null, otherwise
        // we should abort, since an allocation error is pretty unrecoverable for us
        // right now.
        let inner = NonNull::new(inner).ok_or(Some(layout))?;

        // SAFETY: `NonNull` verified that the pointer is valid.
        unsafe {
            // Write the first part, the `SequenceString`.
            inner.as_ptr().write(Self::new(len));
        }

        debug_assert!({
            let inner = inner.as_ptr();
            // SAFETY: `inner` is a valid pointer and `offset` points to the array start.
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
fn seq_dealloc<T: InternalStringType>(ptr: NonNull<JsStringHeader>) {
    // SAFETY: The vtable ensures that the pointer is valid and points to a
    // SequenceString of the correct type.
    let header = unsafe { ptr.as_ref() };
    // SAFETY: Layout was validated on allocation. The `len` field is guaranteed
    // to be valid for the string type `T`.
    let layout = unsafe {
        T::base_layout()
            .extend(Layout::array::<T::Byte>(header.len).unwrap_unchecked())
            .unwrap_unchecked()
            .0
            .pad_to_align()
    };
    // SAFETY: The `ptr` is a valid `NonNull` pointer to the allocated memory.
    // The `layout` correctly describes the allocation.
    unsafe {
        dealloc(ptr.as_ptr().cast(), layout);
    }
}

#[inline]
fn seq_as_str<T: InternalStringType>(header: &JsStringHeader) -> JsStr<'_> {
    // SAFETY: The header is part of a SequenceString<T> and it's aligned.
    let this: &SequenceString<T> = unsafe { &*ptr::from_ref(header).cast::<SequenceString<T>>() };
    let len = header.len;
    let data_ptr = (&raw const this.data).cast::<T::Byte>();

    // SAFETY: SequenceString data is always valid and properly aligned.
    // `data_ptr` points to the start of the character data, and `len` is the
    // number of characters, which is guaranteed to be within the allocated bounds.
    let slice = unsafe { std::slice::from_raw_parts(data_ptr, len) };
    T::str_ctor(slice)
}

#[inline]
fn seq_code_points<T: InternalStringType>(header: &JsStringHeader) -> CodePointsIter<'_> {
    CodePointsIter::new(seq_as_str::<T>(header))
}

#[inline]
fn seq_code_unit_at<T: InternalStringType>(header: &JsStringHeader, index: usize) -> Option<u16> {
    seq_as_str::<T>(header).get(index)
}
