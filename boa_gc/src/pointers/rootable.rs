use std::ptr::NonNull;

use super::set_data_ptr;

/// A [`NonNull`] pointer with a `rooted` tag.
///
/// This pointer can be created only from pointers that are 2-byte aligned. In other words,
/// the pointer must point to an address that is a multiple of 2.
pub(crate) struct Rootable<T: ?Sized> {
    ptr: NonNull<T>,
}

impl<T: ?Sized> Copy for Rootable<T> {}

impl<T: ?Sized> Clone for Rootable<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T: ?Sized> std::fmt::Debug for Rootable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rootable")
            .field("ptr", &self.as_ptr())
            .field("is_rooted", &self.is_rooted())
            .finish()
    }
}

impl<T: ?Sized> Rootable<T> {
    /// Creates a new `Rootable` without checking if the [`NonNull`] is properly aligned.
    ///
    /// # Safety
    ///
    /// `ptr` must be 2-byte aligned.
    pub(crate) const unsafe fn new_unchecked(ptr: NonNull<T>) -> Self {
        Self { ptr }
    }

    /// Returns `true` if the pointer is rooted.
    pub(crate) fn is_rooted(self) -> bool {
        self.ptr.as_ptr().cast::<u8>() as usize & 1 != 0
    }

    /// Returns a pointer with the same address as `self` but rooted.
    pub(crate) fn rooted(self) -> Self {
        let ptr = self.ptr.as_ptr();
        let data = ptr.cast::<u8>();
        let addr = data as isize;
        let ptr = set_data_ptr(ptr, data.wrapping_offset((addr | 1) - addr));
        // SAFETY: ptr must be a non null value.
        unsafe { Self::new_unchecked(NonNull::new_unchecked(ptr)) }
    }

    /// Returns a pointer with the same address as `self` but unrooted.
    pub(crate) fn unrooted(self) -> Self {
        let ptr = self.ptr.as_ptr();
        let data = ptr.cast::<u8>();
        let addr = data as isize;
        let ptr = set_data_ptr(ptr, data.wrapping_offset((addr & !1) - addr));
        // SAFETY: ptr must be a non null value
        unsafe { Self::new_unchecked(NonNull::new_unchecked(ptr)) }
    }

    /// Acquires the underlying `NonNull` pointer.
    pub(crate) fn as_ptr(self) -> NonNull<T> {
        self.unrooted().ptr
    }

    /// Returns a shared reference to the pointee.
    ///
    /// # Safety
    ///
    /// See [`NonNull::as_ref`].
    pub(crate) unsafe fn as_ref(&self) -> &T {
        // SAFETY: it is the caller's job to ensure the safety of this operation.
        unsafe { self.as_ptr().as_ref() }
    }
}
