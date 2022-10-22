use gc::{Trace, Finalize};
use std::cell::Cell;
use std::ptr::{self, NonNull};

const WEAK_MASK: usize = 1 << usize::BITS;
const MARK_MASK: usize = 1 << (usize::BITS - 1);
const ROOTS_MASK: usize = !(WEAK_MASK | MARK_MASK);
const ROOTS_MAX: usize = ROOTS_MASK;

pub(crate) struct HeapBoxHeader {
    references: Cell<usize>,
    next: Cell<Option<NonNull<HeapBox<dyn Trace>>>>,
}

impl HeapBoxHeader {
    #[inline]
    pub fn new(next: Option<NonNull<HeapBox<dyn Trace>>>) -> Self {
        // TODO: implement a way for a cell to start out weak with WEAK_MASK
        HeapBoxHeader {
            references: Cell::new(1),
            next: Cell::new(next),
        }
    }

    #[inline]
    pub fn roots(&self) -> usize {
        &self.roots.get() & ROOTS_MASK
    }

    #[inline]
    pub fn inc_roots(&self) {
        let roots = self.roots.get();

        if (roots & ROOTS_MASK) < ROOTS_MAX {
            self.roots.set(roots + 1); 
        } else {
            // TODO: implement a better way to handle root overload
            panic!("roots counter overflow");
        }
    }

    #[inline]
    pub fn dec_roots(&self) {
        self.roots.set(self.roots.get() - 1) // no underflow check
    }

    #[inline]
    pub fn is_marked(&self) -> bool {
        self.roots.get() & MARK_MASK != 0
    }

    #[inline]
    pub fn mark(&self) {
        self.roots.set(self.roots.get() | MARK_MASK)
    }

    #[inline]
    pub fn unmark(&self) {
        self.roots.set(self.roots.get() & !MARK_MASK)
    }

    #[inline]
    pub fn is_ephemeron(&self) {
        self.roots.get() & WEAK_MASK != 0
    }
}


/// The HeapBox represents a box on the GC Heap. The HeapBox's creation and allocation is managed
/// by the allocator 
pub(crate) struct HeapBox<T: Trace + ?Sized + 'static> {
    header: HeapBoxHeader,
    object: T,
}

impl<T: Trace + ?Sized> HeapBox<T> {
    /// Returns `true` if the two references refer to the same `GcBox`.
    pub(crate) fn ptr_eq(this: &HeapBox<T>, other: &HeapBox<T>) -> bool {
        // Use .header to ignore fat pointer vtables, to work around
        // https://github.com/rust-lang/rust/issues/46139
        ptr::eq(&this.header, &other.header)
    }

    /// Marks this `GcBox` and marks through its data.
    pub(crate) unsafe fn trace_inner(&self) {
        if !self.header.is_marked() && !self.header.is_ephemeron() {
            self.header.mark();
            self.data.trace();
        }
    }

    /// Trace inner data
    pub(crate) unsafe fn weak_trace_inner(&self, queue: &mut Vec<NonNull<HeapBox<dyn Trace>>>) {
        self.data.weak_trace(queue);
    }

    /// Increases the root count on this `GcBox`.
    /// Roots prevent the `GcBox` from being destroyed by the garbage collector.
    pub(crate) unsafe fn root_inner(&self) {
        self.header.inc_roots();
    }

    /// Decreases the root count on this `GcBox`.
    /// Roots prevent the `GcBox` from being destroyed by the garbage collector.
    pub(crate) unsafe fn unroot_inner(&self) {
        self.header.dec_roots();
    }

    /// Returns a pointer to the `GcBox`'s value, without dereferencing it.
    pub(crate) fn value_ptr(this: *const HeapBox<T>) -> *const T {
        unsafe { ptr::addr_of!((*this).data) }
    }

    /// Returns a reference to the `GcBox`'s value.
    pub(crate) fn value(&self) -> &T {
        &self.data
    }

    pub(crate) fn is_marked(&self) -> bool {
        self.header.is_marked()
    }
}