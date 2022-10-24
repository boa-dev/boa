use crate::{Finalize, Trace};
use std::cell::Cell;
use std::ptr::{self, NonNull};

// Age and Weak Flags
const WEAK_MASK: u8 = 1 << 7;
const AGE_MASK: u8 = !WEAK_MASK;
const AGE_MAX: u8 = AGE_MASK;

const MARK_MASK: usize = 1 << (usize::BITS - 1);
const ROOTS_MASK: usize = !MARK_MASK;
const ROOTS_MAX: usize = ROOTS_MASK;

pub enum BoxLoc {
    Stack,
    Heap,
}

pub(crate) struct GcBoxHeader {
    roots: Cell<usize>,
    cycle_age: Cell<u8>,
    loc: Cell<BoxLoc>,
    pub(crate) next: Cell<Option<NonNull<GcBox<dyn Trace>>>>,
}

impl GcBoxHeader {
    #[inline]
    pub fn new() -> Self {
        // TODO: implement a way for a cell to start out weak with WEAK_MASK
        GcBoxHeader {
            roots: Cell::new(1),
            cycle_age: Cell::new(0_u8),
            loc: Cell::new(BoxLoc::Stack),
            next: Cell::new(None),
        }
    }

    #[inline]
    pub fn promote(&self, next: Option<NonNull<GcBox<dyn Trace>>>) {
        self.loc.set(BoxLoc::Heap);
        self.next.set(next);
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
    pub fn age(&self) -> u8 {
        &self.cycle_age.get() & AGE_MASK
    }

    #[inline]
    pub fn inc_age(&self) {
        let age = self.cycle_age.get();

        // There is no need to increment the age after hitting max age
        if (age & AGE_MASK) < AGE_MAX {
            self.cycle_age.set(age + 1);
        }
    }

    #[inline]
    pub fn is_ephemeron(&self) -> bool {
        self.cycle_age.get() & WEAK_MASK != 0
    }
}

/// The HeapBox represents a box on the GC Heap. The HeapBox's creation and allocation is managed
/// by the allocator
pub struct GcBox<T: Trace + ?Sized + 'static> {
    pub(crate) header: GcBoxHeader,
    pub(crate) value: T,
}

impl<T: Trace> GcBox<T> {
    pub(crate) fn new(value: T) -> Self {
        GcBox {
            header: GcBoxHeader::new(),
            value,
        }
    }
}

impl<T: Trace + ?Sized> GcBox<T> {
    /// Returns `true` if the two references refer to the same `GcBox`.
    pub(crate) fn ptr_eq(this: &GcBox<T>, other: &GcBox<T>) -> bool {
        // Use .header to ignore fat pointer vtables, to work around
        // https://github.com/rust-lang/rust/issues/46139
        ptr::eq(&this.header, &other.header)
    }

    pub(crate) fn promote(&self, next: Option<NonNull<GcBox<dyn Trace>>>) {
        self.header.promote(next)
    }

    /// Marks this `GcBox` and marks through its data.
    pub(crate) unsafe fn trace_inner(&self) {
        if !self.header.is_marked() && !self.header.is_ephemeron() {
            self.header.mark();
            self.value.trace();
        }
    }

    /// Trace inner data
    pub(crate) unsafe fn weak_trace_inner(&self, queue: &mut Vec<NonNull<GcBox<dyn Trace>>>) {
        self.value.weak_trace(queue);
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
    pub(crate) fn value_ptr(this: *const GcBox<T>) -> *const T {
        unsafe { ptr::addr_of!((*this).value) }
    }

    /// Returns a reference to the `GcBox`'s value.
    pub(crate) fn value(&self) -> &T {
        &self.value
    }

    pub(crate) fn is_marked(&self) -> bool {
        self.header.is_marked()
    }
}
