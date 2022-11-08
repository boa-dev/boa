use crate::{
    finalizer_safe,
    internals::EphemeronBox,
    trace::{Finalize, Trace},
    Gc, GcAlloc, GcBox, EPHEMERON_QUEUE,
};
use std::cell::Cell;
use std::ptr::NonNull;

pub struct WeakGc<T: Trace + ?Sized + 'static> {
    inner_ptr: Cell<NonNull<GcBox<EphemeronBox<T, ()>>>>,
}

impl<T: Trace> WeakGc<T> {
    pub fn new(value: &Gc<T>) -> Self {
        let weak_box = GcAlloc::new_weak_box(value);
        unsafe {
            Self {
                inner_ptr: Cell::new(NonNull::new_unchecked(weak_box.as_ptr())),
            }
        }
    }
}

impl<T: Trace + ?Sized> WeakGc<T> {
    #[inline]
    fn inner_ptr(&self) -> *mut GcBox<EphemeronBox<T, ()>> {
        assert!(finalizer_safe());

        self.inner_ptr.get().as_ptr()
    }

    #[inline]
    fn inner(&self) -> &GcBox<EphemeronBox<T, ()>> {
        unsafe { &*self.inner_ptr() }
    }

    #[inline]
    pub fn value(&self) -> Option<&T> {
        self.inner().value().key()
    }
}

impl<T: Trace> Finalize for WeakGc<T> {}

unsafe impl<T: Trace> Trace for WeakGc<T> {
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
