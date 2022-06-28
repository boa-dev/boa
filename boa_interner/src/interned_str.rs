use std::{borrow::Borrow, ptr::NonNull};

/// Wrapper for an interned str pointer, required to
/// quickly check using a hash if a string is inside an [`Interner`][`super::Interner`].
///
/// # Safety
///
/// This struct could cause Undefined Behaviour on:
/// - Use without ensuring the referenced memory is still allocated.
/// - Construction of an [`InternedStr`] from an invalid [`NonNull<str>`].
///
/// In general, this should not be used outside of an [`Interner`][`super::Interner`].
#[derive(Debug, Clone)]
pub(super) struct InternedStr {
    ptr: NonNull<str>,
}

impl InternedStr {
    /// Create a new interned string from the given `str`.
    ///
    /// # Safety
    ///
    /// Not maintaining the invariants specified on the struct definition
    /// could cause Undefined Behaviour.
    #[inline]
    pub(super) unsafe fn new(ptr: NonNull<str>) -> Self {
        Self { ptr }
    }

    /// Returns a shared reference to the underlying string.
    ///
    /// # Safety
    ///
    /// Not maintaining the invariants specified on the struct definition
    /// could cause Undefined Behaviour.
    #[inline]
    pub(super) unsafe fn as_str(&self) -> &str {
        // SAFETY: The caller must verify the invariants
        // specified on the struct definition.
        unsafe { self.ptr.as_ref() }
    }
}

impl std::hash::Hash for InternedStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // SAFETY: The caller must verify the invariants
        // specified in the struct definition.
        unsafe {
            self.as_str().hash(state);
        }
    }
}

impl Eq for InternedStr {}

impl PartialEq for InternedStr {
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: The caller must verify the invariants
        // specified in the struct definition.
        unsafe { self.as_str() == other.as_str() }
    }
}

impl Borrow<str> for InternedStr {
    fn borrow(&self) -> &str {
        // SAFETY: The caller must verify the invariants
        // specified in the struct definition.
        unsafe { self.as_str() }
    }
}
