use crate::{
    finalizer_safe,
    internals::Ephemeron,
    trace::{Finalize, Trace},
    GcBox, EPHEMERON_QUEUE,
};
use std::cell::Cell;
use std::ptr::NonNull;

pub struct WeakPair<K: Trace + ?Sized + 'static, V: Trace + ?Sized + 'static> {
    inner_ptr: Cell<NonNull<GcBox<Ephemeron<K, V>>>>,
}

impl<K: Trace + ?Sized, V: Trace + ?Sized> WeakPair<K, V> {
    pub fn new(value: NonNull<GcBox<Ephemeron<K, V>>>) -> Self {
        unsafe {
            Self {
                inner_ptr: Cell::new(NonNull::new_unchecked(value.as_ptr())),
            }
        }
    }
}

impl<K: Trace + ?Sized, V: Trace> WeakPair<K, V> {
    #[inline]
    fn inner_ptr(&self) -> *mut GcBox<Ephemeron<K, V>> {
        assert!(finalizer_safe());

        self.inner_ptr.get().as_ptr()
    }

    #[inline]
    pub fn inner(&self) -> &GcBox<Ephemeron<K, V>> {
        unsafe { &*self.inner_ptr() }
    }

    #[inline]
    pub fn key(&self) -> Option<&K> {
        self.inner().value().key()
    }

    #[inline]
    pub fn value(&self) -> &V {
        self.inner().value().value()
    }
}

impl<K: Trace, V: Trace> Finalize for WeakPair<K, V> {}

unsafe impl<K: Trace, V: Trace> Trace for WeakPair<K, V> {
    #[inline]
    unsafe fn trace(&self) {}

    #[inline]
    unsafe fn is_marked_ephemeron(&self) -> bool {
        false
    }

    #[inline]
    unsafe fn weak_trace(&self) {
        EPHEMERON_QUEUE.with(|q| {
            let mut queue = q.take().expect("queue is initialized by weak_trace");
            queue.push(NonNull::new_unchecked(self.inner_ptr()))
        })
    }

    #[inline]
    unsafe fn root(&self) {}

    #[inline]
    unsafe fn unroot(&self) {}

    #[inline]
    fn run_finalizer(&self) {
        Finalize::finalize(self)
    }
}
