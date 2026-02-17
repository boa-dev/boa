use crate::iter::CodePointsIter;
use crate::vtable::JsStringVTable;
use crate::{JsStr, JsString, JsStringKind};
use std::cell::Cell;
use std::process::abort;
use std::ptr::NonNull;

/// A slice of an existing string.
#[repr(C)]
pub(crate) struct SliceString {
    /// Embedded `VTable` - must be the first field for vtable dispatch.
    vtable: JsStringVTable,
    // Keep this for refcounting the original string.
    owned: JsString,
    // Pointer to the data itself. This is guaranteed to be safe as long as `owned` is
    // owned.
    inner: JsStr<'static>,
    // Refcount for this string as we need to clone/drop it as well.
    refcount: Cell<usize>,
}

impl SliceString {
    /// Create a new slice string given its members.
    ///
    /// # Safety
    /// The caller is responsible for ensuring start and end are safe (`start` <= `end`,
    /// `start` >= 0, `end` <= `owned.len()`).
    #[inline]
    #[must_use]
    pub(crate) unsafe fn new(owned: &JsString, start: usize, end: usize) -> Self {
        // SAFETY: invariant stated for this whole function.
        let inner = unsafe { owned.as_str().get_unchecked(start..end) };
        SliceString {
            vtable: JsStringVTable {
                clone: slice_clone,
                drop: slice_drop,
                as_str: slice_as_str,
                code_points: slice_code_points,
                refcount: slice_refcount,
                len: end - start,
                kind: JsStringKind::Slice,
            },
            owned: owned.clone(),
            // Safety: this whole function is unsafe for the same invariant.
            inner: unsafe { inner.as_static() },
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

#[inline]
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

#[inline]
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

#[inline]
fn slice_as_str(vtable: NonNull<JsStringVTable>) -> JsStr<'static> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SliceString = unsafe { vtable.cast().as_ref() };
    this.inner
}

#[inline]
fn slice_code_points(vtable: NonNull<JsStringVTable>) -> CodePointsIter<'static> {
    CodePointsIter::new(slice_as_str(vtable))
}

/// `VTable` function for refcount, need to return an `Option<usize>`.
#[inline]
#[allow(clippy::unnecessary_wraps)]
fn slice_refcount(vtable: NonNull<JsStringVTable>) -> Option<usize> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SliceString = unsafe { vtable.cast().as_ref() };
    Some(this.refcount.get())
}
