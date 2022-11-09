use crate::{
    finalizer_safe,
    internals::EphemeronBox,
    trace::{Finalize, Trace},
    Allocator, Gc, GcBox, EPHEMERON_QUEUE,
};
use std::cell::Cell;
use std::ptr::NonNull;

#[derive(Debug)]
/// A key-value pair where the value becomes unaccesible when the key is garbage collected.
///
/// See Racket's explanation on [**ephemerons**][eph] for a more detailed explanation.
///
/// [eph]: https://docs.racket-lang.org/reference/ephemerons.html
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
        assert!(finalizer_safe());

        self.inner_ptr.get()
    }

    #[inline]
    fn inner(&self) -> &GcBox<EphemeronBox<K, V>> {
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
}

impl<K: Trace, V: Trace> Finalize for Ephemeron<K, V> {}

unsafe impl<K: Trace, V: Trace> Trace for Ephemeron<K, V> {
    #[inline]
    unsafe fn trace(&self) {}

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

impl<K: Trace + ?Sized, V: Trace> Drop for Ephemeron<K, V> {
    #[inline]
    fn drop(&mut self) {
        unsafe { self.inner().unroot_inner() }
    }
}
