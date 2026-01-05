use crate::vtable::JsStringVTable;
use crate::{JsStr, JsString, JsStringKind};
use std::cell::Cell;
use std::process::abort;
use std::ptr::NonNull;

/// A slice of an existing string.
#[repr(C)]
pub(crate) struct SliceString {
    /// Embedded `VTable` - must be first field for vtable dispatch.
    vtable: JsStringVTable,
    // Keep this for refcounting the original string.
    owned: JsString,
    // Pointer to the data itself. This is guaranteed to be safe as long as `owned` is
    // owned.
    data: NonNull<u8>,
    // Length of this string slice.
    len: usize,
    // Whether the string is Latin1 encoded.
    is_latin1: bool,
    // Refcount for this string as we need to clone/drop it as well.
    refcount: Cell<usize>,
}

impl SliceString {
    /// Create a new slice string given its members.
    #[inline]
    #[must_use]
    pub(crate) fn new(owned: &JsString, data: NonNull<u8>, len: usize, is_latin1: bool) -> Self {
        SliceString {
            vtable: JsStringVTable {
                clone: slice_clone,
                drop: slice_drop,
                as_str: slice_as_str,
                refcount: slice_refcount,
                len,
                kind: JsStringKind::Slice,
            },
            owned: owned.clone(),
            data,
            len,
            is_latin1,
            refcount: Cell::new(1),
        }
    }

    /// Returns the owned string as a const reference.
    #[inline]
    #[must_use]
    pub(crate) fn owned(&self) -> &JsString {
        &self.owned
    }
}

pub(super) fn slice_clone(vtable: NonNull<JsStringVTable>) -> JsString {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SliceString = unsafe { vtable.cast().as_ref() };
    let Some(strong) = this.refcount.get().checked_add(1) else {
        abort();
    };
    this.refcount.set(strong);
    // SAFETY: validated the string outside this function.
    unsafe { JsString::from_ptr(vtable) }
}

fn slice_drop(vtable: NonNull<JsStringVTable>) {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SliceString = unsafe { vtable.cast().as_ref() };
    let Some(new) = this.refcount.get().checked_sub(1) else {
        abort();
    };
    this.refcount.set(new);
    if new != 0 {
        return;
    }

    // SAFETY: This is the last reference, so we can deallocate.
    // The vtable pointer is actually pointing to a SliceString, so cast it correctly.
    unsafe {
        drop(Box::from_raw(vtable.cast::<SliceString>().as_ptr()));
    }
}

fn slice_as_str(vtable: NonNull<JsStringVTable>) -> JsStr<'static> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SliceString = unsafe { vtable.cast().as_ref() };
    let len = this.len;
    let is_latin1 = this.is_latin1;
    let data_ptr = this.data.as_ptr();

    // SAFETY: SliceString data points to valid memory owned by owned.
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
fn slice_refcount(vtable: NonNull<JsStringVTable>) -> Option<usize> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SliceString = unsafe { vtable.cast().as_ref() };
    Some(this.refcount.get())
}
