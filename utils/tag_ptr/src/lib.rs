//! Utility library that enables a pointer to be associated with a tag of type `usize`

use std::ptr::{self, NonNull};

/// A pointer that can be tagged with an `usize`.
///
/// Only pointers with a minimum alignment of 2-bytes are valid, and the tag must have its most
/// significant bit (MSB) unset. In other words, the tag must fit inside `usize::BITS - 1` bits.
/// Using pointers that are not 2-byte aligned won't cause Undefined Behaviour, but it could cause
/// logical errors where the pointer is interpreted as an `usize` instead.
///
/// # Representation
///
/// If the least significant bit (LSB) of the internal [`NonNull`] is set (1), then the pointer
/// address represents a tag where the remaining bits store the tag. Otherwise, the whole pointer
/// represents the pointer itself.
///
/// It uses [`NonNull`], which guarantees that [`Tagged`] can use the "null pointer optimization"
/// to optimize the size of [`Option<Tagged>`].
///
/// # Provenance
///
/// This struct stores a [`NonNull<T>`] instead of a [`NonZeroUsize`][std::num::NonZeroUsize]
/// in order to preserve the provenance of our valid heap pointers.
/// On the other hand, all index values are just casted to invalid pointers, because we don't need to
/// preserve the provenance of [`usize`] indices.
///
/// [tagged_wp]: https://en.wikipedia.org/wiki/Tagged_pointer
#[derive(Debug)]
pub struct Tagged<T>(NonNull<T>);

impl<T> Clone for Tagged<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Tagged<T> {}

impl<T> Tagged<T> {
    /// Creates a new, tagged `Tagged` pointer from an integer.
    ///
    /// # Requirements
    ///
    /// - `tag` must fit inside `usize::BITS - 1` bits
    #[inline]
    #[must_use]
    pub const fn from_tag(tag: usize) -> Self {
        let addr = (tag << 1) | 1;
        // SAFETY: `addr` is never zero, since we always set its LSB to 1
        unsafe { Self(NonNull::new_unchecked(ptr::without_provenance_mut(addr))) }
    }

    /// Creates a new `Tagged` pointer from a raw pointer.
    ///
    /// # Requirements
    ///
    /// - `ptr` must have an alignment of at least 2.
    ///
    /// # Safety
    ///
    /// - `ptr` must be non null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut T) -> Self {
        // SAFETY: the caller must ensure the invariants hold.
        unsafe { Self(NonNull::new_unchecked(ptr)) }
    }

    /// Creates a new `Tagged` pointer from a `NonNull` pointer.
    ///
    /// # Requirements
    ///
    /// - `ptr` must have an alignment of at least 2.
    #[inline]
    #[must_use]
    pub const fn from_non_null(ptr: NonNull<T>) -> Self {
        Self(ptr)
    }

    /// Unwraps the `Tagged` pointer.
    #[inline]
    #[must_use]
    pub fn unwrap(self) -> UnwrappedTagged<T> {
        let addr = self.0.as_ptr().addr();
        if addr & 1 == 0 {
            UnwrappedTagged::Ptr(self.0)
        } else {
            UnwrappedTagged::Tag(addr >> 1)
        }
    }

    /// Gets the address of the inner pointer.
    #[inline]
    #[must_use]
    pub fn addr(self) -> usize {
        self.0.as_ptr().addr()
    }

    /// Returns `true` if `self ` is a tagged pointer.
    #[inline]
    #[must_use]
    pub fn is_tagged(self) -> bool {
        self.0.as_ptr().addr() & 1 > 0
    }
}

/// The unwrapped value of a [`Tagged`] pointer.
#[derive(Debug, Clone, Copy)]
pub enum UnwrappedTagged<T> {
    /// Pointer variant.
    Ptr(NonNull<T>),
    /// Tag variant.
    Tag(usize),
}
