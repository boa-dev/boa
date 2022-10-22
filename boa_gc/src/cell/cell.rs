//! A garbage collected cell implementation
use std::alloc::Layout;
use std::cell::{Cell, UnsafeCell};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};

mod borrow_flag;

use borrow_flag::{BORROWFLAG_INIT, BorrowFlag};


/// A mutable memory location with dynamically checked borrow rules
/// that can be used inside of a garbage-collected pointer.
///
/// This object is a `RefCell` that can be used inside of a `Gc<T>`.
pub struct GcCell<T: ?Sized + 'static> {
    flags: Cell<BorrowFlag>,
    cell: UnsafeCell<T>,
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
    /// # Examples
    ///
    /// ```
    /// use gc::GcCell;
    ///
    /// let c = GcCell::new(5);
    ///
    /// {
    ///     let m = c.borrow_mut();
    ///     assert!(c.try_borrow().is_err());
    /// }
    ///
    /// {
    ///     let m = c.borrow();
    ///     assert!(c.try_borrow().is_ok());
    /// }
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::GcCell;
    ///
    /// let c = GcCell::new(5);
    ///
    /// {
    ///     let m = c.borrow();
    ///     assert!(c.try_borrow_mut().is_err());
    /// }
    ///
    /// assert!(c.try_borrow_mut().is_ok());
    /// ```
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
    unsafe fn weak_trace(&self, queue: &mut Vec<GcPointer>) {
        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => (*self.cell.get()).weak_trace(queue),
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
    fn finalize_glue(&self) {
        Finalize::finalize(self);
        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            _ => unsafe { (*self.cell.get()).finalize_glue() },
        }
    }
}