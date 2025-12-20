//! `VTable` implementations for [`SequenceString`].
use crate::r#type::StringType;
use crate::vtable::JsStringVTable;
use crate::{JsStr, JsString};
use std::alloc::{Layout, dealloc};
use std::cell::Cell;
use std::marker::PhantomData;
use std::process::abort;
use std::ptr::NonNull;

/// A sequential memory array of Latin1 bytes.
#[repr(C)]
pub(crate) struct SequenceString<T: StringType> {
    /// Embedded `VTable` - must be the first field for vtable dispatch.
    vtable: JsStringVTable,
    refcount: Cell<usize>,
    // Invariant, `!Send` and `!Sync`.
    _marker: PhantomData<*mut T>,
    pub(crate) data: [u8; 0],
}

impl<T: StringType> SequenceString<T> {
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
                refcount: seq_refcount::<T>,
                len,
                kind: T::KIND,
            },
            refcount: Cell::new(1),
            _marker: PhantomData,
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

fn seq_clone<T: StringType>(vtable: NonNull<JsStringVTable>) -> JsString {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };
    let Some(strong) = this.refcount.get().checked_add(1) else {
        abort();
    };
    this.refcount.set(strong);
    // SAFETY: validated the string outside this function.
    unsafe { JsString::from_ptr(vtable) }
}

fn seq_drop<T: StringType>(vtable: NonNull<JsStringVTable>) {
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

fn seq_as_str<T: StringType>(vtable: NonNull<JsStringVTable>) -> JsStr<'static> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };
    let len = this.vtable.len;
    let data_ptr = (&raw const this.data).cast::<T::Byte>();

    // SAFETY: SequenceString data is always valid and properly aligned.
    let slice = unsafe { std::slice::from_raw_parts(data_ptr, len) };
    T::str_ctor(slice)
}

/// `VTable` function for refcount, need to return an `Option<usize>`.
#[allow(clippy::unnecessary_wraps)]
fn seq_refcount<T: StringType>(vtable: NonNull<JsStringVTable>) -> Option<usize> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SequenceString<T> = unsafe { vtable.cast().as_ref() };
    Some(this.refcount.get())
}
