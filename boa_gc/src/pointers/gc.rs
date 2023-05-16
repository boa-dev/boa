use crate::{
    finalizer_safe,
    internals::{GcBox, GcBoxHeader},
    trace::{Finalize, Trace},
    Allocator, Ephemeron, WeakGc, BOA_GC,
};
use std::{
    cell::Cell,
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem,
    ops::Deref,
    ptr::{self, NonNull},
    rc::Rc,
};

use super::rootable::Rootable;

/// A garbage-collected pointer type over an immutable value.
pub struct Gc<T: Trace + ?Sized + 'static> {
    pub(crate) inner_ptr: Cell<Rootable<GcBox<T>>>,
    pub(crate) marker: PhantomData<Rc<T>>,
}

impl<T: Trace> Gc<T> {
    /// Constructs a new `Gc<T>` with the given value.
    pub fn new(value: T) -> Self {
        // Create GcBox and allocate it to heap.
        //
        // Note: Allocator can cause Collector to run
        let inner_ptr = Allocator::alloc_gc(GcBox::new(value));

        // SAFETY: inner_ptr was just allocated, so it must be a valid value that implements [`Trace`]
        unsafe { (*inner_ptr.as_ptr()).value().unroot() }

        // SAFETY: inner_ptr is 2-byte aligned.
        let inner_ptr = unsafe { Rootable::new_unchecked(inner_ptr) };

        Self {
            inner_ptr: Cell::new(inner_ptr.rooted()),
            marker: PhantomData,
        }
    }

    /// Constructs a new `Gc<T>` while giving you a `WeakGc<T>` to the allocation,
    /// to allow you to construct a `T` which holds a weak pointer to itself.
    ///
    /// Since the new `Gc<T>` is not fully-constructed until `Gc<T>::new_cyclic`
    /// returns, calling [`WeakGc::upgrade`] on the weak reference inside the closure will
    /// fail and result in a `None` value.
    pub fn new_cyclic<F>(data_fn: F) -> Self
    where
        F: FnOnce(&WeakGc<T>) -> T,
    {
        // Create GcBox and allocate it to heap.
        let inner_ptr = BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();

            Allocator::manage_state(&mut gc);

            let header = GcBoxHeader::new();

            header.next.set(gc.strong_start.take());

            // Safety: value cannot be a null pointer, since `Box` cannot return null pointers.
            unsafe {
                NonNull::new_unchecked(Box::into_raw(Box::new(GcBox {
                    header,
                    value: mem::MaybeUninit::<T>::uninit(),
                })))
            }
        });

        let init_ptr: NonNull<GcBox<T>> = inner_ptr.cast();

        let weak = WeakGc::from(Ephemeron::new_empty());

        let data = data_fn(&weak);

        // SAFETY: `inner_ptr` has been allocated above, so making writes to `init_ptr` is safe.
        let strong = unsafe {
            let inner = init_ptr.as_ptr();
            ptr::write(ptr::addr_of_mut!((*inner).value), data);

            // `strong` must be a valid value that implements [`Trace`].
            (*inner).value.unroot();

            // `init_ptr` is initialized and its contents are unrooted, making this operation safe.
            Self::from_raw(init_ptr)
        };

        BOA_GC.with(|st| {
            let mut gc = st.borrow_mut();
            let erased: NonNull<GcBox<dyn Trace>> = init_ptr;

            gc.strong_start.set(Some(erased));
            gc.runtime.bytes_allocated += mem::size_of::<GcBox<T>>();
        });

        // SAFETY: `init_ptr` is initialized and its contents are unrooted, making this operation
        // safe.
        unsafe {
            weak.inner().inner().init(&strong, Self::from_raw(init_ptr));
        }

        strong
    }

    /// Consumes the `Gc`, returning a wrapped raw pointer.
    ///
    /// To avoid a memory leak, the pointer must be converted back to a `Gc` using [`Gc::from_raw`].
    #[allow(clippy::use_self)]
    pub fn into_raw(this: Gc<T>) -> NonNull<GcBox<T>> {
        let ptr = this.inner_ptr();
        std::mem::forget(this);
        ptr
    }
}

impl<T: Trace + ?Sized> Gc<T> {
    /// Returns `true` if the two `Gc`s point to the same allocation.
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
    pub unsafe fn from_raw(ptr: NonNull<GcBox<T>>) -> Self {
        // SAFETY: it is the caller's job to ensure the safety of this operation.
        unsafe {
            Self {
                inner_ptr: Cell::new(Rootable::new_unchecked(ptr).rooted()),
                marker: PhantomData,
            }
        }
    }
}

impl<T: Trace + ?Sized> Gc<T> {
    fn is_rooted(&self) -> bool {
        self.inner_ptr.get().is_rooted()
    }

    fn root_ptr(&self) {
        self.inner_ptr.set(self.inner_ptr.get().rooted());
    }

    fn unroot_ptr(&self) {
        self.inner_ptr.set(self.inner_ptr.get().unrooted());
    }

    pub(crate) fn inner_ptr(&self) -> NonNull<GcBox<T>> {
        assert!(finalizer_safe() || self.is_rooted());
        self.inner_ptr.get().as_ptr()
    }

    fn inner(&self) -> &GcBox<T> {
        // SAFETY: Please see Gc::inner_ptr()
        unsafe { self.inner_ptr().as_ref() }
    }
}

impl<T: Trace + ?Sized> Finalize for Gc<T> {}

// SAFETY: `Gc` maintains it's own rootedness and implements all methods of
// Trace. It is not possible to root an already rooted `Gc` and vice versa.
unsafe impl<T: Trace + ?Sized> Trace for Gc<T> {
    unsafe fn trace(&self) {
        // SAFETY: Inner must be live and allocated GcBox.
        unsafe {
            self.inner().mark_and_trace();
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

impl<T: Trace + ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        let ptr = self.inner_ptr();
        // SAFETY: since a `Gc` is always valid, its `inner_ptr` must also be always a valid pointer.
        unsafe {
            ptr.as_ref().root();
        }
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
        // If this pointer was a root, we should unroot it.
        if self.is_rooted() {
            self.inner().unroot();
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
