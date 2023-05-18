use crate::{
    finalizer_safe,
    internals::EphemeronBox,
    trace::{Finalize, Trace},
    Allocator, Gc,
};
use std::{cell::Cell, ptr::NonNull};

use super::rootable::Rootable;

/// A key-value pair where the value becomes unaccesible when the key is garbage collected.
///
/// See Racket's explanation on [**ephemerons**][eph] for a brief overview or read Barry Hayes'
/// [_Ephemerons_: a new finalization mechanism][acm].
///
///
/// [eph]: https://docs.racket-lang.org/reference/ephemerons.html
/// [acm]: https://dl.acm.org/doi/10.1145/263700.263733
#[derive(Debug)]
pub struct Ephemeron<K: Trace + ?Sized + 'static, V: Trace + 'static> {
    inner_ptr: Cell<Rootable<EphemeronBox<K, V>>>,
}

impl<K: Trace + ?Sized, V: Trace + Clone> Ephemeron<K, V> {
    /// Gets the stored value of this `Ephemeron`, or `None` if the key was already garbage collected.
    ///
    /// This needs to return a clone of the value because holding a reference to it between
    /// garbage collection passes could drop the underlying allocation, causing an Use After Free.
    pub fn value(&self) -> Option<V> {
        // SAFETY: this is safe because `Ephemeron` is tracked to always point to a valid pointer
        // `inner_ptr`.
        unsafe { self.inner_ptr.get().as_ref().value().cloned() }
    }

    /// Checks if the [`Ephemeron`] has a value.
    pub fn has_value(&self) -> bool {
        // SAFETY: this is safe because `Ephemeron` is tracked to always point to a valid pointer
        // `inner_ptr`.
        unsafe { self.inner_ptr.get().as_ref().value().is_some() }
    }
}

impl<K: Trace + ?Sized, V: Trace> Ephemeron<K, V> {
    /// Creates a new `Ephemeron`.
    pub fn new(key: &Gc<K>, value: V) -> Self {
        // SAFETY: `value` comes from the stack and should be rooted, meaning unrooting
        // it to pass it to the underlying `EphemeronBox` is safe.
        unsafe {
            value.unroot();
        }
        // SAFETY: EphemeronBox is at least 2 bytes in size, and so its alignment is always a
        // multiple of 2.
        unsafe {
            Self {
                inner_ptr: Cell::new(
                    Rootable::new_unchecked(Allocator::alloc_ephemeron(EphemeronBox::new(
                        key, value,
                    )))
                    .rooted(),
                ),
            }
        }
    }

    /// Returns `true` if the two `Ephemeron`s point to the same allocation.
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        EphemeronBox::ptr_eq(this.inner(), other.inner())
    }

    fn is_rooted(&self) -> bool {
        self.inner_ptr.get().is_rooted()
    }

    fn root_ptr(&self) {
        self.inner_ptr.set(self.inner_ptr.get().rooted());
    }

    fn unroot_ptr(&self) {
        self.inner_ptr.set(self.inner_ptr.get().unrooted());
    }

    pub(crate) fn inner_ptr(&self) -> NonNull<EphemeronBox<K, V>> {
        assert!(finalizer_safe() || self.is_rooted());
        self.inner_ptr.get().as_ptr()
    }

    fn inner(&self) -> &EphemeronBox<K, V> {
        // SAFETY: Please see Gc::inner_ptr()
        unsafe { self.inner_ptr().as_ref() }
    }

    /// Constructs an `Ephemeron<K, V>` from a raw pointer.
    ///
    /// # Safety
    ///
    /// This function is unsafe because improper use may lead to memory corruption, double-free,
    /// or misbehaviour of the garbage collector.
    #[must_use]
    unsafe fn from_raw(ptr: NonNull<EphemeronBox<K, V>>) -> Self {
        // SAFETY: it is the caller's job to ensure the safety of this operation.
        unsafe {
            Self {
                inner_ptr: Cell::new(Rootable::new_unchecked(ptr).rooted()),
            }
        }
    }
}

impl<K: Trace + ?Sized, V: Trace> Finalize for Ephemeron<K, V> {}

// SAFETY: `Ephemeron`s trace implementation only marks its inner box because we want to stop
// tracing through weakly held pointers.
unsafe impl<K: Trace + ?Sized, V: Trace> Trace for Ephemeron<K, V> {
    unsafe fn trace(&self) {
        // SAFETY: We need to mark the inner box of the `Ephemeron` since it is reachable
        // from a root and this means it cannot be dropped.
        unsafe {
            self.inner().mark();
        }
    }

    unsafe fn root(&self) {
        assert!(!self.is_rooted(), "Can't double-root a Gc<T>");
        // Try to get inner before modifying our state. Inner may be
        // inaccessible due to this method being invoked during the sweeping
        // phase, and we don't want to modify our state before panicking.
        self.inner().root();
        self.root_ptr();
    }

    unsafe fn unroot(&self) {
        assert!(self.is_rooted(), "Can't double-unroot a Gc<T>");
        // Try to get inner before modifying our state. Inner may be
        // inaccessible due to this method being invoked during the sweeping
        // phase, and we don't want to modify our state before panicking.
        self.inner().unroot();
        self.unroot_ptr();
    }

    fn run_finalizer(&self) {
        Finalize::finalize(self);
    }
}

impl<K: Trace + ?Sized, V: Trace> Clone for Ephemeron<K, V> {
    fn clone(&self) -> Self {
        let ptr = self.inner_ptr();
        // SAFETY: since an `Ephemeron` is always valid, its `inner_ptr` must also be always a valid
        // pointer.
        unsafe {
            ptr.as_ref().root();
        }
        // SAFETY: `&self` is a valid Ephemeron pointer.
        unsafe { Self::from_raw(ptr) }
    }
}

impl<K: Trace + ?Sized, V: Trace> Drop for Ephemeron<K, V> {
    fn drop(&mut self) {
        // If this pointer was a root, we should unroot it.
        if self.is_rooted() {
            self.inner().unroot();
        }
    }
}
