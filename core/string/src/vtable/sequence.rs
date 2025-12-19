//! `VTable` implementations for [`SequenceString`].
use crate::vtable::JsStringVTable;
use crate::{JsStr, JsString, JsStringKind, TaggedLen};
use std::alloc::{Layout, dealloc};
use std::cell::Cell;
use std::process::abort;
use std::ptr::NonNull;

pub(crate) const DATA_OFFSET: usize = size_of::<SequenceString>();

/// A sequential memory array of strings.
#[repr(C)]
pub(crate) struct SequenceString {
    /// Embedded `VTable` - must be first field for vtable dispatch.
    vtable: JsStringVTable,
    tagged_len: TaggedLen,
    refcount: Cell<usize>,
    pub(crate) data: [u8; 0],
}

impl SequenceString {
    /// Creates a dummy [`SequenceString
    /// `]. This should only be used to write to
    /// an allocation which contains all the information.
    #[inline]
    #[must_use]
    pub(crate) fn new(len: usize, is_latin1: bool) -> Self {
        SequenceString {
            vtable: JsStringVTable {
                clone: seq_clone,
                drop: seq_drop,
                as_str: seq_as_str,
                refcount: seq_refcount,
                len,
                kind: JsStringKind::Sequence,
            },
            tagged_len: TaggedLen::new(len, is_latin1),
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

fn seq_clone(vtable: NonNull<JsStringVTable>) -> JsString {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString = unsafe { vtable.cast().as_ref() };
    let Some(strong) = this.refcount.get().checked_add(1) else {
        abort();
    };
    this.refcount.set(strong);
    // SAFETY: validated the string outside this function.
    unsafe { JsString::from_ptr(vtable) }
}

fn seq_drop(vtable: NonNull<JsStringVTable>) {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString = unsafe { vtable.cast().as_ref() };
    let Some(new) = this.refcount.get().checked_sub(1) else {
        abort();
    };
    this.refcount.set(new);
    if new != 0 {
        return;
    }

    // SAFETY: All the checks for the validity of the layout have already been made on allocation.
    let layout = unsafe {
        if this.tagged_len.is_latin1() {
            Layout::for_value(this)
                .extend(Layout::array::<u8>(this.tagged_len.len()).unwrap_unchecked())
                .unwrap_unchecked()
                .0
                .pad_to_align()
        } else {
            Layout::for_value(this)
                .extend(Layout::array::<u16>(this.tagged_len.len()).unwrap_unchecked())
                .unwrap_unchecked()
                .0
                .pad_to_align()
        }
    };

    // SAFETY: If refcount is 0, this is the last reference, so deallocating is safe.
    unsafe {
        dealloc(vtable.as_ptr().cast::<u8>(), layout);
    }
}

fn seq_as_str(vtable: NonNull<JsStringVTable>) -> JsStr<'static> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString = unsafe { vtable.cast().as_ref() };
    let len = this.tagged_len.len();
    let is_latin1 = this.tagged_len.is_latin1();
    let data_ptr = (&raw const this.data).cast::<u8>();

    // SAFETY: SequenceString
    // data is always valid and properly aligned.
    unsafe {
        if is_latin1 {
            JsStr::latin1(std::slice::from_raw_parts(data_ptr, len))
        } else {
            #[allow(clippy::cast_ptr_alignment)]
            JsStr::utf16(std::slice::from_raw_parts(data_ptr.cast::<u16>(), len))
        }
    }
}

/// `VTable` function for refcount, need to return an `Option<usize>`.
#[allow(clippy::unnecessary_wraps)]
fn seq_refcount(vtable: NonNull<JsStringVTable>) -> Option<usize> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString = unsafe { vtable.cast().as_ref() };
    Some(this.refcount.get())
}
