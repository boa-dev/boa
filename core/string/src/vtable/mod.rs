use crate::iter::CodePointsIter;
use crate::{JsStr, JsStringKind};
use std::ptr::NonNull;

pub(crate) mod sequence;
pub(crate) use sequence::SequenceString;
pub(crate) use sequence::{LATIN1_VTABLE, UTF16_VTABLE};

pub(crate) mod slice;
pub(crate) use slice::SliceString;

pub(crate) mod r#static;
pub use r#static::StaticString;

pub(crate) mod rope;
pub(crate) use rope::RopeString;

/// Header for all `JsString` allocations.
///
/// This is stored at the beginning of every string allocation.
/// By using a reference to a static vtable, we reduce the header size
/// and improve cache locality for common operations.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RawJsString {
    /// Reference to the static vtable for this string kind.
    pub(crate) vtable: &'static JsStringVTable,
    /// Length of the string in code units.
    pub(crate) len: usize,
    /// Reference count for this string.
    pub(crate) refcount: usize,
    /// Kind tag for fast devirtualization without dereferencing the vtable.
    pub(crate) kind: JsStringKind,
    /// Cached hash of the string content.
    pub(crate) hash: u64,
}

// SAFETY: We only mutate refcount and hash via atomic-casts when kind != Static.
unsafe impl Sync for RawJsString {}
// SAFETY: RawJsString contains only thread-safe data.
unsafe impl Send for RawJsString {}

/// Static vtable for `JsString` operations.
///
/// This contains function pointers for polymorphic operations.
/// Shared across all strings of the same kind.
#[derive(Debug, Copy, Clone)]
pub(crate) struct JsStringVTable {
    /// Get the string as a `JsStr`.
    pub as_str: fn(NonNull<RawJsString>) -> JsStr<'static>,
    /// Get an iterator of code points.
    pub code_points: fn(NonNull<RawJsString>) -> CodePointsIter<'static>,
    /// Get the code unit at the given index.
    pub code_unit_at: fn(NonNull<RawJsString>, usize) -> Option<u16>,
    /// Deallocate the string.
    pub dealloc: fn(NonNull<RawJsString>),
}
