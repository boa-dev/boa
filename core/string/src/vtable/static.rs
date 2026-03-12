use crate::iter::CodePointsIter;
use crate::vtable::{JsStringVTable, RawJsString};
use crate::{JsStr, JsStringKind};
use std::hash::{Hash, Hasher};
use std::ptr::{self, NonNull};

/// Static vtable for static strings.
pub(crate) static STATIC_VTABLE: JsStringVTable = JsStringVTable {
    as_str: static_as_str,
    code_points: static_code_points,
    code_unit_at: static_code_unit_at,
    dealloc: |_| {}, // Static strings are never deallocated.
    kind: JsStringKind::Static,
};

/// A static string with vtable for uniform dispatch.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StaticString {
    /// Standardized header for all strings.
    pub(crate) header: RawJsString,
    /// The actual string data.
    pub(crate) str: JsStr<'static>,
}

impl StaticString {
    /// Create a new static string.
    #[must_use]
    pub const fn new(str: JsStr<'static>) -> Self {
        Self {
            header: RawJsString {
                vtable: &STATIC_VTABLE,
                len: str.len(),
                refcount: 0,
                hash: 0,
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

// Unused static_clone removed.

#[inline]
fn static_as_str(header: &RawJsString) -> JsStr<'_> {
    // SAFETY: The header is part of a StaticString and it's aligned.
    let this: &StaticString = unsafe { &*ptr::from_ref(header).cast::<StaticString>() };
    this.str
}

#[inline]
fn static_code_points(ptr: NonNull<RawJsString>) -> CodePointsIter<'static> {
    // SAFETY: ptr is valid.
    let header = unsafe { ptr.as_ref() };
    CodePointsIter::new(static_as_str(header))
}

#[inline]
fn static_code_unit_at(ptr: NonNull<RawJsString>, index: usize) -> Option<u16> {
    // SAFETY: ptr is valid.
    let header = unsafe { ptr.as_ref() };
    static_as_str(header).get(index)
}

// Unused static_refcount removed.
