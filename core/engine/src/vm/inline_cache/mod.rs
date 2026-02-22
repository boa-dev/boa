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

    /// Additional PIC entries beyond the primary `shape`/`slot` entry.
    pub(crate) secondary_shapes: GcRefCell<[WeakShape; PIC_CAPACITY - 1]>,

    /// [`Slot`] values for additional PIC entries.
    #[unsafe_ignore_trace]
    pub(crate) secondary_slots: [Cell<Slot>; PIC_CAPACITY - 1],

    /// Number of active entries in `secondary_shapes` / `secondary_slots`.
    #[unsafe_ignore_trace]
    pub(crate) secondary_len: Cell<u8>,

    /// A site is megamorphic when we observe more distinct shapes than the PIC capacity.
    #[unsafe_ignore_trace]
    pub(crate) megamorphic: Cell<bool>,
}

impl InlineCache {
    pub(crate) fn new(name: JsString) -> Self {
        Self {
            name,
            shape: GcRefCell::new(WeakShape::None),
            slot: Cell::new(Slot::new()),
            secondary_shapes: GcRefCell::new(std::array::from_fn(|_| WeakShape::None)),
            secondary_slots: std::array::from_fn(|_| Cell::new(Slot::new())),
            secondary_len: Cell::new(0),
            megamorphic: Cell::new(false),
        }
    }

    fn remove_secondary_entry(
        &self,
        index: usize,
        secondary_len: usize,
        secondary_shapes: &mut [WeakShape; PIC_CAPACITY - 1],
    ) {
        for i in index..(secondary_len - 1) {
            secondary_shapes[i] = secondary_shapes[i + 1].clone();
            self.secondary_slots[i].set(self.secondary_slots[i + 1].get());
        }

        let last = secondary_len - 1;
        secondary_shapes[last] = WeakShape::None;
        self.secondary_slots[last].set(Slot::new());
    }

    fn transition_to_megamorphic(
        &self,
        secondary_shapes: &mut [WeakShape; PIC_CAPACITY - 1],
    ) {
        self.megamorphic.set(true);
        *self.shape.borrow_mut() = WeakShape::None;
        self.slot.set(Slot::new());
        for i in 0..(PIC_CAPACITY - 1) {
            secondary_shapes[i] = WeakShape::None;
            self.secondary_slots[i].set(Slot::new());
        }
        self.secondary_len.set(0);
    }

    pub(crate) fn set(&self, shape: &Shape, slot: Slot) {
        if self.megamorphic.get() {
            return;
        }

        let target_addr = shape.to_addr_usize();

        {
            let mut primary = self.shape.borrow_mut();
            let primary_addr = primary.to_addr_usize();
            if primary_addr == target_addr {
                self.slot.set(slot);
                return;
            }

            if primary_addr == 0 {
                *primary = shape.into();
                self.slot.set(slot);
                return;
            }
        }

        let mut secondary_shapes = self.secondary_shapes.borrow_mut();
        let mut secondary_len = usize::from(self.secondary_len.get());

        let mut i = 0;
        while i < secondary_len {
            let secondary_addr = secondary_shapes[i].to_addr_usize();
            if secondary_addr == 0 {
                self.remove_secondary_entry(i, secondary_len, &mut secondary_shapes);
                secondary_len -= 1;
                self.secondary_len.set(secondary_len as u8);
                continue;
            }

            if secondary_addr == target_addr {
                self.secondary_slots[i].set(slot);
                return;
            }

            i += 1;
        }

        if secondary_len < (PIC_CAPACITY - 1) {
            secondary_shapes[secondary_len] = shape.into();
            self.secondary_slots[secondary_len].set(slot);
            self.secondary_len.set((secondary_len + 1) as u8);
            return;
        }

        self.transition_to_megamorphic(&mut secondary_shapes);
    }

    pub(crate) fn slot(&self) -> Slot {
        self.slot.get()
    }

    /// Returns the cached `(shape, slot)` if this PIC contains a matching shape.
    pub(crate) fn match_shape(&self, shape: &Shape) -> Option<(Shape, Slot)> {
        if self.megamorphic.get() {
            return None;
        }

        let target_addr = shape.to_addr_usize();

        {
            let mut primary = self.shape.borrow_mut();
            let primary_addr = primary.to_addr_usize();
            if primary_addr == target_addr {
                if let Some(shape) = primary.upgrade() {
                    return Some((shape, self.slot()));
                }
                *primary = WeakShape::None;
            } else if primary_addr == 0 {
                *primary = WeakShape::None;
            }
        }

        let mut secondary_shapes = self.secondary_shapes.borrow_mut();
        let mut secondary_len = usize::from(self.secondary_len.get());

        let mut i = 0;
        while i < secondary_len {
            let secondary_addr = secondary_shapes[i].to_addr_usize();
            if secondary_addr == 0 {
                self.remove_secondary_entry(i, secondary_len, &mut secondary_shapes);
                secondary_len -= 1;
                self.secondary_len.set(secondary_len as u8);
                continue;
            }

            if secondary_addr == target_addr {
                if let Some(shape) = secondary_shapes[i].upgrade() {
                    return Some((shape, self.secondary_slots[i].get()));
                }

                self.remove_secondary_entry(i, secondary_len, &mut secondary_shapes);
                secondary_len -= 1;
                self.secondary_len.set(secondary_len as u8);
                continue;
            }

            i += 1;
        }

        None
    }

    #[cfg(test)]
    pub(crate) fn is_megamorphic(&self) -> bool {
        self.megamorphic.get()
    }

    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> usize {
        usize::from(self.shape.borrow().to_addr_usize() != 0) + usize::from(self.secondary_len.get())
    }

    #[cfg(test)]
    pub(crate) fn contains_shape(&self, shape: &Shape) -> bool {
        let target_addr = shape.to_addr_usize();

        if self.shape.borrow().to_addr_usize() == target_addr {
            return true;
        }

        let secondary_len = usize::from(self.secondary_len.get());
        let secondary_shapes = self.secondary_shapes.borrow();
        for i in 0..secondary_len {
            if secondary_shapes[i].to_addr_usize() == target_addr {
                return true;
            }
        }

        false
    }
}
