//! A garbage collected cell implementation

use crate::trace::{Finalize, Trace};
use std::{
    cell::{Cell, UnsafeCell},
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::Hash,
    ops::{Deref, DerefMut},
};

/// `BorrowFlag` represent the internal state of a `GcCell` and
/// keeps track of the amount of current borrows.
#[derive(Copy, Clone)]
pub(crate) struct BorrowFlag(usize);

/// `BorrowState` represents the various states of a `BorrowFlag`
///
///  - Reading: the value is currently being read/borrowed.
///  - Writing: the value is currently being written/borrowed mutably.
///  - Unused: the value is currently unrooted.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum BorrowState {
    Reading,
    Writing,
    Unused,
}

const ROOT: usize = 1;
const WRITING: usize = !1;
const UNUSED: usize = 0;

/// The base borrowflag init is rooted, and has no outstanding borrows.
pub(crate) const BORROWFLAG_INIT: BorrowFlag = BorrowFlag(ROOT);

impl BorrowFlag {
    /// Check the current `BorrowState` of `BorrowFlag`.
    pub(crate) const fn borrowed(self) -> BorrowState {
        match self.0 & !ROOT {
            UNUSED => BorrowState::Unused,
            WRITING => BorrowState::Writing,
            _ => BorrowState::Reading,
        }
    }

    /// Check whether the borrow bit is flagged.
    pub(crate) const fn rooted(self) -> bool {
        self.0 & ROOT > 0
    }

    /// Set the `BorrowFlag`'s state to writing.
    pub(crate) const fn set_writing(self) -> Self {
        // Set every bit other than the root bit, which is preserved
        Self(self.0 | WRITING)
    }

    /// Remove the root flag on `BorrowFlag`
    pub(crate) const fn set_unused(self) -> Self {
        // Clear every bit other than the root bit, which is preserved
        Self(self.0 & ROOT)
    }

    /// Increments the counter for a new borrow.
    ///
    /// # Panic
    ///  - This method will panic if the current `BorrowState` is writing.
    ///  - This method will panic after incrementing if the borrow count overflows.
    pub(crate) fn add_reading(self) -> Self {
        assert!(self.borrowed() != BorrowState::Writing);
        // Add 1 to the integer starting at the second binary digit. As our
        // borrowstate is not writing, we know that overflow cannot happen, so
        // this is equivalent to the following, more complicated, expression:
        //
        // BorrowFlag((self.0 & ROOT) | (((self.0 >> 1) + 1) << 1))
        let flags = Self(self.0 + 0b10);

        // This will fail if the borrow count overflows, which shouldn't happen,
        // but let's be safe
        {
            assert!(flags.borrowed() == BorrowState::Reading);
        }
        flags
    }

    /// Decrements the counter to remove a borrow.
    ///
    /// # Panic
    ///  - This method will panic if the current `BorrowState` is not reading.
    pub(crate) fn sub_reading(self) -> Self {
        assert!(self.borrowed() == BorrowState::Reading);
        // Subtract 1 from the integer starting at the second binary digit. As
        // our borrowstate is not writing or unused, we know that overflow or
        // undeflow cannot happen, so this is equivalent to the following, more
        // complicated, expression:
        //
        // BorrowFlag((self.0 & ROOT) | (((self.0 >> 1) - 1) << 1))
        Self(self.0 - 0b10)
    }

    /// Set the root flag on the `BorrowFlag`.
    pub(crate) fn set_rooted(self, rooted: bool) -> Self {
        // Preserve the non-root bits
        Self((self.0 & !ROOT) | (usize::from(rooted)))
    }
}

impl Debug for BorrowFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowFlag")
            .field("Rooted", &self.rooted())
            .field("State", &self.borrowed())
            .finish()
    }
}

/// A mutable memory location with dynamically checked borrow rules
/// that can be used inside of a garbage-collected pointer.
///
/// This object is a `RefCell` that can be used inside of a `Gc<T>`.
pub struct GcRefCell<T: ?Sized + 'static> {
    pub(crate) flags: Cell<BorrowFlag>,
    pub(crate) cell: UnsafeCell<T>,
}

impl<T: Trace> GcRefCell<T> {
    /// Creates a new `GcCell` containing `value`.
    pub const fn new(value: T) -> Self {
        Self {
            flags: Cell::new(BORROWFLAG_INIT),
            cell: UnsafeCell::new(value),
        }
    }

    /// Consumes the `GcCell`, returning the wrapped value.
    pub fn into_inner(self) -> T {
        self.cell.into_inner()
    }
}

impl<T: Trace + ?Sized> GcRefCell<T> {
    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `GcCellRef` exits scope.
    /// Multiple immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    pub fn borrow(&self) -> GcRef<'_, T> {
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
    pub fn borrow_mut(&self) -> GcRefMut<'_, T> {
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
    /// # Errors
    ///
    /// Returns an `Err` if the value is currently mutably borrowed.
    pub fn try_borrow(&self) -> Result<GcRef<'_, T>, BorrowError> {
        if self.flags.get().borrowed() == BorrowState::Writing {
            return Err(BorrowError);
        }
        self.flags.set(self.flags.get().add_reading());

        // SAFETY: calling value on a rooted value may cause Undefined Behavior
        unsafe {
            Ok(GcRef {
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
    /// # Errors
    ///
    /// Returns an `Err` if the value is currently borrowed.
    pub fn try_borrow_mut(&self) -> Result<GcRefMut<'_, T>, BorrowMutError> {
        if self.flags.get().borrowed() != BorrowState::Unused {
            return Err(BorrowMutError);
        }
        self.flags.set(self.flags.get().set_writing());

        // SAFETY: This is safe as the value is rooted if it was not previously rooted,
        // so it cannot be dropped.
        unsafe {
            // Force the val_ref's contents to be rooted for the duration of the
            // mutable borrow
            if !self.flags.get().rooted() {
                (*self.cell.get()).root();
            }

            Ok(GcRefMut {
                gc_cell: self,
                value: &mut *self.cell.get(),
            })
        }
    }
}

/// An error returned by [`GcCell::try_borrow`](struct.GcCell.html#method.try_borrow).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct BorrowError;

impl Display for BorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt("GcCell<T> already mutably borrowed", f)
    }
}

/// An error returned by [`GcCell::try_borrow_mut`](struct.GcCell.html#method.try_borrow_mut).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct BorrowMutError;

impl Display for BorrowMutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt("GcCell<T> already borrowed", f)
    }
}

impl<T: Trace + ?Sized> Finalize for GcRefCell<T> {}

// SAFETY: GcCell maintains it's own BorrowState and rootedness. GcCell's implementation
// focuses on only continuing Trace based methods while the cell state is not written.
// Implementing a Trace while the cell is being written to or incorrectly implementing Trace
// on GcCell's value may cause Undefined Behavior
unsafe impl<T: Trace + ?Sized> Trace for GcRefCell<T> {
    unsafe fn trace(&self) {
        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            // SAFETY: Please see GcCell's Trace impl Safety note.
            _ => unsafe { (*self.cell.get()).trace() },
        }
    }

    unsafe fn root(&self) {
        assert!(!self.flags.get().rooted(), "Can't root a GcCell twice!");
        self.flags.set(self.flags.get().set_rooted(true));

        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            // SAFETY: Please see GcCell's Trace impl Safety note.
            _ => unsafe { (*self.cell.get()).root() },
        }
    }

    unsafe fn unroot(&self) {
        assert!(self.flags.get().rooted(), "Can't unroot a GcCell twice!");
        self.flags.set(self.flags.get().set_rooted(false));

        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            // SAFETY: Please see GcCell's Trace impl Safety note.
            _ => unsafe { (*self.cell.get()).unroot() },
        }
    }

    fn run_finalizer(&self) {
        Finalize::finalize(self);
        match self.flags.get().borrowed() {
            BorrowState::Writing => (),
            // SAFETY: Please see GcCell's Trace impl Safety note.
            _ => unsafe { (*self.cell.get()).run_finalizer() },
        }
    }
}

/// A wrapper type for an immutably borrowed value from a `GcCell<T>`.
pub struct GcRef<'a, T: ?Sized + 'static> {
    pub(crate) flags: &'a Cell<BorrowFlag>,
    pub(crate) value: &'a T,
}

impl<'a, T: ?Sized> GcRef<'a, T> {
    /// Copies a `GcCellRef`.
    ///
    /// The `GcCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `GcCellRef::clone(...)`. A `Clone` implementation or a method
    /// would interfere with the use of `c.borrow().clone()` to clone
    /// the contents of a `GcCell`.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn clone(orig: &GcRef<'a, T>) -> GcRef<'a, T> {
        orig.flags.set(orig.flags.get().add_reading());
        GcRef {
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
    pub fn map<U, F>(orig: Self, f: F) -> GcRef<'a, U>
    where
        U: ?Sized,
        F: FnOnce(&T) -> &U,
    {
        let ret = GcRef {
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
    /// This is an associated function that needs to be used as `GcCellRef::map_split(...)`.
    /// A method would interfere with methods of the same name on the contents of a `GcCellRef` used through `Deref`.
    pub fn map_split<U, V, F>(orig: Self, f: F) -> (GcRef<'a, U>, GcRef<'a, V>)
    where
        U: ?Sized,
        V: ?Sized,
        F: FnOnce(&T) -> (&U, &V),
    {
        let (a, b) = f(orig.value);

        orig.flags.set(orig.flags.get().add_reading());

        let ret = (
            GcRef {
                flags: orig.flags,
                value: a,
            },
            GcRef {
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

impl<T: ?Sized> Deref for GcRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value
    }
}

impl<T: ?Sized> Drop for GcRef<'_, T> {
    fn drop(&mut self) {
        debug_assert!(self.flags.get().borrowed() == BorrowState::Reading);
        self.flags.set(self.flags.get().sub_reading());
    }
}

impl<T: ?Sized + Debug> Debug for GcRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + Display> Display for GcRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

/// A wrapper type for a mutably borrowed value from a `GcCell<T>`.
pub struct GcRefMut<'a, T: Trace + ?Sized + 'static, U: ?Sized = T> {
    pub(crate) gc_cell: &'a GcRefCell<T>,
    pub(crate) value: &'a mut U,
}

impl<'a, T: Trace + ?Sized, U: ?Sized> GcRefMut<'a, T, U> {
    /// Makes a new `GcCellRefMut` for a component of the borrowed data, e.g., an enum
    /// variant.
    ///
    /// The `GcCellRefMut` is already mutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `GcCellRefMut::map(...)`. A method would interfere with methods of the same
    /// name on the contents of a `GcCell` used through `Deref`.
    pub fn map<V, F>(orig: Self, f: F) -> GcRefMut<'a, T, V>
    where
        V: ?Sized,
        F: FnOnce(&mut U) -> &mut V,
    {
        #[allow(trivial_casts)]
        // SAFETY: This is safe as `GcCellRefMut` is already borrowed, so the value is rooted.
        let value = unsafe { &mut *(orig.value as *mut U) };

        let ret = GcRefMut {
            gc_cell: orig.gc_cell,
            value: f(value),
        };

        // We have to tell the compiler not to call the destructor of GcCellRefMut,
        // because it will update the borrow flags.
        std::mem::forget(orig);

        ret
    }
}

impl<T: Trace + ?Sized, U: ?Sized> Deref for GcRefMut<'_, T, U> {
    type Target = U;

    fn deref(&self) -> &U {
        self.value
    }
}

impl<T: Trace + ?Sized, U: ?Sized> DerefMut for GcRefMut<'_, T, U> {
    fn deref_mut(&mut self) -> &mut U {
        self.value
    }
}

impl<T: Trace + ?Sized, U: ?Sized> Drop for GcRefMut<'_, T, U> {
    fn drop(&mut self) {
        debug_assert!(self.gc_cell.flags.get().borrowed() == BorrowState::Writing);
        // Restore the rooted state of the GcCell's contents to the state of the GcCell.
        // During the lifetime of the GcCellRefMut, the GcCell's contents are rooted.
        if !self.gc_cell.flags.get().rooted() {
            // SAFETY: If `GcCell` is no longer rooted, then unroot it. This should be safe
            // as the internal `GcBox` should be guaranteed to have at least 1 root.
            unsafe {
                (*self.gc_cell.cell.get()).unroot();
            }
        }
        self.gc_cell
            .flags
            .set(self.gc_cell.flags.get().set_unused());
    }
}

impl<T: Trace + ?Sized, U: Debug + ?Sized> Debug for GcRefMut<'_, T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: Trace + ?Sized, U: Display + ?Sized> Display for GcRefMut<'_, T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

// SAFETY: GcCell<T> tracks it's `BorrowState` is `Writing`
unsafe impl<T: ?Sized + Send> Send for GcRefCell<T> {}

impl<T: Trace + Clone> Clone for GcRefCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.borrow().clone())
    }
}

impl<T: Trace + Default> Default for GcRefCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[allow(clippy::inline_always)]
impl<T: Trace + ?Sized + PartialEq> PartialEq for GcRefCell<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: Trace + ?Sized + Eq> Eq for GcRefCell<T> {}

#[allow(clippy::inline_always)]
impl<T: Trace + ?Sized + PartialOrd> PartialOrd for GcRefCell<T> {
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

impl<T: Trace + ?Sized + Ord> Ord for GcRefCell<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self.borrow()).cmp(&*other.borrow())
    }
}

impl<T: Trace + ?Sized + Debug> Debug for GcRefCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.flags.get().borrowed() {
            BorrowState::Unused | BorrowState::Reading => f
                .debug_struct("GcCell")
                .field("flags", &self.flags.get())
                .field("value", &self.borrow())
                .finish(),
            BorrowState::Writing => f
                .debug_struct("GcCell")
                .field("flags", &self.flags.get())
                .field("value", &"<borrowed>")
                .finish(),
        }
    }
}
