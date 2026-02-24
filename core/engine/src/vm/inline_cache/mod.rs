use std::cell::Cell;

use boa_gc::GcRefCell;
use boa_macros::{Finalize, Trace};

use crate::{
    JsString,
    object::shape::{Shape, WeakShape, slot::Slot},
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

    /// Number of prototype chain hops to reach the object that owns the property.
    ///
    /// `0` means the property is on the object itself. `1` means it is on the
    /// direct prototype, `2` on the prototype's prototype, and so on.
    #[unsafe_ignore_trace]
    pub(crate) prototype_hops: Cell<u8>,
}

impl InlineCache {
    pub(crate) const fn new(name: JsString) -> Self {
        Self {
            name,
            shape: GcRefCell::new(WeakShape::None),
            slot: Cell::new(Slot::new()),
            prototype_hops: Cell::new(0),
        }
    }

    pub(crate) fn set(&self, shape: &Shape, slot: Slot, prototype_hops: u8) {
        *self.shape.borrow_mut() = shape.into();
        self.slot.set(slot);
        self.prototype_hops.set(prototype_hops);
    }

    pub(crate) fn slot(&self) -> Slot {
        self.slot.get()
    }

    pub(crate) fn prototype_hops(&self) -> u8 {
        self.prototype_hops.get()
    }

    /// Returns true, if the [`InlineCache`]'s shape matches with the given shape.
    ///
    /// Otherwise we reset the internal weak reference to [`WeakShape::None`],
    /// so it can be deallocated by the GC.
    pub(crate) fn match_or_reset(&self, shape: &Shape) -> Option<(Shape, Slot, u8)> {
        let mut old = self.shape.borrow_mut();

        let old_upgraded = old.upgrade();
        if old_upgraded.as_ref().map_or(0, Shape::to_addr_usize) == shape.to_addr_usize() {
            return old_upgraded.map(|shape| (shape, self.slot(), self.prototype_hops()));
        }

        *old = WeakShape::None;
        None
    }
}
