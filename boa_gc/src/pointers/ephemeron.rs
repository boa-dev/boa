use crate::{
    finalizer_safe,
    internals::EphemeronBox,
    trace::{Finalize, Trace},
    Allocator, Gc, WeakGcHandle,
};
use std::{cell::Cell, ptr::NonNull, rc::Rc};

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
    inner_ptr: Cell<NonNull<EphemeronBox<K, V>>>,
    pub(crate) handle: Rc<WeakGcHandle>,
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
        let (handle, inner_ptr) = Allocator::alloc_ephemeron(EphemeronBox::new(key, value));
        Self {
            inner_ptr: Cell::new(inner_ptr),
            handle,
        }
    }

    /// Returns `true` if the two `Ephemeron`s point to the same allocation.
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        EphemeronBox::ptr_eq(this.inner(), other.inner())
    }

    pub(crate) fn inner_ptr(&self) -> NonNull<EphemeronBox<K, V>> {
        assert!(finalizer_safe());
        self.inner_ptr.get()
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
    unsafe fn from_raw(ptr: NonNull<EphemeronBox<K, V>>, handle: Rc<WeakGcHandle>) -> Self {
        Self {
            inner_ptr: Cell::new(ptr),
            handle,
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

    fn trace_non_roots(&self) {
        self.handle
            .is_non_root
            .set(self.handle.is_non_root.get() + 1);
    }

    fn run_finalizer(&self) {
        Finalize::finalize(self);
    }
}

impl<K: Trace + ?Sized, V: Trace> Clone for Ephemeron<K, V> {
    fn clone(&self) -> Self {
        let ptr = self.inner_ptr();
        // SAFETY: `&self` is a valid Ephemeron pointer.
        unsafe { Self::from_raw(ptr, self.handle.clone()) }
    }
}
