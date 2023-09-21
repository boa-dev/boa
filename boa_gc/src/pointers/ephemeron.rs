use crate::{
    finalizer_safe,
    internals::EphemeronBox,
    trace::{Finalize, Trace},
    Allocator, Gc,
};
use std::ptr::NonNull;

/// A key-value pair where the value becomes unaccesible when the key is garbage collected.
///
/// You can read more about ephemerons on:
/// - Racket's page about [**ephemerons**][eph], which gives a brief overview.
/// - Barry Hayes' paper ["_Ephemerons_: a new finalization mechanism"][acm] which explains the topic
/// in full detail.
///
///
/// [eph]: https://docs.racket-lang.org/reference/ephemerons.html
/// [acm]: https://dl.acm.org/doi/10.1145/263700.263733
#[derive(Debug)]
pub struct Ephemeron<K: Trace + 'static, V: Trace + 'static> {
    inner_ptr: NonNull<EphemeronBox<K, V>>,
}

impl<K: Trace, V: Trace + Clone> Ephemeron<K, V> {
    /// Gets the stored value of this `Ephemeron`, or `None` if the key was already garbage collected.
    ///
    /// This needs to return a clone of the value because holding a reference to it between
    /// garbage collection passes could drop the underlying allocation, causing an Use After Free.
    #[must_use]
    pub fn value(&self) -> Option<V> {
        // SAFETY: this is safe because `Ephemeron` is tracked to always point to a valid pointer
        // `inner_ptr`.
        unsafe { self.inner_ptr.as_ref().value().cloned() }
    }

    /// Checks if the [`Ephemeron`] has a value.
    #[must_use]
    pub fn has_value(&self) -> bool {
        // SAFETY: this is safe because `Ephemeron` is tracked to always point to a valid pointer
        // `inner_ptr`.
        unsafe { self.inner_ptr.as_ref().value().is_some() }
    }
}

impl<K: Trace, V: Trace> Ephemeron<K, V> {
    /// Creates a new `Ephemeron`.
    #[must_use]
    pub fn new(key: &Gc<K>, value: V) -> Self {
        let inner_ptr = Allocator::alloc_ephemeron(EphemeronBox::new(key, value));
        Self { inner_ptr }
    }

    /// Returns `true` if the two `Ephemeron`s point to the same allocation.
    #[must_use]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        EphemeronBox::ptr_eq(this.inner(), other.inner())
    }

    pub(crate) fn inner_ptr(&self) -> NonNull<EphemeronBox<K, V>> {
        assert!(finalizer_safe());
        self.inner_ptr
    }

    pub(crate) fn inner(&self) -> &EphemeronBox<K, V> {
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
    pub(crate) const unsafe fn from_raw(inner_ptr: NonNull<EphemeronBox<K, V>>) -> Self {
        Self { inner_ptr }
    }
}

impl<K: Trace, V: Trace> Finalize for Ephemeron<K, V> {
    fn finalize(&self) {
        // SAFETY: inner_ptr should be alive when calling finalize.
        // We don't call inner_ptr() to avoid overhead of calling finalizer_safe().
        unsafe {
            self.inner_ptr.as_ref().dec_ref_count();
        }
    }
}

// SAFETY: `Ephemeron`s trace implementation only marks its inner box because we want to stop
// tracing through weakly held pointers.
unsafe impl<K: Trace, V: Trace> Trace for Ephemeron<K, V> {
    unsafe fn trace(&self) {
        // SAFETY: We need to mark the inner box of the `Ephemeron` since it is reachable
        // from a root and this means it cannot be dropped.
        unsafe {
            self.inner().mark();
        }
    }

    fn trace_non_roots(&self) {
        self.inner().inc_non_root_count();
    }

    fn run_finalizer(&self) {
        Finalize::finalize(self);
    }
}

impl<K: Trace, V: Trace> Clone for Ephemeron<K, V> {
    fn clone(&self) -> Self {
        let ptr = self.inner_ptr();
        self.inner().inc_ref_count();
        // SAFETY: `&self` is a valid Ephemeron pointer.
        unsafe { Self::from_raw(ptr) }
    }
}

impl<K: Trace, V: Trace> Drop for Ephemeron<K, V> {
    fn drop(&mut self) {
        if finalizer_safe() {
            Finalize::finalize(self);
        }
    }
}
