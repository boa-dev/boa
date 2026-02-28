use arrayvec::ArrayVec;
use std::cell::Cell;

use boa_gc::GcRefCell;
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
pub(crate) struct PicEntry {
    /// A weak reference is kept to the shape to avoid the shape preventing deallocation.
    pub(crate) shape: WeakShape,
    #[unsafe_ignore_trace]
    pub(crate) slot: Slot,
}

/// An inline cache entry for a property access.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct InlineCache {
    /// The property that is accessed.
    pub(crate) name: JsString,

    /// Multiple cached shape-to-slot entries.
    pub(crate) entries: GcRefCell<ArrayVec<PicEntry, PIC_CAPACITY>>,

    /// Whether this access site has seen too many shapes and should no longer be cached.
    #[unsafe_ignore_trace]
    pub(crate) megamorphic: Cell<bool>,
}

impl InlineCache {
    pub(crate) fn new(name: JsString) -> Self {
        Self {
            name,
            entries: GcRefCell::new(ArrayVec::new()),
            megamorphic: Cell::new(false),
        }
    }

    pub(crate) fn set(&self, shape: &Shape, slot: Slot) {
        if self.megamorphic.get() {
            return;
        }

        let mut entries = self.entries.borrow_mut();
        let shape_addr = shape.to_addr_usize();

        // If the shape already exists, update its slot.
        // This handles cases where property transitions preserve the shape but change the slot.
        for entry in entries.iter_mut() {
            if let Some(upgraded) = entry.shape.upgrade()
                && upgraded.to_addr_usize() == shape_addr
            {
                entry.slot = slot;
                return;
            }
        }

        // Add a new entry if there's space.
        if entries.len() < PIC_CAPACITY {
            entries.push(PicEntry {
                shape: shape.into(),
                slot,
            });
        } else {
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

        let mut entries = self.entries.borrow_mut();
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

    /// Returns a formatted string displaying all cached shape addresses.
    pub(crate) fn shapes_display(&self) -> String {
        if self.megamorphic.get() {
            return "megamorphic".into();
        }
        let entries = self.entries.borrow();
        let addrs: Vec<String> = entries
            .iter()
            .filter_map(|e| e.shape.upgrade())
            .map(|s| format!("0x{:x}", s.to_addr_usize()))
            .collect();
        if addrs.is_empty() {
            "empty".into()
        } else {
            addrs.join(", ")
        }
    }
}
