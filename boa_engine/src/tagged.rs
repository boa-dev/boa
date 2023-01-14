// Remove when/if https://github.com/rust-lang/rust/issues/95228 stabilizes.
// Right now this allows us to use the stable polyfill from the `sptr` crate, which uses
// the same names from the unstable functions of the `std::ptr` module.
#![allow(unstable_name_collisions)]

use sptr::Strict;
use std::ptr::NonNull;

/// A pointer that can be tagged with an `usize`.
///
/// Only pointers with an alignment of 2-bytes are valid, and the tag must have its MSB unset,
/// since the `usize` must fit inside `usize::BITS - 1` bits.
///
/// # Representation
///
/// If the LSB of the internal [`NonNull`] is set (1), then the pointer address represents
/// a tag, where the remaining MSBs stores the tag. Otherwise, the whole pointer represents
/// the pointer itself.
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
pub(crate) struct Tagged<T>(NonNull<T>);

impl<T> Clone for Tagged<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for Tagged<T> {}

impl<T> Tagged<T> {
    /// Creates a new, tagged `Tagged` pointer from an integer.
    ///
    /// # Requirements
    ///
    /// - `T` must have an alignment of at least 2.
    /// - `tag` must fit inside `usize::BITS - 1` bits
    pub(crate) const fn from_tag(tag: usize) -> Tagged<T> {
        debug_assert!(std::mem::align_of::<T>() >= 2);
        let addr = (tag << 1) | 1;
        // SAFETY: `addr` is never zero, since we always set its LSB to 1
        unsafe { Tagged(NonNull::new_unchecked(sptr::invalid_mut(addr))) }
    }

    /// Creates a new `Tagged` pointer from a raw pointer.
    ///
    /// # Requirements
    ///
    /// - `T` must have an alignment of at least 2.
    ///
    /// # Safety
    ///
    /// - `T` must be non null.
    pub(crate) const unsafe fn from_ptr(ptr: *mut T) -> Tagged<T> {
        debug_assert!(std::mem::align_of::<T>() >= 2);
        // SAFETY: the caller must ensure the invariants hold.
        unsafe { Tagged(NonNull::new_unchecked(ptr)) }
    }

    /// Creates a new `Tagged` pointer from a `NonNull` pointer.
    ///
    /// # Requirements
    ///
    /// - `T` must have an alignment of at least 2.
    pub(crate) const fn from_non_null(ptr: NonNull<T>) -> Tagged<T> {
        debug_assert!(std::mem::align_of::<T>() >= 2);
        Tagged(ptr)
    }

    /// Unwraps the `Tagged` pointer.
    pub(crate) fn unwrap(self) -> UnwrappedTagged<T> {
        let addr = self.0.as_ptr().addr();
        if addr & 1 == 0 {
            UnwrappedTagged::Ptr(self.0)
        } else {
            UnwrappedTagged::Tag(addr >> 1)
        }
    }

    /// Gets the address of the inner pointer.
    #[allow(unused)]
    pub(crate) fn addr(self) -> usize {
        self.0.as_ptr().addr()
    }

    /// Returns `true` if `self ` is a tagged pointer.
    #[allow(unused)]
    pub(crate) fn is_tagged(self) -> bool {
        self.0.as_ptr().addr() & 1 > 0
    }
}

/// The unwrapped value of a [`Tagged`] pointer.
#[derive(Debug, Clone, Copy)]
pub(crate) enum UnwrappedTagged<T> {
    Ptr(NonNull<T>),
    Tag(usize),
}
