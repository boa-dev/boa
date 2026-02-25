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

#[derive(Clone, Debug, Trace, Finalize)]
struct PicEntries {
    entries: [Option<PicEntry>; PIC_CAPACITY],

    #[unsafe_ignore_trace]
    len: u8,
}

impl PicEntries {
    fn new() -> Self {
        Self {
            entries: std::array::from_fn(|_| None),
            len: 0,
        }
    }

    fn len(&self) -> usize {
        self.len as usize
    }

    fn clear(&mut self) {
        for i in 0..self.len() {
            self.entries[i] = None;
        }
        self.len = 0;
    }

    fn retain_live(&mut self) {
        let old_len = self.len();
        let mut write = 0;

        for read in 0..old_len {
            let Some(entry) = self.entries[read].take() else {
                continue;
            };
            if entry.shape.to_addr_usize() != 0 {
                self.entries[write] = Some(entry);
                write += 1;
            }
        }

        for i in write..old_len {
            self.entries[i] = None;
        }
        self.len = write as u8;
    }

    fn find_index_by_shape_addr(&self, target_addr: usize) -> Option<usize> {
        (0..self.len()).find(|&index| {
            self.entries[index]
                .as_ref()
                .is_some_and(|entry| entry.shape.to_addr_usize() == target_addr)
        })
    }

    fn push(&mut self, entry: PicEntry) {
        debug_assert!(self.len() < PIC_CAPACITY);
        self.entries[self.len()] = Some(entry);
        self.len += 1;
    }

    fn first_shape_addr(&self) -> usize {
        (0..self.len())
            .find_map(|index| self.entries[index].as_ref().map(|entry| entry.shape.to_addr_usize()))
            .unwrap_or_default()
    }
}

/// An inline cache entry for a property access.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct InlineCache {
    /// The property that is accessed.
    pub(crate) name: JsString,

    /// Cached `(shape, slot)` entries for this access site.
    ///
    /// NOTE: This should never exceed `PIC_CAPACITY`.
    entries: GcRefCell<PicEntries>,

    /// A site is megamorphic when we observe more distinct shapes than the PIC capacity.
    #[unsafe_ignore_trace]
    megamorphic: Cell<bool>,
}

impl InlineCache {
    pub(crate) fn new(name: JsString) -> Self {
        Self {
            name,
            entries: GcRefCell::new(PicEntries::new()),
            megamorphic: Cell::new(false),
        }
    }

    fn transition_to_megamorphic(&self, entries: &mut PicEntries) {
        self.megamorphic.set(true);
        entries.clear();
    }

    pub(crate) fn set(&self, shape: &Shape, slot: Slot) {
        if self.megamorphic.get() {
            return;
        }

        let target_addr = shape.to_addr_usize();
        let mut entries = self.entries.borrow_mut();

        entries.retain_live();

        if let Some(index) = entries.find_index_by_shape_addr(target_addr)
            && let Some(entry) = entries.entries[index].as_mut()
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

        entries.retain_live();

        if let Some(index) = entries.find_index_by_shape_addr(target_addr)
            && let Some(entry) = entries.entries[index].as_ref()
            && let Some(shape) = entry.shape.upgrade()
        {
            return Some((shape, entry.slot));
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
        entries.retain_live();
        entries.first_shape_addr()
    }

    #[cfg(test)]
    pub(crate) fn is_megamorphic(&self) -> bool {
        self.megamorphic.get()
    }

    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> usize {
        let mut entries = self.entries.borrow_mut();
        entries.retain_live();
        entries.len()
    }

    #[cfg(test)]
    pub(crate) fn contains_shape(&self, shape: &Shape) -> bool {
        let mut entries = self.entries.borrow_mut();
        entries.retain_live();
        let target_addr = shape.to_addr_usize();
        entries.find_index_by_shape_addr(target_addr).is_some()
    }
}
