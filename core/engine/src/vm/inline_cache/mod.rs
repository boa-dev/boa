use std::cell::Cell;

use boa_gc::GcRefCell;
use boa_macros::{Finalize, Trace};

use crate::{
    object::shape::{slot::Slot, Shape, WeakShape},
    JsString,
};

#[cfg(test)]
mod tests;

/// An inline cache entry for a property access.
#[derive(Clone, Debug, Trace, Finalize)]
pub(crate) struct InlineCache {
    /// The property that is accessed.
    pub(crate) name: JsString,

    /// A pointer is kept to the shape to avoid the shape from being deallocated.
    pub(crate) shape: GcRefCell<WeakShape>,

    /// The [`Slot`] of the property.
    #[unsafe_ignore_trace]
    pub(crate) slot: Cell<Slot>,
}

impl InlineCache {
    pub(crate) const fn new(name: JsString) -> Self {
        Self {
            name,
            shape: GcRefCell::new(WeakShape::None),
            slot: Cell::new(Slot::new()),
        }
    }

    pub(crate) fn set(&self, shape: &Shape, slot: Slot) {
        *self.shape.borrow_mut() = shape.into();
        self.slot.set(slot);
    }

    pub(crate) fn slot(&self) -> Slot {
        self.slot.get()
    }

    /// Returns true, if the [`InlineCache`]'s shape matches with the given shape.
    ///
    /// Otherwise we reset the internal weak reference to [`WeakShape::None`],
    /// so it can be deallocated by the GC.
    pub(crate) fn match_or_reset(&self, shape: &Shape) -> Option<(Shape, Slot)> {
        let mut old = self.shape.borrow_mut();

        let old_upgraded = old.upgrade();
        if old_upgraded.as_ref().map_or(0, Shape::to_addr_usize) == shape.to_addr_usize() {
            return old_upgraded.map(|shape| (shape, self.slot()));
        }

        *old = WeakShape::None;
        None
    }
}
