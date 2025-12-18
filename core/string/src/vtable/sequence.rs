//! `VTable` implementations for [`Latin1SequenceString`] and [`Utf16SequenceString`].
use crate::vtable::JsStringVTable;
use crate::{JsStr, JsString, JsStringKind};
use std::alloc::{Layout, dealloc};
use std::cell::Cell;
use std::process::abort;
use std::ptr::NonNull;

pub(crate) const LATIN1_DATA_OFFSET: usize = size_of::<Latin1SequenceString>();
pub(crate) const UTF16_DATA_OFFSET: usize = size_of::<Utf16SequenceString>();

/// A sequential memory array of Latin1 bytes.
#[repr(C)]
pub(crate) struct Latin1SequenceString {
    /// Embedded `VTable` - must be first field for vtable dispatch.
    vtable: JsStringVTable,
    len: usize,
    refcount: Cell<usize>,
    pub(crate) data: [u8; 0],
}

impl Latin1SequenceString {
    /// Creates a dummy [`Latin1SequenceString`]. This should only be used to write to
    /// an allocation which contains all the information.
    #[inline]
    #[must_use]
    pub(crate) fn new(len: usize) -> Self {
        Latin1SequenceString {
            vtable: JsStringVTable {
                clone: latin1_seq_clone,
                drop: latin1_seq_drop,
                as_str: latin1_seq_as_str,
                refcount: latin1_seq_refcount,
                len,
                kind: JsStringKind::Latin1Sequence,
            },
            len,
            refcount: Cell::new(1),
            data: [0; 0],
        }
    }

    /// Returns the pointer to the data.
    #[inline]
    #[must_use]
    pub(crate) const fn data(&self) -> *const u8 {
        self.data.as_ptr()
    }
}

fn latin1_seq_clone(vtable: NonNull<JsStringVTable>) -> JsString {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &Latin1SequenceString = unsafe { vtable.cast().as_ref() };
    let Some(strong) = this.refcount.get().checked_add(1) else {
        abort();
    };
    this.refcount.set(strong);
    // SAFETY: validated the string outside this function.
    unsafe { JsString::from_ptr(vtable) }
}

fn latin1_seq_drop(vtable: NonNull<JsStringVTable>) {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &Latin1SequenceString = unsafe { vtable.cast().as_ref() };
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
            .extend(Layout::array::<u8>(this.len).unwrap_unchecked())
            .unwrap_unchecked()
            .0
            .pad_to_align()
    };

    // SAFETY: If refcount is 0, this is the last reference, so deallocating is safe.
    unsafe {
        dealloc(vtable.as_ptr().cast::<u8>(), layout);
    }
}

fn latin1_seq_as_str(vtable: NonNull<JsStringVTable>) -> JsStr<'static> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &Latin1SequenceString = unsafe { vtable.cast().as_ref() };
    let len = this.len;
    let data_ptr = (&raw const this.data).cast::<u8>();

    // SAFETY: Latin1SequenceString data is always valid and properly aligned.
    unsafe { JsStr::latin1(std::slice::from_raw_parts(data_ptr, len)) }
}

/// `VTable` function for refcount, need to return an `Option<usize>`.
#[allow(clippy::unnecessary_wraps)]
fn latin1_seq_refcount(vtable: NonNull<JsStringVTable>) -> Option<usize> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &Latin1SequenceString = unsafe { vtable.cast().as_ref() };
    Some(this.refcount.get())
}

/// A sequential memory array of UTF-16 code units.
#[repr(C)]
pub(crate) struct Utf16SequenceString {
    /// Embedded `VTable` - must be first field for vtable dispatch.
    vtable: JsStringVTable,
    len: usize,
    refcount: Cell<usize>,
    pub(crate) data: [u16; 0],
}

impl Utf16SequenceString {
    /// Creates a dummy [`Utf16SequenceString`]. This should only be used to write to
    /// an allocation which contains all the information.
    #[inline]
    #[must_use]
    pub(crate) fn new(len: usize) -> Self {
        Utf16SequenceString {
            vtable: JsStringVTable {
                clone: utf16_seq_clone,
                drop: utf16_seq_drop,
                as_str: utf16_seq_as_str,
                refcount: utf16_seq_refcount,
                len,
                kind: JsStringKind::Utf16Sequence,
            },
            len,
            refcount: Cell::new(1),
            data: [0; 0],
        }
    }

    /// Returns the pointer to the data.
    #[inline]
    #[must_use]
    pub(crate) const fn data(&self) -> *const u16 {
        self.data.as_ptr()
    }
}

fn utf16_seq_clone(vtable: NonNull<JsStringVTable>) -> JsString {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &Utf16SequenceString = unsafe { vtable.cast().as_ref() };
    let Some(strong) = this.refcount.get().checked_add(1) else {
        abort();
    };
    this.refcount.set(strong);
    // SAFETY: validated the string outside this function.
    unsafe { JsString::from_ptr(vtable) }
}

fn utf16_seq_drop(vtable: NonNull<JsStringVTable>) {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &Utf16SequenceString = unsafe { vtable.cast().as_ref() };
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
            .extend(Layout::array::<u16>(this.len).unwrap_unchecked())
            .unwrap_unchecked()
            .0
            .pad_to_align()
    };

    // SAFETY: If refcount is 0, this is the last reference, so deallocating is safe.
    unsafe {
        dealloc(vtable.as_ptr().cast::<u8>(), layout);
    }
}

fn utf16_seq_as_str(vtable: NonNull<JsStringVTable>) -> JsStr<'static> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &Utf16SequenceString = unsafe { vtable.cast().as_ref() };
    let len = this.len;
    let data_ptr = (&raw const this.data).cast::<u16>();

    // SAFETY: Utf16SequenceString data is always valid and properly aligned.
    unsafe { JsStr::utf16(std::slice::from_raw_parts(data_ptr, len)) }
}

/// `VTable` function for refcount, need to return an `Option<usize>`.
#[allow(clippy::unnecessary_wraps)]
fn utf16_seq_refcount(vtable: NonNull<JsStringVTable>) -> Option<usize> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &Utf16SequenceString = unsafe { vtable.cast().as_ref() };
    Some(this.refcount.get())
}
