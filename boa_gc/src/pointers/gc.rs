use std::cell::Cell;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::{self, NonNull};
use std::rc::Rc;

use crate::internals::GcBox;
use crate::trace::{Finalize, Trace};
use crate::{finalizer_safe, Allocator};

pub(crate) unsafe fn set_data_ptr<T: ?Sized, U>(mut ptr: *mut T, data: *mut U) -> *mut T {
    ptr::write(&mut ptr as *mut _ as *mut *mut u8, data as *mut u8);
    ptr
}

/// A garbage-collected pointer type over an immutable value.
pub struct Gc<T: Trace + ?Sized + 'static> {
    inner_ptr: Cell<NonNull<GcBox<T>>>,
    marker: PhantomData<Rc<T>>,
}

impl<T: Trace> Gc<T> {
    /// Constructs a new `Gc<T>` with the given value.
    pub fn new(value: T) -> Self {
        unsafe {
            value.unroot();
        }
        let inner_ptr = Allocator::new(GcBox::new(value));
        let gc = Self {
            inner_ptr: Cell::new(inner_ptr),
            marker: PhantomData,
        };
        unsafe { gc.set_root() };
        gc
    }
}

impl<T: Trace + ?Sized> Gc<T> {
    /// Returns `true` if the two `Gc`s point to the same allocation.
    pub fn ptr_eq(this: &Gc<T>, other: &Gc<T>) -> bool {
        GcBox::ptr_eq(this.inner(), other.inner())
    }
}

/// Returns the given pointer with its root bit cleared.
pub(crate) unsafe fn clear_root_bit<T: ?Sized + Trace>(
    ptr: NonNull<GcBox<T>>,
) -> NonNull<GcBox<T>> {
    let ptr = ptr.as_ptr();
    let data = ptr as *mut u8;
    let addr = data as isize;
    let ptr = set_data_ptr(ptr, data.wrapping_offset((addr & !1) - addr));
    NonNull::new_unchecked(ptr)
}

impl<T: Trace + ?Sized> Gc<T> {
    fn rooted(&self) -> bool {
        self.inner_ptr.get().as_ptr() as *mut u8 as usize & 1 != 0
    }

    unsafe fn set_root(&self) {
        let ptr = self.inner_ptr.get().as_ptr();
        let data = ptr as *mut u8;
        let addr = data as isize;
        let ptr = set_data_ptr(ptr, data.wrapping_offset((addr | 1) - addr));
        self.inner_ptr.set(NonNull::new_unchecked(ptr));
    }

    unsafe fn clear_root(&self) {
        self.inner_ptr.set(clear_root_bit(self.inner_ptr.get()));
    }

    #[inline]
    pub(crate) fn inner_ptr(&self) -> NonNull<GcBox<T>> {
        assert!(finalizer_safe());
        unsafe { clear_root_bit(self.inner_ptr.get()) }
    }

    #[inline]
    fn inner(&self) -> &GcBox<T> {
        unsafe { self.inner_ptr().as_ref() }
    }
}

impl<T: Trace + ?Sized> Finalize for Gc<T> {}

unsafe impl<T: Trace + ?Sized> Trace for Gc<T> {
    #[inline]
    unsafe fn trace(&self) {
        self.inner().trace_inner();
    }

    #[inline]
    unsafe fn weak_trace(&self) {
        self.inner().weak_trace_inner();
    }

    #[inline]
    unsafe fn root(&self) {
        assert!(!self.rooted(), "Can't double-root a Gc<T>");
        // Try to get inner before modifying our state. Inner may be
        // inaccessible due to this method being invoked during the sweeping
        // phase, and we don't want to modify our state before panicking.
        self.inner().root_inner();

        self.set_root();
    }

    #[inline]
    unsafe fn unroot(&self) {
        assert!(self.rooted(), "Can't double-unroot a Gc<T>");
        // Try to get inner before modifying our state. Inner may be
        // inaccessible due to this method being invoked during the sweeping
        // phase, and we don't want to modify our state before panicking.
        self.inner().unroot_inner();

        self.clear_root();
    }

    #[inline]
    fn run_finalizer(&self) {
        Finalize::finalize(self);
    }
}

impl<T: Trace + ?Sized> Clone for Gc<T> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            self.inner().root_inner();
            let gc = Gc {
                inner_ptr: Cell::new(self.inner_ptr.get()),
                marker: PhantomData,
            };
            gc.set_root();
            gc
        }
    }
}

impl<T: Trace + ?Sized> Deref for Gc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.inner().value()
    }
}

impl<T: Trace + ?Sized> Drop for Gc<T> {
    #[inline]
    fn drop(&mut self) {
        // If this pointer was a root, we should unroot it.
        if self.rooted() {
            unsafe {
                self.inner().unroot_inner();
            }
        }
    }
}

impl<T: Trace + Default> Default for Gc<T> {
    #[inline]
    fn default() -> Self {
        Gc::new(Default::default())
    }
}

impl<T: Trace + ?Sized + PartialEq> PartialEq for Gc<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Trace + ?Sized + Eq> Eq for Gc<T> {}

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
    #[inline]
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
        &**self
    }
}

impl<T: Trace + ?Sized> std::convert::AsRef<T> for Gc<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}
