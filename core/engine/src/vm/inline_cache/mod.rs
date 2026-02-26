use arrayvec::ArrayVec;
use std::cell::Cell;

use boa_gc::{GcRefCell, Trace as BoaTrace, custom_trace};
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

#[derive(Clone, Debug, Finalize)]
pub(crate) struct PicEntries(pub(crate) ArrayVec<PicEntry, PIC_CAPACITY>);

unsafe impl BoaTrace for PicEntries {
    custom_trace!(this, mark, {
        for entry in &this.0 {
            mark(entry);
        }
    });
}

/// An inline cache entry for a property access.
#[derive(Clone, Debug, Finalize)]
pub(crate) struct InlineCache {
    /// The property that is accessed.
    pub(crate) name: JsString,

    /// Multiple cached shape-to-slot entries.
    pub(crate) entries: GcRefCell<PicEntries>,

    /// Whether this access site has seen too many shapes and should no longer be cached.
    pub(crate) megamorphic: Cell<bool>,
}

unsafe impl BoaTrace for InlineCache {
    custom_trace!(this, mark, {
        mark(&this.name);
        mark(&this.entries);
    });
}

impl InlineCache {
    pub(crate) fn new(name: JsString) -> Self {
        Self {
            name,
            entries: GcRefCell::new(PicEntries(ArrayVec::new())),
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
        for entry in &mut entries.0.iter_mut() {
            if let Some(upgraded) = entry.shape.upgrade()
                && upgraded.to_addr_usize() == shape_addr
            {
                entry.slot = slot;
                return;
            }
        }

        // Add a new entry if there's space.
        if entries.0.len() < PIC_CAPACITY {
            entries.0.push(PicEntry {
                shape: shape.into(),
                slot,
            });
        } else {
            // Polymorphic cache is full, transition to megamorphic.
            self.megamorphic.set(true);
            entries.0.clear();
        }
    }

    /// Returns the cached `(Shape, Slot)` if a matching shape exists in the inline cache.
    ///
    /// Otherwise we reset the internal weak reference to [`WeakShape::None`],
    /// so it can be deallocated by the GC.
    pub(crate) fn match_or_reset(&self, shape: &Shape) -> Option<(Shape, Slot)> {
        if self.megamorphic.get() {
            return None;
        }

        let mut entries = self.entries.borrow_mut();
        let mut i = 0;
        let mut result = None;
        let shape_addr = shape.to_addr_usize();

        while i < entries.0.len() {
            if let Some(upgraded) = entries.0[i].shape.upgrade() {
                if upgraded.to_addr_usize() == shape_addr {
                    result = Some((upgraded, entries.0[i].slot));
                    break;
                }
                i += 1;
            } else {
                // Opportunistically clean up stale weak shapes.
                entries.0.remove(i);
            }
        }

        result
    }

    pub(crate) fn first_shape_addr(&self) -> usize {
        self.entries
            .borrow()
            .0
            .first()
            .and_then(|e| e.shape.upgrade())
            .map_or(0, |s| s.to_addr_usize())
    }
}
