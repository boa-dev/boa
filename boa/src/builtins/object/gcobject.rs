//! This module implements the `GcObject` structure.
//!
//! The `GcObject` is a garbage collected Object.

use super::Object;
use gc::{Finalize, Gc, GcCell, GcCellRef, GcCellRefMut, Trace};
use std::fmt::{self, Display};

/// Garbage collected `Object`.
#[derive(Debug, Trace, Finalize, Clone)]
pub struct GcObject(Gc<GcCell<Object>>);

impl GcObject {
    #[inline]
    pub(crate) fn new(object: Object) -> Self {
        Self(Gc::new(GcCell::new(object)))
    }

    #[inline]
    pub fn borrow(&self) -> GcCellRef<'_, Object> {
        self.try_borrow().expect("Object already mutably borrowed")
    }

    #[inline]
    pub fn borrow_mut(&self) -> GcCellRefMut<'_, Object> {
        self.try_borrow_mut().expect("Object already borrowed")
    }

    #[inline]
    pub fn try_borrow(&self) -> Result<GcCellRef<'_, Object>, BorrowError> {
        self.0.try_borrow().map_err(|_| BorrowError)
    }

    #[inline]
    pub fn try_borrow_mut(&self) -> Result<GcCellRefMut<'_, Object>, BorrowMutError> {
        self.0.try_borrow_mut().map_err(|_| BorrowMutError)
    }

    /// Checks if the garbage collected memory is the same.
    #[inline]
    pub fn equals(lhs: &Self, rhs: &Self) -> bool {
        std::ptr::eq(lhs.as_ref(), rhs.as_ref())
    }
}

impl AsRef<GcCell<Object>> for GcObject {
    #[inline]
    fn as_ref(&self) -> &GcCell<Object> {
        &*self.0
    }
}

/// An error returned by [`GcObject::try_borrow`](struct.GcObject.html#method.try_borrow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowError;

impl Display for BorrowError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already mutably borrowed", f)
    }
}

/// An error returned by [`GcObject::try_borrow_mut`](struct.GcObject.html#method.try_borrow_mut).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowMutError;

impl Display for BorrowMutError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("Object already borrowed", f)
    }
}
