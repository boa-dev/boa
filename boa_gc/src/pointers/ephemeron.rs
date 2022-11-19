use crate::{
    finalizer_safe,
    internals::EphemeronBox,
    trace::{Finalize, Trace},
    Allocator, Gc, GcBox, EPHEMERON_QUEUE,
};
use std::{cell::Cell, ptr::NonNull};

#[derive(Debug)]
/// A key-value pair where the value becomes unaccesible when the key is garbage collected.
///
/// See Racket's explanation on [**ephemerons**][eph] for a brief overview or read Barry Hayes'
/// [_Ephemerons_: a new finalization mechanism][acm].
///
///
/// [eph]: https://docs.racket-lang.org/reference/ephemerons.html
/// [acm]: https://dl.acm.org/doi/10.1145/263700.263733
pub struct Ephemeron<K: Trace + ?Sized + 'static, V: Trace + 'static> {
    inner_ptr: Cell<NonNull<GcBox<EphemeronBox<K, V>>>>,
}

impl<K: Trace + ?Sized, V: Trace> Ephemeron<K, V> {
    /// Creates a new `Ephemeron`.
    pub fn new(key: &Gc<K>, value: V) -> Self {
        Self {
            inner_ptr: Cell::new(Allocator::allocate(GcBox::new_weak(EphemeronBox::new(
                key, value,
            )))),
        }
    }
}

impl<K: Trace + ?Sized, V: Trace> Ephemeron<K, V> {
    #[inline]
    fn inner_ptr(&self) -> NonNull<GcBox<EphemeronBox<K, V>>> {
        self.inner_ptr.get()
    }

    #[inline]
    fn inner(&self) -> &GcBox<EphemeronBox<K, V>> {
        // SAFETY: GcBox<EphemeronBox<K,V>> must live until it is unrooted by Drop
        unsafe { &*self.inner_ptr().as_ptr() }
    }

    #[inline]
    /// Gets the weak key of this `Ephemeron`, or `None` if the key was already garbage
    /// collected.
    pub fn key(&self) -> Option<&K> {
        self.inner().value().key()
    }

    #[inline]
    /// Gets the stored value of this `Ephemeron`.
    pub fn value(&self) -> &V {
        self.inner().value().value()
    }

    #[inline]
    /// Gets a `Gc` for the stored key of this `Ephemeron`.
    pub fn upgrade_key(&self) -> Option<Gc<K>> {
        // SAFETY: ptr must be a valid pointer or None would have been returned.
        self.inner().value().inner_key_ptr().map(|ptr| unsafe {
            let inner_ptr = NonNull::new_unchecked(ptr);
            Gc::from_ptr(inner_ptr)
        })
    }
}

impl<K: Trace, V: Trace> Finalize for Ephemeron<K, V> {}

// SAFETY: Ephemerons trace implementation is standard for everything except `Trace::weak_trace()`,
// which pushes the GcBox<EphemeronBox<_>> onto the EphemeronQueue
unsafe impl<K: Trace, V: Trace> Trace for Ephemeron<K, V> {
    #[inline]
    unsafe fn trace(&self) {}

    // Push this Ephemeron's pointer onto the EphemeronQueue
    #[inline]
    unsafe fn weak_trace(&self) {
        EPHEMERON_QUEUE.with(|q| {
            let mut queue = q.take().expect("queue is initialized by weak_trace");
            queue.push(self.inner_ptr());
        });
    }

    #[inline]
    unsafe fn root(&self) {}

    #[inline]
    unsafe fn unroot(&self) {}

    #[inline]
    fn run_finalizer(&self) {
        Finalize::finalize(self);
    }
}

impl<K: Trace + ?Sized, V: Trace> Clone for Ephemeron<K, V> {
    #[inline]
    fn clone(&self) -> Self {
        // SAFETY: This is safe because the inner_ptr must live as long as it's roots.
        // Mismanagement of roots can cause inner_ptr to use after free or Undefined
        // Behavior.
        unsafe {
            let eph = Self {
                inner_ptr: Cell::new(NonNull::new_unchecked(self.inner_ptr().as_ptr())),
            };
            // Increment the Ephemeron's GcBox roots by 1
            self.inner().root_inner();
            eph
        }
    }
}

impl<K: Trace + ?Sized, V: Trace> Drop for Ephemeron<K, V> {
    #[inline]
    fn drop(&mut self) {
        // NOTE: We assert that this drop call is not a
        // drop from `Collector::dump` or `Collector::sweep`
        if finalizer_safe() {
            self.inner().unroot_inner();
        }
    }
}
