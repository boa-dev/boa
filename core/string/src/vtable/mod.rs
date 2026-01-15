//! Module defining the [`JsString`] `VTable` and kinds of strings.
use crate::iter::CodePointsIter;
use crate::{JsStr, JsString, JsStringKind};
use std::ptr::NonNull;

mod sequence;
pub(crate) use sequence::SequenceString;

pub(crate) mod slice;
pub(crate) use slice::SliceString;

pub(crate) mod r#static;
pub use r#static::StaticString;

/// Embedded vtable for `JsString` operations. This is stored directly in each string
/// struct (not as a reference) to eliminate one level of indirection on hot paths.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub(crate) struct JsStringVTable {
    /// Clone the string, incrementing the refcount.
    pub clone: fn(NonNull<JsStringVTable>) -> JsString,
    /// Drop the string, decrementing the refcount and freeing if needed.
    pub drop: fn(NonNull<JsStringVTable>),
    /// Get the string as a `JsStr`. Although this is marked as `'static`, this is really
    /// of the lifetime of the string itself. This is conveyed by the [`JsString`] API
    /// itself rather than this vtable.
    pub as_str: fn(NonNull<JsStringVTable>) -> JsStr<'static>,
    /// Get an iterator of code points. This is the basic form of character access.
    /// Although this is marked as `'static`, this is really of the lifetime of the string
    /// itself. This is conveyed by the [`JsString`] API itself rather than this vtable.
    pub code_points: fn(NonNull<JsStringVTable>) -> CodePointsIter<'static>,
    /// Get the refcount, if applicable.
    pub refcount: fn(NonNull<JsStringVTable>) -> Option<usize>,
    /// Get the length of the string. Since a string is immutable, this does not need
    /// to be a call, can be calculated at construction.
    pub len: usize,
    /// Kind tag to identify the string type.
    pub kind: JsStringKind,
}
