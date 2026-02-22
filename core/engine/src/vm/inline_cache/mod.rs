use std::cell::Cell;

use boa_gc::GcRefCell;
use boa_macros::{Finalize, Trace};

use crate::{
    JsString,
    object::shape::{Shape, WeakShape, slot::Slot},
};

#[cfg(test)]
mod tests;

const PIC_CAPACITY: usize = 4;

#[derive(Clone, Debug, Trace, Finalize)]
struct PicEntry {
    shape: WeakShape,

    #[unsafe_ignore_trace]
    slot: Slot,
}

/// An inline cache entry for a property access.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct InlineCache {
    /// The property that is accessed.
    pub(crate) name: JsString,

    /// Cached `(shape, slot)` entries for this access site.
    ///
    /// NOTE: This should never exceed `PIC_CAPACITY`.
    entries: GcRefCell<Vec<PicEntry>>,

    /// A site is megamorphic when we observe more distinct shapes than the PIC capacity.
    #[unsafe_ignore_trace]
    megamorphic: Cell<bool>,
}

impl InlineCache {
    pub(crate) fn new(name: JsString) -> Self {
        Self {
            name,
            entries: GcRefCell::new(Vec::with_capacity(PIC_CAPACITY)),
            megamorphic: Cell::new(false),
        }
    }

    fn transition_to_megamorphic(&self, entries: &mut Vec<PicEntry>) {
        self.megamorphic.set(true);
        entries.clear();
    }

    pub(crate) fn set(&self, shape: &Shape, slot: Slot) {
        if self.megamorphic.get() {
            return;
        }

        let target_addr = shape.to_addr_usize();
        let mut entries = self.entries.borrow_mut();

        entries.retain(|entry| entry.shape.to_addr_usize() != 0);

        if let Some(entry) = entries
            .iter_mut()
            .find(|entry| entry.shape.to_addr_usize() == target_addr)
        {
            entry.slot = slot;
            return;
        }

        if entries.len() < PIC_CAPACITY {
            entries.push(PicEntry {
                shape: shape.into(),
                slot,
            });
            return;
        }

        self.transition_to_megamorphic(&mut entries);
    }

    /// Returns the cached `(shape, slot)` if this PIC contains a matching shape.
    pub(crate) fn match_shape(&self, shape: &Shape) -> Option<(Shape, Slot)> {
        if self.megamorphic.get() {
            return None;
        }

        let target_addr = shape.to_addr_usize();
        let mut entries = self.entries.borrow_mut();

        entries.retain(|entry| entry.shape.to_addr_usize() != 0);

        if let Some(entry) = entries
            .iter()
            .find(|entry| entry.shape.to_addr_usize() == target_addr)
        {
            if let Some(shape) = entry.shape.upgrade() {
                return Some((shape, entry.slot));
            }
        }

        None
    }

    /// Returns the address of the first cached shape.
    ///
    /// This is only used for VM disassembly/debug output.
    pub(crate) fn first_shape_addr(&self) -> usize {
        if self.megamorphic.get() {
            return 0;
        }

        let mut entries = self.entries.borrow_mut();
        entries.retain(|entry| entry.shape.to_addr_usize() != 0);
        entries
            .first()
            .map(|entry| entry.shape.to_addr_usize())
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn is_megamorphic(&self) -> bool {
        self.megamorphic.get()
    }

    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> usize {
        self.entries.borrow().len()
    }

    #[cfg(test)]
    pub(crate) fn contains_shape(&self, shape: &Shape) -> bool {
        let target_addr = shape.to_addr_usize();
        self.entries
            .borrow()
            .iter()
            .any(|entry| entry.shape.to_addr_usize() == target_addr)
    }
}
