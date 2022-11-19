use crate::{finalizer_safe, trace::Trace, Finalize, Gc, GcBox};
use std::{cell::Cell, ptr::NonNull};

/// The inner allocation of an [`Ephemeron`][crate::Ephemeron] pointer.
pub(crate) struct EphemeronBox<K: Trace + ?Sized + 'static, V: Trace + ?Sized + 'static> {
    key: Cell<Option<NonNull<GcBox<K>>>>,
    value: V,
}

impl<K: Trace + ?Sized, V: Trace> EphemeronBox<K, V> {
    pub(crate) fn new(key: &Gc<K>, value: V) -> Self {
        Self {
            key: Cell::new(Some(key.inner_ptr())),
            value,
        }
    }
}

impl<K: Trace + ?Sized, V: Trace + ?Sized> EphemeronBox<K, V> {
    /// Checks if the key pointer is marked by Trace
    #[inline]
    pub(crate) fn is_marked(&self) -> bool {
        self.inner_key().map_or(false, GcBox::is_marked)
    }

    /// Returns some pointer to the `key`'s `GcBox` or None
    /// # Panics
    /// This method will panic if called while the garbage collector is dropping.
    #[inline]
    pub(crate) fn inner_key_ptr(&self) -> Option<*mut GcBox<K>> {
        assert!(finalizer_safe());
        self.key.get().map(NonNull::as_ptr)
    }

    /// Returns some reference to `key`'s `GcBox` or None
    #[inline]
    pub(crate) fn inner_key(&self) -> Option<&GcBox<K>> {
        // SAFETY: This is safe as `EphemeronBox::inner_key_ptr()` will
        // fetch either a live `GcBox` or None. The value of `key` is set
        // to None in the case where `EphemeronBox` and `key`'s `GcBox`
        // entered into `Collector::sweep()` as unmarked.
        unsafe { self.inner_key_ptr().map(|inner_key| &*inner_key) }
    }

    /// Returns a reference to the value of `key`'s `GcBox`
    #[inline]
    pub(crate) fn key(&self) -> Option<&K> {
        self.inner_key().map(GcBox::value)
    }

    /// Returns a reference to `value`
    #[inline]
    pub(crate) const fn value(&self) -> &V {
        &self.value
    }

    /// Calls [`Trace::weak_trace()`][crate::Trace] on key
    #[inline]
    fn weak_trace_key(&self) {
        if let Some(key) = self.inner_key() {
            key.weak_trace_inner();
        }
    }

    /// Calls [`Trace::weak_trace()`][crate::Trace] on value
    #[inline]
    fn weak_trace_value(&self) {
        // SAFETY: Value is a sized element that must implement trace. The
        // operation is safe as EphemeronBox owns value and `Trace::weak_trace`
        // must be implemented on it
        unsafe {
            self.value().weak_trace();
        }
    }
}

// `EphemeronBox`'s Finalize is special in that if it is determined to be unreachable
// and therefore so has the `GcBox` that `key`stores the pointer to, then we set `key`
// to None to guarantee that we do not access freed memory.
impl<K: Trace + ?Sized, V: Trace + ?Sized> Finalize for EphemeronBox<K, V> {
    #[inline]
    fn finalize(&self) {
        self.key.set(None);
    }
}

// SAFETY: EphemeronBox implements primarly two methods of trace `Trace::is_marked_ephemeron`
// to determine whether the key field is stored and `Trace::weak_trace` which continues the `Trace::weak_trace()`
// into `key` and `value`.
unsafe impl<K: Trace + ?Sized, V: Trace + ?Sized> Trace for EphemeronBox<K, V> {
    #[inline]
    unsafe fn trace(&self) {
        /* An ephemeron is never traced with Phase One Trace */
    }

    /// Checks if the `key`'s `GcBox` has been marked by `Trace::trace()` or `Trace::weak_trace`.
    #[inline]
    fn is_marked_ephemeron(&self) -> bool {
        self.is_marked()
    }

    /// Checks if this `EphemeronBox` has already been determined reachable. If so, continue to trace
    /// value in `key` and `value`.
    #[inline]
    unsafe fn weak_trace(&self) {
        if self.is_marked() {
            self.weak_trace_key();
            self.weak_trace_value();
        }
    }

    // EphemeronBox does not implement root.
    #[inline]
    unsafe fn root(&self) {}

    // EphemeronBox does not implement unroot
    #[inline]
    unsafe fn unroot(&self) {}

    // An `EphemeronBox`'s key is set to None once it has been finalized.
    //
    // NOTE: while it is possible for the `key`'s pointer value to be
    // resurrected, we should still consider the finalize the ephemeron
    // box and set the `key` to None.
    #[inline]
    fn run_finalizer(&self) {
        Finalize::finalize(self);
    }
}
