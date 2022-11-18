use std::{hash::Hash, ptr::NonNull};

/// Wrapper for an interned str pointer, required to
/// quickly check using a hash if a string is inside an [`Interner`][`super::Interner`].
///
/// # Safety
///
/// This struct could cause Undefined Behaviour on:
/// - Use without ensuring the referenced memory is still allocated.
/// - Construction of an [`InternedStr`] from an invalid [`NonNull<Char>`] pointer.
/// - Construction of an [`InternedStr`] from a [`NonNull<Char>`] pointer
/// without checking if the pointed memory of the [`NonNull<Char>`] outlives
/// the [`InternedStr`].
///
/// In general, this should not be used outside of an [`Interner`][`super::Interner`].
#[derive(Debug)]
pub(super) struct InternedStr<Char> {
    ptr: NonNull<[Char]>,
}

impl<Char> InternedStr<Char> {
    /// Create a new interned string from the given `*const u8` pointer,
    /// length and encoding kind
    ///
    /// # Safety
    ///
    /// Not maintaining the invariants specified on the struct definition
    /// could cause Undefined Behaviour.
    #[inline]
    pub(super) const unsafe fn new(ptr: NonNull<[Char]>) -> Self {
        Self { ptr }
    }

    /// Returns a shared reference to the underlying string.
    ///
    /// # Safety
    ///
    /// Not maintaining the invariants specified on the struct definition
    /// could cause Undefined Behaviour.
    #[inline]
    pub(super) unsafe fn as_ref(&self) -> &[Char] {
        // SAFETY:
        // The caller must ensure `ptr` is still valid throughout the
        // lifetime of `self`.
        unsafe { self.ptr.as_ref() }
    }
}

impl<Char> Clone for InternedStr<Char> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<Char> Copy for InternedStr<Char> {}

impl<Char> Eq for InternedStr<Char> where Char: Eq {}

impl<Char> PartialEq for InternedStr<Char>
where
    Char: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: The caller must verify the invariants
        // specified in the struct definition.
        unsafe { self.as_ref() == other.as_ref() }
    }
}

impl<Char> Hash for InternedStr<Char>
where
    Char: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // SAFETY:
        // The caller must ensure `ptr` is still valid throughout the
        // lifetime of `self`.
        unsafe {
            self.as_ref().hash(state);
        }
    }
}
