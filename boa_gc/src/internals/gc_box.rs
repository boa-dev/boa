use crate::Trace;
use std::{
    cell::Cell,
    fmt,
    ptr::{self, NonNull},
};

// Age and Weak Flags
const MARK_MASK: usize = 1 << (usize::BITS - 1);
const ROOTS_MASK: usize = !MARK_MASK;
const ROOTS_MAX: usize = ROOTS_MASK;

/// The `GcBoxheader` contains the `GcBox`'s current state for the `Collector`'s
/// Mark/Sweep as well as a pointer to the next node in the heap.
///
/// These flags include:
///  - Root Count
///  - Mark Flag Bit
///
/// The next node is set by the `Allocator` during initialization and by the
/// `Collector` during the sweep phase.
pub(crate) struct GcBoxHeader {
    roots: Cell<usize>,
    pub(crate) next: Cell<Option<NonNull<GcBox<dyn Trace>>>>,
}

impl GcBoxHeader {
    /// Creates a new `GcBoxHeader` with a root of 1 and next set to None.
    pub(crate) fn new() -> Self {
        Self {
            roots: Cell::new(1),
            next: Cell::new(None),
        }
    }

    /// Returns the `GcBoxHeader`'s current root count
    pub(crate) fn roots(&self) -> usize {
        self.roots.get() & ROOTS_MASK
    }

    /// Increments `GcBoxHeader`'s root count.
    pub(crate) fn inc_roots(&self) {
        let roots = self.roots.get();

        if (roots & ROOTS_MASK) < ROOTS_MAX {
            self.roots.set(roots + 1);
        } else {
            // TODO: implement a better way to handle root overload.
            panic!("roots counter overflow");
        }
    }

    /// Decreases `GcBoxHeader`'s current root count.
    pub(crate) fn dec_roots(&self) {
        // Underflow check as a stop gap for current issue when dropping.
        if self.roots.get() > 0 {
            self.roots.set(self.roots.get() - 1);
        }
    }

    /// Returns a bool for whether `GcBoxHeader`'s mark bit is 1.
    pub(crate) fn is_marked(&self) -> bool {
        self.roots.get() & MARK_MASK != 0
    }

    /// Sets `GcBoxHeader`'s mark bit to 1.
    pub(crate) fn mark(&self) {
        self.roots.set(self.roots.get() | MARK_MASK);
    }

    /// Sets `GcBoxHeader`'s mark bit to 0.
    pub(crate) fn unmark(&self) {
        self.roots.set(self.roots.get() & !MARK_MASK);
    }
}

impl fmt::Debug for GcBoxHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GcBoxHeader")
            .field("roots", &self.roots())
            .field("marked", &self.is_marked())
            .finish()
    }
}

/// A garbage collected allocation.
#[derive(Debug)]
pub struct GcBox<T: Trace + ?Sized + 'static> {
    pub(crate) header: GcBoxHeader,
    value: T,
}

impl<T: Trace> GcBox<T> {
    /// Returns a new `GcBox` with a rooted `GcBoxHeader`.
    pub(crate) fn new(value: T) -> Self {
        Self {
            header: GcBoxHeader::new(),
            value,
        }
    }
}

impl<T: Trace + ?Sized> GcBox<T> {
    /// Returns `true` if the two references refer to the same `GcBox`.
    pub(crate) fn ptr_eq(this: &Self, other: &Self) -> bool {
        // Use .header to ignore fat pointer vtables, to work around
        // https://github.com/rust-lang/rust/issues/46139
        ptr::eq(&this.header, &other.header)
    }

    /// Marks this `GcBox` and traces its value.
    pub(crate) unsafe fn mark_and_trace(&self) {
        if !self.header.is_marked() {
            self.header.mark();
            // SAFETY: if `GcBox::trace_inner()` has been called, then,
            // this box must have been deemed as reachable via tracing
            // from a root, which by extension means that value has not
            // been dropped either.
            unsafe {
                self.value.trace();
            }
        }
    }

    /// Increases the root count on this `GcBox`.
    ///
    /// Roots prevent the `GcBox` from being destroyed by the garbage collector.
    pub(crate) fn root(&self) {
        self.header.inc_roots();
    }

    /// Decreases the root count on this `GcBox`.
    ///
    /// Roots prevent the `GcBox` from being destroyed by the garbage collector.
    pub(crate) fn unroot(&self) {
        self.header.dec_roots();
    }

    /// Returns a reference to the `GcBox`'s value.
    pub(crate) const fn value(&self) -> &T {
        &self.value
    }

    /// Returns `true` if the header is marked.
    pub(crate) fn is_marked(&self) -> bool {
        self.header.is_marked()
    }
}
