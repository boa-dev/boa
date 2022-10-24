//! Implementation of a garbage collected cell reference
use std::alloc::Layout;
use std::cell::{Cell, UnsafeCell};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};

use crate::{
    internals::{
        borrow_flag::{BorrowFlag, BorrowState},
        GcCell,
    },
    trace::{Finalize, Trace},
};

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
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::{GcCell, GcCellRef};
    ///
    /// let c = GcCell::new((5, 'b'));
    /// let b1: GcCellRef<(u32, char)> = c.borrow();
    /// let b2: GcCellRef<u32> = GcCellRef::map(b1, |t| &t.0);
    /// //assert_eq!(b2, 5);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::{GcCell, GcCellRef};
    ///
    /// let cell = GcCell::new((1, 'c'));
    /// let borrow = cell.borrow();
    /// let (first, second) = GcCellRef::map_split(borrow, |x| (&x.0, &x.1));
    /// assert_eq!(*first, 1);
    /// assert_eq!(*second, 'c');
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::{GcCell, GcCellRefMut};
    ///
    /// let c = GcCell::new((5, 'b'));
    /// {
    ///     let b1: GcCellRefMut<(u32, char)> = c.borrow_mut();
    ///     let mut b2: GcCellRefMut<(u32, char), u32> = GcCellRefMut::map(b1, |t| &mut t.0);
    ///     assert_eq!(*b2, 5);
    ///     *b2 = 42;
    /// }
    /// assert_eq!(*c.borrow(), (42, 'b'));
    /// ```
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
