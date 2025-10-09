//! Boa's **`boa_mempool`** crate implements a simple memory pool allocator.
//!
//! # Crate Overview
//! More stuff to be explained later.
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]

use std::alloc::{Layout, alloc, dealloc};
use std::cell::RefCell;
use std::fmt::Debug;
use std::ptr::NonNull;

#[cfg(test)]
mod tests;

/// TODO: Make this related to cache size or something.
const THRESHOLD: usize = 128;

/// An empty slot is a reference (indices within the same pool) to the next free item
/// after this one.
type EmptySlot = usize;

/// A single pool allocated.
struct Pool<T> {
    layout: Layout,
    total: usize,
    available: usize,
    next: usize,
    slots: *mut T,
}

impl<T> Pool<T> {
    /// Create a new pool without checking that `count * size_of::<T>()` is valid.
    #[must_use]
    fn new_unchecked(count: usize) -> Self {
        // Statically assert that the size of a unit is bigger than the size expected
        // for the empty slot.
        let _: () = const {
            assert!(size_of::<T>() >= size_of::<EmptySlot>());
        };

        let layout = Layout::array::<T>(count).expect("Could not allocate this pool.");
        // SAFETY: This will panic if memory or count is not right, which is safe.
        let slots: *mut T = unsafe { alloc(layout).cast() };

        // The first slot should always be pointing to itself as an `EmptySlot`.
        // SAFETY: We statically validated that `size_of::<T>() > size_of::<EmptySlot>()`.
        unsafe {
            slots.cast::<EmptySlot>().write(0);
        }

        Self {
            layout,
            total: count,
            available: count,
            next: 0,
            slots,
        }
    }

    /// Allocate a new block.
    #[inline]
    #[must_use]
    fn alloc(&mut self) -> Option<NonNull<T>> {
        if self.available == 0 {
            return None;
        }

        // Reduce availability.
        self.available -= 1;

        // Get an empty slot.
        // SAFETY: If `self.availability > 0`, `self.next` points to within our slots.
        let ptr = unsafe { self.slots.add(self.next) };
        // SAFETY: We statically ensure `size_of::<T> > size_of::<EmptySlot>`.
        let next: EmptySlot = unsafe { *ptr.cast::<EmptySlot>() };

        // Move next to the next one.
        // If `next` is itself, we know that we haven't allocated past this,
        // so we move to the next slot and update it to be itself as well.
        // If `next` is different, we just set next to next.
        if next == self.next {
            self.next += 1;
            // Unless there's no available in this case, `self.next` points to
            // past the pool at this point.
            if self.available > 0 {
                unsafe {
                    self.slots
                        .add(self.next)
                        .cast::<EmptySlot>()
                        .write(self.next);
                }
            }
        } else {
            self.next = next;
        }

        // SAFETY: We know `ptr` to be within the bounds of our memory.
        Some(unsafe { NonNull::new_unchecked(ptr) })
    }

    /// Free the memory and set its `EmptySlot` value properly.
    /// Returns false if the pointer is not contained in this pool.
    #[inline]
    fn dealloc(&mut self, ptr: NonNull<T>) -> bool {
        if ptr.addr().get() < self.slots.addr() {
            return false;
        }
        let slot_index = (ptr.addr().get() - self.slots.addr()) / size_of::<T>();
        if slot_index >= self.total {
            return false;
        }

        let next = self.next;
        // SAFETY: We know by now that slot_index is between 0 and `self.total`.
        unsafe {
            self.slots.add(slot_index).cast::<EmptySlot>().write(next);
        }
        self.next = slot_index;
        self.available += 1;

        true
    }
}

impl<T> Drop for Pool<T> {
    fn drop(&mut self) {
        // SAFETY: We use the same layout, so this is sure to work.
        unsafe {
            dealloc(self.slots.cast(), self.layout);
        }
    }
}

/// A simple Pool-based memory allocator. This is not thread safe. `T` must
/// have a size larger than `usize`.
///
/// ```compile_fail
/// let pool = MemPoolAllocator::<u8>::new();
/// ```
pub struct MemPoolAllocator<T> {
    pools: RefCell<Vec<Pool<T>>>,
}

impl<T> Debug for MemPoolAllocator<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MemPoolAllocator")
            .field("allocated", &self.allocated())
            .field("available", &self.available())
            .finish()
    }
}

impl<T> Default for MemPoolAllocator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MemPoolAllocator<T> {
    /// Create a new empty allocator. Capacity will grow with allocations.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    /// Create an allocator with `capacity` amount of `T`s. That is, the total
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        // Double-check that the total capacity doesn't exceed `usize`.
        debug_assert!(capacity.checked_mul(size_of::<T>()).is_some());

        Self {
            pools: RefCell::new(vec![Pool::<T>::new_unchecked(capacity)]),
        }
    }

    /// Allocate a new slot and return a pointer to it.
    ///
    /// # Panics
    /// If allocating a new pool region fails, this will panic. Otherwise, it can't.
    #[must_use]
    pub fn alloc(&self) -> NonNull<T> {
        let mut pools = self.pools.borrow_mut();
        // Find the first pool with an unused slot. Use reverse because
        // the last pool is the most likely one to have availability.
        if let Some(p) = pools.iter_mut().find_map(Pool::alloc) {
            p
        } else {
            // Allocate twice the last allocation if smaller than THRESHOLD, or 10% more otherwise.
            let last_total = pools
                .last()
                .expect("There should always be at least one pool.")
                .total;
            let mut new_pool = Pool::<T>::new_unchecked(if last_total < THRESHOLD {
                last_total * 2
            } else {
                last_total + last_total / 10
            });
            let ptr = new_pool.alloc().expect("Could not allocate memory.");
            pools.push(new_pool);
            ptr
        }
    }

    /// Deallocate an existing slot. If the pointer is not within our pool, this
    /// will do nothing.
    pub fn dealloc(&self, ptr: NonNull<T>) {
        self.pools.borrow_mut().iter_mut().any(|p| p.dealloc(ptr));
    }

    /// Return the total capacity of the pool.
    pub fn allocated(&self) -> usize {
        self.pools
            .borrow()
            .iter()
            .fold(0usize, |acc, p| acc + p.total)
    }

    /// Return the total number of objects allocated.
    pub fn available(&self) -> usize {
        self.pools
            .borrow()
            .iter()
            .fold(0usize, |acc, p| acc + p.available)
    }
}
