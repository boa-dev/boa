//! A garbage collected cell implementation
use std::cell::{Cell, UnsafeCell};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use crate::{
    internals::borrow_flag::{BorrowFlag, BorrowState, BORROWFLAG_INIT},
    trace::{Finalize, Trace},
};

/// A mutable memory location with dynamically checked borrow rules
/// that can be used inside of a garbage-collected pointer.
///
/// This object is a `RefCell` that can be used inside of a `Gc<T>`.
pub struct GcCell<T: ?Sized + 'static> {
    pub(crate) flags: Cell<BorrowFlag>,
    pub(crate) cell: UnsafeCell<T>,
}

impl<T: Trace> GcCell<T> {
    /// Creates a new `GcCell` containing `value`.
    #[inline]
    pub fn new(value: T) -> Self {
        GcCell {
            flags: Cell::new(BORROWFLAG_INIT),
            cell: UnsafeCell::new(value),
        }
    }

    /// Consumes the `GcCell`, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.cell.into_inner()
    }
}

impl<T: Trace + ?Sized> GcCell<T> {
    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `GcCellRef` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    #[inline]
    pub fn borrow(&self) -> GcCellRef<'_, T> {
        match self.try_borrow() {
            Ok(value) => value,
            Err(e) => panic!("{}", e),
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `GcCellRefMut` exits scope.
    /// The value cannot be borrowed while this borrow is active.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    #[inline]
    pub fn borrow_mut(&self) -> GcCellRefMut<'_, T> {
        match self.try_borrow_mut() {
            Ok(value) => value,
            Err(e) => panic!("{}", e),
        }
    }

    /// Immutably borrows the wrapped value, returning an error if the value is currently mutably
    /// borrowed.
    ///
    /// The borrow lasts until the returned `GcCellRef` exits scope. Multiple immutable borrows can be
    /// taken out at the same time.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    pub fn try_borrow(&self) -> Result<GcCellRef<'_, T>, BorrowError> {
        if self.flags.get().borrowed() == BorrowState::Writing {
            return Err(BorrowError);
        }
        self.flags.set(self.flags.get().add_reading());

        // This will fail if the borrow count overflows, which shouldn't happen,
        // but let's be safe
        assert!(self.flags.get().borrowed() == BorrowState::Reading);

        unsafe {
            Ok(GcCellRef {
                flags: &self.flags,
                value: &*self.cell.get(),
            })
        }
    }

    /// Mutably borrows the wrapped value, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts until the returned `GcCellRefMut` exits scope.
    /// The value cannot be borrowed while this borrow is active.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    pub fn try_borrow_mut(&self) -> Result<GcCellRefMut<'_, T>, BorrowMutError> {
        if self.flags.get().borrowed() != BorrowState::Unused {
            return Err(BorrowMutError);
        }
        self.flags.set(self.flags.get().set_writing());

        unsafe {
            // Force the val_ref's contents to be rooted for the duration of the
            // mutable borrow
            if !self.flags.get().rooted() {
                (*self.cell.get()).root();
            }

            Ok(GcCellRefMut {
                gc_cell: self,
                value: &mut *self.cell.get(),
            })
        }
    }
}

/// An error returned by [`GcCell::try_borrow`](struct.GcCell.html#method.try_borrow).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct BorrowError;

impl std::fmt::Display for BorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt("GcCell<T> already mutably borrowed", f)
    }
}

/// An error returned by [`GcCell::try_borrow_mut`](struct.GcCell.html#method.try_borrow_mut).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct BorrowMutError;

impl std::fmt::Display for BorrowMutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt("GcCell<T> already borrowed", f)
    }
}

impl<T: Trace + ?Sized> Finalize for GcCell<T> {}

unsafe impl<T: Trace + ?Sized> Trace for GcCell<T> {
    #[inline]
    unsafe fn trace(&self) {
        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => (*self.cell.get()).trace(),
        }
    }

    #[inline]
    unsafe fn is_marked_ephemeron(&self) -> bool {
        false
    }

    #[inline]
    unsafe fn weak_trace(&self) {
        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => (*self.cell.get()).weak_trace(),
        }
    }

    unsafe fn root(&self) {
        assert!(!self.flags.get().rooted(), "Can't root a GcCell twice!");
        self.flags.set(self.flags.get().set_rooted(true));

        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => (*self.cell.get()).root(),
        }
    }

    #[inline]
    unsafe fn unroot(&self) {
        assert!(self.flags.get().rooted(), "Can't unroot a GcCell twice!");
        self.flags.set(self.flags.get().set_rooted(false));

        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => (*self.cell.get()).unroot(),
        }
    }

    #[inline]
    fn run_finalizer(&self) {
        Finalize::finalize(self);
        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => unsafe { (*self.cell.get()).run_finalizer() },
        }
    }
}

/// A wrapper type for an immutably borrowed value from a `GcCell<T>`.
pub struct GcCellRef<'a, T: ?Sized + 'static> {
    pub(crate) flags: &'a Cell<BorrowFlag>,
    pub(crate) value: &'a T,
}

impl<'a, T: ?Sized> GcCellRef<'a, T> {
    /// Copies a `GcCellRef`.
    ///
    /// The `GcCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `GcCellRef::clone(...)`. A `Clone` implementation or a method
    /// would interfere with the use of `c.borrow().clone()` to clone
    /// the contents of a `GcCell`.
    #[inline]
    pub fn clone(orig: &GcCellRef<'a, T>) -> GcCellRef<'a, T> {
        orig.flags.set(orig.flags.get().add_reading());
        GcCellRef {
            flags: orig.flags,
            value: orig.value,
        }
    }

    /// Makes a new `GcCellRef` from a component of the borrowed data.
    ///
    /// The `GcCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as `GcCellRef::map(...)`.
    /// A method would interfere with methods of the same name on the contents
    /// of a `GcCellRef` used through `Deref`.
    #[inline]
    pub fn map<U, F>(orig: Self, f: F) -> GcCellRef<'a, U>
    where
        U: ?Sized,
        F: FnOnce(&T) -> &U,
    {
        let ret = GcCellRef {
            flags: orig.flags,
            value: f(orig.value),
        };

        // We have to tell the compiler not to call the destructor of GcCellRef,
        // because it will update the borrow flags.
        std::mem::forget(orig);

        ret
    }

    /// Splits a `GcCellRef` into multiple `GcCellRef`s for different components of the borrowed data.
    ///
    /// The `GcCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as GcCellRef::map_split(...).
    /// A method would interfere with methods of the same name on the contents of a `GcCellRef` used through `Deref`.
    #[inline]
    pub fn map_split<U, V, F>(orig: Self, f: F) -> (GcCellRef<'a, U>, GcCellRef<'a, V>)
    where
        U: ?Sized,
        V: ?Sized,
        F: FnOnce(&T) -> (&U, &V),
    {
        let (a, b) = f(orig.value);

        orig.flags.set(orig.flags.get().add_reading());

        let ret = (
            GcCellRef {
                flags: orig.flags,
                value: a,
            },
            GcCellRef {
                flags: orig.flags,
                value: b,
            },
        );

        // We have to tell the compiler not to call the destructor of GcCellRef,
        // because it will update the borrow flags.
        std::mem::forget(orig);

        ret
    }
}

impl<'a, T: ?Sized> Deref for GcCellRef<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<'a, T: ?Sized> Drop for GcCellRef<'a, T> {
    fn drop(&mut self) {
        debug_assert!(self.flags.get().borrowed() == BorrowState::Reading);
        self.flags.set(self.flags.get().sub_reading());
    }
}

impl<'a, T: ?Sized + Debug> Debug for GcCellRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + Display> Display for GcCellRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

/// A wrapper type for a mutably borrowed value from a `GcCell<T>`.
pub struct GcCellRefMut<'a, T: Trace + ?Sized + 'static, U: ?Sized = T> {
    pub(crate) gc_cell: &'a GcCell<T>,
    pub(crate) value: &'a mut U,
}

impl<'a, T: Trace + ?Sized, U: ?Sized> GcCellRefMut<'a, T, U> {
    /// Makes a new `GcCellRefMut` for a component of the borrowed data, e.g., an enum
    /// variant.
    ///
    /// The `GcCellRefMut` is already mutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `GcCellRefMut::map(...)`. A method would interfere with methods of the same
    /// name on the contents of a `GcCell` used through `Deref`.
    #[inline]
    pub fn map<V, F>(orig: Self, f: F) -> GcCellRefMut<'a, T, V>
    where
        V: ?Sized,
        F: FnOnce(&mut U) -> &mut V,
    {
        let value = unsafe { &mut *(orig.value as *mut U) };

        let ret = GcCellRefMut {
            gc_cell: orig.gc_cell,
            value: f(value),
        };

        // We have to tell the compiler not to call the destructor of GcCellRefMut,
        // because it will update the borrow flags.
        std::mem::forget(orig);

        ret
    }
}

impl<'a, T: Trace + ?Sized, U: ?Sized> Deref for GcCellRefMut<'a, T, U> {
    type Target = U;

    #[inline]
    fn deref(&self) -> &U {
        self.value
    }
}

impl<'a, T: Trace + ?Sized, U: ?Sized> DerefMut for GcCellRefMut<'a, T, U> {
    #[inline]
    fn deref_mut(&mut self) -> &mut U {
        self.value
    }
}

impl<'a, T: Trace + ?Sized, U: ?Sized> Drop for GcCellRefMut<'a, T, U> {
    #[inline]
    fn drop(&mut self) {
        debug_assert!(self.gc_cell.flags.get().borrowed() == BorrowState::Writing);
        // Restore the rooted state of the GcCell's contents to the state of the GcCell.
        // During the lifetime of the GcCellRefMut, the GcCell's contents are rooted.
        if !self.gc_cell.flags.get().rooted() {
            unsafe {
                (*self.gc_cell.cell.get()).unroot();
            }
        }
        self.gc_cell
            .flags
            .set(self.gc_cell.flags.get().set_unused());
    }
}

impl<'a, T: Trace + ?Sized, U: Debug + ?Sized> Debug for GcCellRefMut<'a, T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&*(self.deref()), f)
    }
}

impl<'a, T: Trace + ?Sized, U: Display + ?Sized> Display for GcCellRefMut<'a, T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

unsafe impl<T: ?Sized + Send> Send for GcCell<T> {}

impl<T: Trace + Clone> Clone for GcCell<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.borrow().clone())
    }
}

impl<T: Trace + Default> Default for GcCell<T> {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Trace + ?Sized + PartialEq> PartialEq for GcCell<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: Trace + ?Sized + Eq> Eq for GcCell<T> {}

impl<T: Trace + ?Sized + PartialOrd> PartialOrd for GcCell<T> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (*self.borrow()).partial_cmp(&*other.borrow())
    }

    #[inline(always)]
    fn lt(&self, other: &Self) -> bool {
        *self.borrow() < *other.borrow()
    }

    #[inline(always)]
    fn le(&self, other: &Self) -> bool {
        *self.borrow() <= *other.borrow()
    }

    #[inline(always)]
    fn gt(&self, other: &Self) -> bool {
        *self.borrow() > *other.borrow()
    }

    #[inline(always)]
    fn ge(&self, other: &Self) -> bool {
        *self.borrow() >= *other.borrow()
    }
}

impl<T: Trace + ?Sized + Ord> Ord for GcCell<T> {
    #[inline]
    fn cmp(&self, other: &GcCell<T>) -> Ordering {
        (*self.borrow()).cmp(&*other.borrow())
    }
}

impl<T: Trace + ?Sized + Debug> Debug for GcCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.flags.get().borrowed() {
            BorrowState::Unused | BorrowState::Reading => f
                .debug_struct("GcCell")
                .field("value", &self.borrow())
                .finish(),
            BorrowState::Writing => f
                .debug_struct("GcCell")
                .field("value", &"<borrowed>")
                .finish(),
        }
    }
}
