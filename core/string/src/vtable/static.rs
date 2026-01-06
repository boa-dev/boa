use crate::iter::CodePointsIter;
use crate::vtable::JsStringVTable;
use crate::{JsStr, JsString, JsStringKind};
use std::hash::{Hash, Hasher};
use std::ptr::NonNull;

/// A static string with vtable for uniform dispatch.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StaticString {
    /// Embedded `VTable` - must be the first field for vtable dispatch.
    vtable: JsStringVTable,
    /// The actual string data.
    pub(crate) str: JsStr<'static>,
}

impl StaticString {
    /// Create a new static string.
    #[must_use]
    pub const fn new(str: JsStr<'static>) -> Self {
        Self {
            vtable: JsStringVTable {
                clone: static_clone,
                drop: static_drop,
                as_str: static_as_str,
                code_points: static_code_points,
                refcount: static_refcount,
                len: str.len(),
                kind: JsStringKind::Static,
            },
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

#[inline]
pub(crate) fn static_clone(this: NonNull<JsStringVTable>) -> JsString {
    // Static strings don't need ref counting, just copy the pointer.
    // SAFETY: validated the string outside this function.
    unsafe { JsString::from_ptr(this) }
}

#[inline]
fn static_drop(_ptr: NonNull<JsStringVTable>) {
    // Static strings don't need cleanup.
}

#[inline]
fn static_as_str(this: NonNull<JsStringVTable>) -> JsStr<'static> {
    // SAFETY: validated the string outside this function.
    let this: &StaticString = unsafe { this.cast().as_ref() };
    this.str
}

#[inline]
fn static_code_points(this: NonNull<JsStringVTable>) -> CodePointsIter<'static> {
    CodePointsIter::new(static_as_str(this))
}

#[inline]
fn static_refcount(_ptr: NonNull<JsStringVTable>) -> Option<usize> {
    // Static strings don't have refcount.
    None
}
