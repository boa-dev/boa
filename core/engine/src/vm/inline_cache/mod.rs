use std::cell::Cell;

use boa_gc::GcRefCell;
use boa_macros::{Finalize, Trace};

use crate::{
    JsString,
    object::{
        JsObject,
        shape::{Shape, WeakShape, slot::Slot},
    },
};

#[cfg(test)]
mod tests;

/// Maximum number of shape entries stored per inline cache site (polymorphic IC).
const IC_CAPACITY: usize = 4;

/// One entry in a polymorphic inline cache.
///
/// A zeroed/default entry has `shape = WeakShape::None` and is treated as empty.
#[derive(Clone, Debug, Trace, Finalize)]
struct IcEntry {
    shape: WeakShape,
    prototype: Option<JsObject>,
    #[unsafe_ignore_trace]
    slot: Slot,
}

impl IcEntry {
    const fn empty() -> Self {
        Self {
            shape: WeakShape::None,
            prototype: None,
            slot: Slot::new(),
        }
    }
}

#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct InlineCache {
    /// The property that is accessed.
    pub(crate) name: JsString,

    /// Fixed-size array of cached entries.
    entries: GcRefCell<[IcEntry; IC_CAPACITY]>,

    /// Number of live entries (0..=IC_CAPACITY).
    #[unsafe_ignore_trace]
    count: Cell<u8>,

    /// Round-robin eviction pointer used when the cache is full.
    #[unsafe_ignore_trace]
    next_evict: Cell<u8>,
}

impl InlineCache {
    pub(crate) fn new(name: JsString) -> Self {
        Self {
            name,
            entries: GcRefCell::new([
                IcEntry::empty(),
                IcEntry::empty(),
                IcEntry::empty(),
                IcEntry::empty(),
            ]),
            count: Cell::new(0),
            next_evict: Cell::new(0),
        }
    }

    /// Cache a property lookup result.
    ///
    /// If `shape` already has an entry it is updated in place.
    /// Otherwise a new slot is used (up to [`IC_CAPACITY`]) or the oldest
    /// entry is evicted with a round-robin policy.
    pub(crate) fn set(&self, shape: &Shape, slot: Slot, prototype: Option<JsObject>) {
        let shape_addr = shape.to_addr_usize();
        let count = self.count.get() as usize;
        let mut entries = self.entries.borrow_mut();

        // Update existing entry for this shape.
        for entry in entries[..count].iter_mut() {
            if entry.shape.to_addr_usize() == shape_addr {
                entry.shape = shape.into();
                entry.slot = slot;
                entry.prototype = prototype;
                return;
            }
        }

        let idx = if count < IC_CAPACITY {
            // Append.
            self.count.set((count + 1) as u8);
            count
        } else {
            // Round-robin eviction.
            let idx = self.next_evict.get() as usize;
            self.next_evict.set(((idx + 1) % IC_CAPACITY) as u8);
            idx
        };

        entries[idx] = IcEntry {
            shape: shape.into(),
            slot,
            prototype,
        };
    }

    /// Returns the slot of the first cached entry.
    ///
    /// Used only in tests.
    pub(crate) fn slot(&self) -> Slot {
        self.entries.borrow()[0].slot
    }

    /// Returns the prototype of the first cached entry.
    ///
    /// Used only in tests.
    pub(crate) fn prototype(&self) -> Option<JsObject> {
        self.entries.borrow()[0].prototype.clone()
    }

    /// Returns the address of the first cached shape, for debug display only.
    pub(crate) fn first_shape_addr(&self) -> usize {
        self.entries.borrow()[0].shape.to_addr_usize()
    }

    /// Returns `Some((shape, slot, prototype))` if any cached entry matches
    /// `shape`, scanning up to `count` live entries.
    ///
    /// Unlike the original monomorphic implementation, a miss does **not**
    /// clear the cache — other entries may still be valid for different shapes.
    pub(crate) fn match_or_reset(&self, shape: &Shape) -> Option<(Shape, Slot, Option<JsObject>)> {
        let shape_addr = shape.to_addr_usize();
        let count = self.count.get() as usize;
        let entries = self.entries.borrow();

        for entry in &entries[..count] {
            let cached_addr = entry.shape.to_addr_usize();
            if cached_addr == shape_addr {
                return entry.shape.upgrade().map(|s| (s, entry.slot, entry.prototype.clone()));
            }
        }

        None
    }
}
