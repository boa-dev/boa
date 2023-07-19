use crate::Trace;
use std::{
    cell::Cell,
    fmt,
    ptr::{self, NonNull},
};

/// The `GcBoxheader` contains the `GcBox`'s current state for the `Collector`'s
/// Mark/Sweep as well as a pointer to the next node in the heap.
///
/// These flags include:
///  - Mark Flag Bit
///
/// The next node is set by the `Allocator` during initialization and by the
/// `Collector` during the sweep phase.
pub(crate) struct GcBoxHeader {
    marked: Cell<bool>,
    ref_count: Cell<u32>,
    non_root_count: Cell<u32>,
    pub(crate) next: Cell<Option<NonNull<GcBox<dyn Trace>>>>,
}

impl GcBoxHeader {
    /// Creates a new `GcBoxHeader` with a root of 1 and next set to None.
    pub(crate) fn new() -> Self {
        Self {
            marked: Cell::new(false),
            ref_count: Cell::new(1),
            non_root_count: Cell::new(0),
            next: Cell::new(None),
        }
    }

    /// Returns a bool for whether `GcBoxHeader`'s mark bit is 1.
    pub(crate) fn is_marked(&self) -> bool {
        self.marked.get()
    }

    /// Sets `GcBoxHeader`'s mark bit to 1.
    pub(crate) fn mark(&self) {
        self.marked.set(true);
    }

    /// Sets `GcBoxHeader`'s mark bit to 0.
    pub(crate) fn unmark(&self) {
        self.marked.set(false);
    }
}

impl fmt::Debug for GcBoxHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GcBoxHeader")
            .field("marked", &self.is_marked())
            .field("ref_count", &self.ref_count.get())
            .field("non_root_count", &self.non_root_count.get())
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

    /// Returns a reference to the `GcBox`'s value.
    pub(crate) const fn value(&self) -> &T {
        &self.value
    }

    /// Returns `true` if the header is marked.
    pub(crate) fn is_marked(&self) -> bool {
        self.header.is_marked()
    }

    #[inline]
    pub(crate) fn get_ref_count(&self) -> u32 {
        self.header.ref_count.get()
    }

    #[inline]
    pub(crate) fn inc_ref_count(&self) {
        self.header.ref_count.set(self.header.ref_count.get() + 1);
    }

    #[inline]
    pub(crate) fn dec_ref_count(&self) {
        self.header.ref_count.set(self.header.ref_count.get() - 1);
    }

    #[inline]
    pub(crate) fn get_non_root_count(&self) -> u32 {
        self.header.non_root_count.get()
    }

    #[inline]
    pub(crate) fn inc_non_root_count(&self) {
        self.header
            .non_root_count
            .set(self.header.non_root_count.get() + 1);
    }

    pub(crate) fn reset_non_root_count(&self) {
        self.header.non_root_count.set(0);
    }
}
