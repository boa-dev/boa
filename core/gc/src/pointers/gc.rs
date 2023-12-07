use crate::{
    finalizer_safe,
    internals::{EphemeronBox, GcBox},
    trace::{Finalize, Trace},
    Allocator, Ephemeron, GcErasedPointer, Tracer, WeakGc,
};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
    rc::Rc,
};

/// Zero sized struct that is used to ensure that we do not call trace methods,
/// call its finalization method or drop it.
///
/// This can only happen if we are accessing a [`GcErasedPointer`] directly which is a bug.
/// Panics if any of it's methods are called.
///
/// Note: Accessing the [`crate::internals::GcHeader`] of [`GcErasedPointer`] is fine.
pub(crate) struct NonTraceable(());

impl Finalize for NonTraceable {
    fn finalize(&self) {
        unreachable!()
    }
}

unsafe impl Trace for NonTraceable {
    unsafe fn trace(&self, _tracer: &mut Tracer) {
        unreachable!()
    }
    unsafe fn trace_non_roots(&self) {
        unreachable!()
    }
    fn run_finalizer(&self) {
        unreachable!()
    }
}

impl Drop for NonTraceable {
    fn drop(&mut self) {
        unreachable!()
    }
}

/// A garbage-collected pointer type over an immutable value.
pub struct Gc<T: Trace + ?Sized + 'static> {
    pub(crate) inner_ptr: NonNull<GcBox<T>>,
    pub(crate) marker: PhantomData<Rc<T>>,
}

impl<T: Trace> Gc<T> {
    /// Constructs a new `Gc<T>` with the given value.
    #[must_use]
    pub fn new(value: T) -> Self {
        // Create GcBox and allocate it to heap.
        //
        // Note: Allocator can cause Collector to run
        let inner_ptr = Allocator::alloc_gc(GcBox::new(value));

        Self {
            inner_ptr,
            marker: PhantomData,
        }
    }

    /// Constructs a new `Gc<T>` while giving you a `WeakGc<T>` to the allocation, to allow
    /// constructing a T which holds a weak pointer to itself.
    ///
    /// Since the new `Gc<T>` is not fully-constructed until `Gc<T>::new_cyclic` returns, calling
    /// [`upgrade`][WeakGc::upgrade]  on the weak reference inside the closure will fail and result
    /// in a `None` value.
    #[must_use]
    pub fn new_cyclic<F>(data_fn: F) -> Self
    where
        F: FnOnce(&WeakGc<T>) -> T,
    {
        // SAFETY: The newly allocated ephemeron is only live here, meaning `Ephemeron` is the
        // sole owner of the allocation after passing it to `from_raw`, making this operation safe.
        let weak = unsafe {
            Ephemeron::from_raw(Allocator::alloc_ephemeron(EphemeronBox::new_empty())).into()
        };

        let gc = Self::new(data_fn(&weak));

        // SAFETY:
        // - `as_mut`: `weak` is properly initialized by `alloc_ephemeron` and cannot escape the
        //   `unsafe` block.
        // - `set_kv`: `weak` is a newly created `EphemeronBox`, meaning it isn't possible to
        //   collect it since `weak` is still live.
        unsafe { weak.inner().inner_ptr().as_mut().set(&gc, ()) }

        gc
    }

    /// Consumes the `Gc`, returning a wrapped raw pointer.
    ///
    /// To avoid a memory leak, the pointer must be converted back to a `Gc` using [`Gc::from_raw`].
    #[must_use]
    pub fn into_raw(this: Self) -> NonNull<GcBox<T>> {
        let ptr = this.inner_ptr();
        std::mem::forget(this);
        ptr
    }
}

impl<T: Trace + ?Sized> Gc<T> {
    /// Returns `true` if the two `Gc`s point to the same allocation.
    #[must_use]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        GcBox::ptr_eq(this.inner(), other.inner())
    }

    /// Constructs a `Gc<T>` from a raw pointer.
    ///
    /// The raw pointer must have been returned by a previous call to [`Gc<U>::into_raw`][Gc::into_raw]
    /// where `U` must have the same size and alignment as `T`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because improper use may lead to memory corruption, double-free,
    /// or misbehaviour of the garbage collector.
    #[must_use]
    pub const unsafe fn from_raw(inner_ptr: NonNull<GcBox<T>>) -> Self {
        Self {
            inner_ptr,
            marker: PhantomData,
        }
    }

    pub(crate) fn as_erased(&self) -> GcErasedPointer {
        self.inner_ptr.cast()
    }
}

impl<T: Trace + ?Sized> Gc<T> {
    pub(crate) fn inner_ptr(&self) -> NonNull<GcBox<T>> {
        assert!(finalizer_safe());
        self.inner_ptr
    }

    fn inner(&self) -> &GcBox<T> {
        // SAFETY: Please see Gc::inner_ptr()
        unsafe { self.inner_ptr().as_ref() }
    }
}

impl<T: Trace + ?Sized> Finalize for Gc<T> {
    fn finalize(&self) {
        // SAFETY: inner_ptr should be alive when calling finalize.
        // We don't call inner_ptr() to avoid overhead of calling finalizer_safe().
        unsafe {
            self.inner_ptr.as_ref().dec_ref_count();
        }
    }
}

// SAFETY: `Gc` maintains it's own rootedness and implements all methods of
// Trace. It is not possible to root an already rooted `Gc` and vice versa.
unsafe impl<T: Trace + ?Sized> Trace for Gc<T> {
    unsafe fn trace(&self, tracer: &mut Tracer) {
        tracer.enqueue(self.as_erased());
    }

    unsafe fn trace_non_roots(&self) {
        self.inner().inc_non_root_count();
    }

    fn run_finalizer(&self) {
        Finalize::finalize(self);
    }
}

impl<T: Trace + ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        let ptr = self.inner_ptr();
        self.inner().inc_ref_count();
        // SAFETY: though `ptr` doesn't come from a `into_raw` call, it essentially does the same,
        // but it skips the call to `std::mem::forget` since we have a reference instead of an owned
        // value.
        unsafe { Self::from_raw(ptr) }
    }
}

impl<T: Trace + ?Sized> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner().value()
    }
}

impl<T: Trace + ?Sized> Drop for Gc<T> {
    fn drop(&mut self) {
        if finalizer_safe() {
            Finalize::finalize(self);
        }
    }
}

impl<T: Trace + Default> Default for Gc<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[allow(clippy::inline_always)]
impl<T: Trace + ?Sized + PartialEq> PartialEq for Gc<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Trace + ?Sized + Eq> Eq for Gc<T> {}

#[allow(clippy::inline_always)]
impl<T: Trace + ?Sized + PartialOrd> PartialOrd for Gc<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (**self).partial_cmp(&**other)
    }

    #[inline(always)]
    fn lt(&self, other: &Self) -> bool {
        **self < **other
    }

    #[inline(always)]
    fn le(&self, other: &Self) -> bool {
        **self <= **other
    }

    #[inline(always)]
    fn gt(&self, other: &Self) -> bool {
        **self > **other
    }

    #[inline(always)]
    fn ge(&self, other: &Self) -> bool {
        **self >= **other
    }
}

impl<T: Trace + ?Sized + Ord> Ord for Gc<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

impl<T: Trace + ?Sized + Hash> Hash for Gc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: Trace + ?Sized + Display> Display for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T: Trace + ?Sized + Debug> Debug for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: Trace + ?Sized> fmt::Pointer for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.inner(), f)
    }
}

impl<T: Trace + ?Sized> std::borrow::Borrow<T> for Gc<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: Trace + ?Sized> AsRef<T> for Gc<T> {
    fn as_ref(&self) -> &T {
        self
    }
}
