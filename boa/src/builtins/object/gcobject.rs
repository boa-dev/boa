//! This module implements the `GcObject` structure.
//!
//! The `GcObject` is a garbage collected Object.

use super::Object;
use gc::{Finalize, Gc, GcCell, GcCellRef, GcCellRefMut, Trace};
use std::{
    cell::RefCell,
    collections::HashSet,
    fmt::{self, Debug, Display},
};

/// Garbage collected `Object`.
#[derive(Trace, Finalize, Clone)]
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

#[derive(Debug)]
/// Prevents infinite recursion during `Debug::fmt`.
struct RecursionLimiter {
    /// If this was the first `GcObject` in the tree.
    free: bool,
    /// If this is the first time a specific `GcObject` has been seen.
    first: bool,
}

impl Clone for RecursionLimiter {
    fn clone(&self) -> Self {
        Self {
            // Cloning this value would result in a premature free.
            free: false,
            // Cloning this vlaue would result in a value being written multiple times.
            first: false,
        }
    }
}

impl Drop for RecursionLimiter {
    fn drop(&mut self) {
        // Typically, calling hs.remove(ptr) for "first" objects would be the correct choice here. This would allow the
        // same object to appear multiple times in the output (provided it does not appear under itself recursively).
        // However, the JS object hierarchy involves quite a bit of repitition, and the sheer amount of data makes
        // understanding the Debug output impossible; limiting the usefulness of it.
        //
        // Instead, the entire hashset is emptied at by the first GcObject involved. This means that objects will appear
        // at most once, throughout the graph, hopefully making things a bit clearer.
        if self.free {
            Self::VISITED.with(|hs| hs.borrow_mut().clear());
        }
    }
}

impl RecursionLimiter {
    thread_local! {
        pub static VISITED: RefCell<HashSet<usize>> = RefCell::new(HashSet::new());
    }

    /// Determines if the specified `GcObject` has been visited, and returns a struct that will free it when dropped.
    ///
    /// This is done by maintaining a thread-local hashset containing the pointers of `GcObject` values that have been
    /// visited. The first `GcObject` visited will clear the hashset, while any others will check if they are contained
    /// by the hashset.
    fn new(o: &GcObject) -> Self {
        // We shouldn't have to worry too much about this being moved during Debug::fmt.
        let ptr = (o.as_ref() as *const _) as usize;
        let (free, first) = Self::VISITED.with(|hs| {
            let mut hs = hs.borrow_mut();
            (hs.is_empty(), hs.insert(ptr))
        });

        Self { free, first }
    }
}

impl Debug for GcObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let limiter = RecursionLimiter::new(&self);

        if limiter.first {
            f.debug_tuple("GcObject").field(&self.0).finish()
        } else {
            f.write_str("{ ... }")
        }
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
