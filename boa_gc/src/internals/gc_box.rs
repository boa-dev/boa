use crate::Trace;
use std::{
    cell::Cell,
    fmt,
    ptr::{self, NonNull},
};

const MARK_MASK: u32 = 1 << (u32::BITS - 1);
const NON_ROOTS_MASK: u32 = !MARK_MASK;
const NON_ROOTS_MAX: u32 = NON_ROOTS_MASK;

/// The `GcBoxheader` contains the `GcBox`'s current state for the `Collector`'s
/// Mark/Sweep as well as a pointer to the next node in the heap.
///
/// `ref_count` is the number of Gc instances, and `non_root_count` is the number of
/// Gc instances in the heap. `non_root_count` also includes Mark Flag bit.
///
/// The next node is set by the `Allocator` during initialization and by the
/// `Collector` during the sweep phase.
pub(crate) struct GcBoxHeader {
    ref_count: Cell<u32>,
    non_root_count: Cell<u32>,
    pub(crate) next: Cell<Option<NonNull<GcBox<dyn Trace>>>>,
}

impl GcBoxHeader {
    /// Creates a new `GcBoxHeader` with a root of 1 and next set to None.
    pub(crate) fn new() -> Self {
        Self {
            ref_count: Cell::new(1),
            non_root_count: Cell::new(0),
            next: Cell::new(None),
        }
    }

    /// Returns the `GcBoxHeader`'s current non-roots count
    pub(crate) fn get_non_root_count(&self) -> u32 {
        self.non_root_count.get() & NON_ROOTS_MASK
    }

    /// Increments `GcBoxHeader`'s non-roots count.
    pub(crate) fn inc_non_root_count(&self) {
        let non_root_count = self.non_root_count.get();

        if (non_root_count & NON_ROOTS_MASK) < NON_ROOTS_MAX {
            self.non_root_count.set(non_root_count.wrapping_add(1));
        } else {
            // TODO: implement a better way to handle root overload.
            panic!("non-roots counter overflow");
        }
    }

    /// Decreases `GcBoxHeader`'s current non-roots count.
    pub(crate) fn reset_non_root_count(&self) {
        self.non_root_count
            .set(self.non_root_count.get() & !NON_ROOTS_MASK);
    }

    /// Returns a bool for whether `GcBoxHeader`'s mark bit is 1.
    pub(crate) fn is_marked(&self) -> bool {
        self.non_root_count.get() & MARK_MASK != 0
    }

    /// Sets `GcBoxHeader`'s mark bit to 1.
    pub(crate) fn mark(&self) {
        self.non_root_count
            .set(self.non_root_count.get() | MARK_MASK);
    }

    /// Sets `GcBoxHeader`'s mark bit to 0.
    pub(crate) fn unmark(&self) {
        self.non_root_count
            .set(self.non_root_count.get() & !MARK_MASK);
    }
}

impl fmt::Debug for GcBoxHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GcBoxHeader")
            .field("marked", &self.is_marked())
            .field("ref_count", &self.ref_count.get())
            .field("non_root_count", &self.get_non_root_count())
            .finish_non_exhaustive()
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
        self.header.get_non_root_count()
    }

    #[inline]
    pub(crate) fn inc_non_root_count(&self) {
        self.header.inc_non_root_count();
    }

    pub(crate) fn reset_non_root_count(&self) {
        self.header.reset_non_root_count();
    }
}
