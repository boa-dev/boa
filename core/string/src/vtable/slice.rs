use crate::iter::CodePointsIter;
use crate::vtable::{JsStringVTable, RawJsString};
use crate::{JsStr, JsString, JsStringKind};
use std::ptr::{self, NonNull};

/// Static vtable for slice strings.
pub(crate) static SLICE_VTABLE: JsStringVTable = JsStringVTable {
    as_str: slice_as_str,
    code_points: slice_code_points,
    code_unit_at: slice_code_unit_at,
    dealloc: slice_dealloc, // Slice strings are now correctly deallocated.
    kind: JsStringKind::Slice,
};

/// A slice of an existing string.
#[repr(C)]
pub(crate) struct SliceString {
    /// Standardized header for all strings.
    pub(crate) header: RawJsString,
    // Keep this for refcounting the original string.
    pub(crate) owned: JsString,
    // Pointer to the data itself. This is guaranteed to be safe as long as `owned` is
    // owned.
    pub(crate) inner: JsStr<'static>,
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
        // SAFETY: The caller is responsible for ensuring start and end are safe (`start` <= `end`,
        // `start` >= 0, `end` <= `owned.len()`).
        let inner = unsafe { owned.as_str().get_unchecked(start..end) };
        SliceString {
            header: RawJsString {
                vtable: &SLICE_VTABLE,
                len: end - start,
                refcount: 1,
                hash: 0,
            },
            owned: owned.clone(),
            // SAFETY: this inner's lifetime is tied to the owned string above.
            // We transmute the lifetime to 'static to satisfy the long-lived nature of the string vtable.
            inner: unsafe { inner.as_static() },
        }
    }

    /// Returns the owned string as a const reference.
    #[inline]
    #[must_use]
    pub(crate) fn owned(&self) -> &JsString {
        &self.owned
    }
}

// Unused slice_clone removed.

#[inline]
fn slice_as_str(header: &RawJsString) -> JsStr<'_> {
    // SAFETY: The header is part of a SliceString and it's aligned.
    let this: &SliceString = unsafe { &*ptr::from_ref(header).cast::<SliceString>() };
    this.inner
}

#[inline]
fn slice_dealloc(ptr: NonNull<RawJsString>) {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    // The pointer is guaranteed to be a valid `NonNull<RawJsString>` pointing to a `SliceString`.
    unsafe {
        drop(Box::from_raw(ptr.cast::<SliceString>().as_ptr()));
    }
}

#[inline]
fn slice_code_points(ptr: NonNull<RawJsString>) -> CodePointsIter<'static> {
    // SAFETY: ptr is valid.
    let header = unsafe { ptr.as_ref() };
    // SAFETY: Transmuting to 'static is currently used for vtable consistency.
    unsafe { std::mem::transmute(CodePointsIter::new(slice_as_str(header))) }
}

#[inline]
fn slice_code_unit_at(ptr: NonNull<RawJsString>, index: usize) -> Option<u16> {
    // SAFETY: ptr is valid.
    let header = unsafe { ptr.as_ref() };
    slice_as_str(header).get(index)
}

// Unused refcount method removed.
