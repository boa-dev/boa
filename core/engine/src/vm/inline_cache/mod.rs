use arrayvec::ArrayVec;
use itertools::Itertools;
use std::{cell::Cell, fmt};

use boa_macros::{Finalize, Trace};

use crate::{
    JsString,
    object::shape::{Shape, WeakShape, slot::Slot},
};

#[cfg(test)]
mod tests;

pub(crate) const PIC_CAPACITY: usize = 4;

/// A cached shape-to-slot mapping for a polymorphic inline cache.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct CacheEntry {
    /// A weak reference is kept to the shape to avoid the shape preventing deallocation.
    pub(crate) shape: WeakShape,
    #[unsafe_ignore_trace]
    pub(crate) slot: Slot,
}

/// An inline cache entry for a property access.
#[repr(C)]
#[derive(Trace, Finalize)]
pub(crate) struct InlineCache {
    /// Whether this access site has seen too many shapes and should no longer be cached.
    #[unsafe_ignore_trace]
    pub(crate) megamorphic: Cell<bool>,

    /// The property that is accessed.
    pub(crate) name: JsString,

    /// Multiple cached shape-to-slot entries.
    pub(crate) entries: Cell<ArrayVec<CacheEntry, PIC_CAPACITY>>,
}

impl Clone for InlineCache {
    fn clone(&self) -> Self {
        // SAFETY: `entries` is only ever accessed through `&self`/`&mut self`
        // on this single-threaded cache, and cloning `CacheEntry` doesn't
        // reenter this `Cell`, so it's safe to read through the raw pointer
        // for the duration of this borrow without disturbing the cell's contents.
        let cloned_entries = unsafe { (*self.entries.as_ptr()).clone() };

        Self {
            megamorphic: self.megamorphic.clone(),
            name: self.name.clone(),
            entries: Cell::new(cloned_entries),
        }
    }
}

impl fmt::Debug for InlineCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: `entries` is only ever accessed through `&self`/`&mut self`
        // on this single-threaded cache, and printing doesn't reenter this `Cell`,
        // so it's safe to read through the raw pointer.
        let entries = unsafe { &*self.entries.as_ptr() };
        f.debug_struct("InlineCache")
            .field("name", &self.name)
            .field("entries", entries)
            .field("megamorphic", &self.megamorphic)
            .finish()
    }
}

impl fmt::Display for InlineCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(name:{} entries:", self.name.display_escaped())?;

        if self.megamorphic.get() {
            return write!(f, "(megamorphic))");
        }

        // SAFETY: `entries` is only ever accessed through `&self`/`&mut self`
        // on this single-threaded cache, and printing doesn't reenter this `Cell`,
        // so it's safe to read through the raw pointer.
        let entries = unsafe { &*self.entries.as_ptr() };
        let formatted = entries.iter().map(|e| e.shape.to_addr_usize()).format(", ");
        write!(f, "({formatted:#x}))")
    }
}

impl InlineCache {
    pub(crate) fn new(name: JsString) -> Self {
        Self {
            megamorphic: Cell::new(false),
            name,
            entries: Cell::new(ArrayVec::new()),
        }
    }

    #[cfg(test)]
    pub(crate) fn entries(&self) -> ArrayVec<CacheEntry, PIC_CAPACITY> {
        // SAFETY: `entries` is only ever accessed through `&self`/`&mut self`
        // on this single-threaded cache, so it's safe to clone through the raw pointer.
        unsafe { (*self.entries.as_ptr()).clone() }
    }

    pub(crate) fn set(&self, shape: &Shape, slot: Slot) {
        if self.megamorphic.get() {
            return;
        }

        // SAFETY: `entries` is only ever accessed through `&self`/`&mut self`
        // on this single-threaded cache, and updating doesn't call user-code that
        // could re-entrantly access or mutate `entries`, so it's safe to obtain
        // a mutable reference to the cell's contents.
        let entries = unsafe { &mut *self.entries.as_ptr() };

        // Add a new entry if there's space.
        if entries
            .try_push(CacheEntry {
                shape: shape.into(),
                slot,
            })
            .is_err()
        {
            // Polymorphic cache is full, transition to megamorphic.
            self.megamorphic.set(true);
            entries.clear();
        }
    }

    /// Returns the cached `(Shape, Slot)` if a matching shape exists in the inline cache.
    ///
    /// Opportunistically cleans up stale weak shape references during lookup.
    pub(crate) fn get(&self, shape: &Shape) -> Option<(Shape, Slot)> {
        if self.megamorphic.get() {
            return None;
        }

        // SAFETY: `entries` is only ever accessed through `&self`/`&mut self`
        // on this single-threaded cache, and looking up/upgrading weak shapes
        // doesn't call user-code that could re-entrantly access or mutate `entries`,
        // so it's safe to obtain a mutable reference to the cell's contents.
        let entries = unsafe { &mut *self.entries.as_ptr() };
        let mut i = 0;
        let mut result = None;
        let shape_addr = shape.to_addr_usize();

        while i < entries.len() {
            if let Some(upgraded) = entries[i].shape.upgrade() {
                if upgraded.to_addr_usize() == shape_addr {
                    result = Some((upgraded, entries[i].slot));
                    break;
                }
                i += 1;
            } else {
                // Opportunistically clean up stale weak shapes.
                entries.swap_remove(i);
            }
        }

        result
    }
}
