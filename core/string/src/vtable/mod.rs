use crate::iter::CodePointsIter;
use crate::{JsStr, JsStringKind};
use std::ptr::NonNull;

/// Sequential string implementation.
pub(crate) mod sequence;
pub(crate) use sequence::SequenceString;

/// Slice string implementation.
pub(crate) mod slice;
pub(crate) use slice::SliceString;

pub(crate) mod r#static;
pub use r#static::StaticString;

/// Rope string implementation.
pub(crate) mod rope;
pub(crate) use rope::RopeString;

/// Header for all `JsString` allocations.
///
/// This is stored at the beginning of every string allocation.
/// The `kind` field is used to index into a global vtable table,
/// avoiding pointer indirection and improving cache locality.
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy)]
pub struct JsStringHeader {
    /// Kind tag identifying the string type. Used to index into `VTABLES`.
    pub(crate) kind: JsStringKind,
    /// Length of the string in code units.
    pub(crate) len: usize,
    /// Reference count for this string.
    pub(crate) refcount: usize,
    /// Explicit padding to ensure `hash` is 8-byte aligned on 32-bit platforms.
    #[cfg(target_pointer_width = "32")]
    pub(crate) _padding: u32,
    /// Cached hash of the string content.
    pub(crate) hash: u64,
}

impl JsStringHeader {
    /// Create a new `JsStringHeader`.
    pub(crate) const fn new(kind: JsStringKind, len: usize, refcount: usize) -> Self {
        Self {
            kind,
            len,
            refcount,
            #[cfg(target_pointer_width = "32")]
            _padding: 0,
            hash: 0,
        }
    }
}

/// Static vtable for `JsString` operations.
///
/// This contains function pointers for polymorphic operations.
/// Vtables are stored by value in the global `VTABLES` array, indexed by [`JsStringKind`].
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct JsStringVTable {
    /// Get the string as a `JsStr`.
    pub(crate) as_str: for<'a> fn(&'a JsStringHeader) -> JsStr<'a>,
    /// Get an iterator of code points.
    pub(crate) code_points: for<'a> fn(&'a JsStringHeader) -> CodePointsIter<'a>,
    /// Get the code unit at the given index.
    pub(crate) code_unit_at: fn(&JsStringHeader, usize) -> Option<u16>,
    /// Deallocate the string.
    pub(crate) dealloc: fn(NonNull<JsStringHeader>),
}

/// Global vtable table indexed by [`JsStringKind`].
///
/// Since `JsString` has a fixed, compile-time-known set of kinds,
/// we store vtables by value in a contiguous array instead of using
/// pointer indirection. This improves cache locality and avoids
/// an extra pointer chase on every dispatch.
pub(crate) static VTABLES: [JsStringVTable; 5] = [
    sequence::LATIN1_VTABLE, // JsStringKind::Latin1Sequence = 0
    sequence::UTF16_VTABLE,  // JsStringKind::Utf16Sequence = 1
    slice::SLICE_VTABLE,     // JsStringKind::Slice = 2
    r#static::STATIC_VTABLE, // JsStringKind::Static = 3
    rope::ROPE_VTABLE,       // JsStringKind::Rope = 4
];
