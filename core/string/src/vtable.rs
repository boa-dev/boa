//! Module defining the [`JsString`] `VTable` and kinds of strings.
use crate::{JsStr, JsString, JsStringKind, TaggedLen};
use std::alloc::{Layout, dealloc};
use std::cell::Cell;
use std::hash::{Hash, Hasher};
use std::process::abort;
use std::ptr::NonNull;

pub(super) const DATA_OFFSET: usize = size_of::<SeqString>();

/// Embedded vtable for `JsString` operations. This is stored directly in each string
/// struct (not as a reference) to eliminate one level of indirection on hot paths.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub(crate) struct JsStringVTable {
    /// Clone the string, incrementing the refcount.
    pub clone: fn(NonNull<JsStringVTable>) -> JsString,
    /// Drop the string, decrementing the refcount and freeing if needed.
    pub drop: fn(NonNull<JsStringVTable>),
    /// Get the string as a `JsStr`.
    pub as_str: fn(NonNull<JsStringVTable>) -> JsStr<'static>,
    /// Get the length of the string.
    pub len: fn(NonNull<JsStringVTable>) -> usize,
    /// Get the refcount, if applicable.
    pub refcount: fn(NonNull<JsStringVTable>) -> Option<usize>,
    /// Kind tag to identify the string type.
    pub kind: JsStringKind,
}

/// A sequential memory array of strings.
#[repr(C, align(8))]
pub(crate) struct SeqString {
    /// Embedded `VTable` - must be first field for vtable dispatch.
    vtable: JsStringVTable,
    tagged_len: TaggedLen,
    refcount: Cell<usize>,
    pub(crate) data: [u8; 0],
}

impl SeqString {
    /// Creates a dummy [`SeqString`]. This should only be used to write to
    /// an allocation which contains all the information.
    #[inline]
    #[must_use]
    pub(crate) fn new(len: usize, is_latin1: bool) -> Self {
        SeqString {
            vtable: SEQ_VTABLE,
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

/// A slice of an existing string.
#[repr(C, align(8))]
pub(crate) struct SliceString {
    /// Embedded `VTable` - must be first field for vtable dispatch.
    vtable: JsStringVTable,
    // Keep this for refcounting the original string.
    owned: JsString,
    // Pointer to the data itself. This is guaranteed to be safe as long as `owned` is
    // owned.
    data: NonNull<u8>,
    // Length (and latin1 tag) for this string. We drop start/end.
    tagged_len: TaggedLen,
    // Refcount for this string as we need to clone/drop it as well.
    refcount: Cell<usize>,
}

impl SliceString {
    /// Create a new slice string given its members.
    #[inline]
    #[must_use]
    pub(super) fn new(owned: &JsString, data: NonNull<u8>, len: usize, is_latin1: bool) -> Self {
        SliceString {
            vtable: SLICE_VTABLE,
            owned: owned.clone(),
            data,
            tagged_len: TaggedLen::new(len, is_latin1),
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

/// A static string with vtable for uniform dispatch.
#[derive(Debug, Clone, Copy)]
#[repr(C, align(8))]
pub struct StaticString {
    /// Embedded `VTable` - must be first field for vtable dispatch.
    vtable: JsStringVTable,
    /// The actual string data.
    pub(crate) str: JsStr<'static>,
}

// =============================================================================
// VTable implementations for SeqString
// =============================================================================

pub(super) fn seq_clone(vtable: NonNull<JsStringVTable>) -> JsString {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SeqString = unsafe { vtable.cast().as_ref() };
    let Some(strong) = this.refcount.get().checked_add(1) else {
        abort();
    };
    this.refcount.set(strong);
    // SAFETY: validated the string outside this function.
    unsafe { JsString::from_ptr(vtable) }
}

fn seq_drop(vtable: NonNull<JsStringVTable>) {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SeqString = unsafe { vtable.cast().as_ref() };
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
    let this: &SeqString = unsafe { vtable.cast().as_ref() };
    let len = this.tagged_len.len();
    let is_latin1 = this.tagged_len.is_latin1();
    let data_ptr = (&raw const this.data).cast::<u8>();

    // SAFETY: SeqString data is always valid and properly aligned.
    unsafe {
        if is_latin1 {
            JsStr::latin1(std::slice::from_raw_parts(data_ptr, len))
        } else {
            #[allow(clippy::cast_ptr_alignment)]
            JsStr::utf16(std::slice::from_raw_parts(data_ptr.cast::<u16>(), len))
        }
    }
}

fn seq_len(vtable: NonNull<JsStringVTable>) -> usize {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SeqString = unsafe { vtable.cast().as_ref() };
    this.tagged_len.len()
}

/// `VTable` function for refcount, need to return an `Option<usize>`.
#[allow(clippy::unnecessary_wraps)]
fn seq_refcount(vtable: NonNull<JsStringVTable>) -> Option<usize> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SeqString = unsafe { vtable.cast().as_ref() };
    Some(this.refcount.get())
}

static SEQ_VTABLE: JsStringVTable = JsStringVTable {
    clone: seq_clone,
    drop: seq_drop,
    as_str: seq_as_str,
    len: seq_len,
    refcount: seq_refcount,
    kind: JsStringKind::Sequence,
};

// =============================================================================
// VTable implementations for SliceString
// =============================================================================

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
    let len = this.tagged_len.len();
    let is_latin1 = this.tagged_len.is_latin1();
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

fn slice_len(vtable: NonNull<JsStringVTable>) -> usize {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SliceString = unsafe { vtable.cast().as_ref() };
    this.tagged_len.len()
}

/// `VTable` function for refcount, need to return an `Option<usize>`.
#[allow(clippy::unnecessary_wraps)]
fn slice_refcount(vtable: NonNull<JsStringVTable>) -> Option<usize> {
    // SAFETY: This is part of the correct vtable which is validated on construction.
    let this: &SliceString = unsafe { vtable.cast().as_ref() };
    Some(this.refcount.get())
}

static SLICE_VTABLE: JsStringVTable = JsStringVTable {
    clone: slice_clone,
    drop: slice_drop,
    as_str: slice_as_str,
    len: slice_len,
    refcount: slice_refcount,
    kind: JsStringKind::Slice,
};

// =============================================================================
// VTable implementations for StaticJsString
// =============================================================================

pub(super) fn static_clone(this: NonNull<JsStringVTable>) -> JsString {
    // Static strings don't need refcounting, just copy the pointer.
    // SAFETY: validated the string outside this function.
    unsafe { JsString::from_ptr(this) }
}

fn static_drop(_ptr: NonNull<JsStringVTable>) {
    // Static strings don't need cleanup.
}

fn static_as_str(this: NonNull<JsStringVTable>) -> JsStr<'static> {
    // SAFETY: validated the string outside this function.
    let this: &StaticString = unsafe { this.cast().as_ref() };
    this.str
}

fn static_len(this: NonNull<JsStringVTable>) -> usize {
    // SAFETY: validated the string outside this function.
    let this: &StaticString = unsafe { this.cast().as_ref() };
    this.str.len()
}

fn static_refcount(_ptr: NonNull<JsStringVTable>) -> Option<usize> {
    // Static strings don't have refcount.
    None
}

/// `VTable` for static strings.
static STATIC_VTABLE: JsStringVTable = JsStringVTable {
    clone: static_clone,
    drop: static_drop,
    as_str: static_as_str,
    len: static_len,
    refcount: static_refcount,
    kind: JsStringKind::Static,
};

impl StaticString {
    /// Create a new static string.
    #[must_use]
    pub const fn new(str: JsStr<'static>) -> Self {
        Self {
            vtable: STATIC_VTABLE,
            str,
        }
    }
}

impl Hash for StaticString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.str.hash(state);
    }
}

impl PartialEq for StaticString {
    fn eq(&self, other: &Self) -> bool {
        self.str == other.str
    }
}

impl Eq for StaticString {}

impl std::borrow::Borrow<JsStr<'static>> for &'static StaticString {
    fn borrow(&self) -> &JsStr<'static> {
        &self.str
    }
}
