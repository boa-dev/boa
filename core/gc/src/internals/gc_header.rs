use std::{cell::Cell, fmt, hint};

const MARK_MASK: u32 = 1 << (u32::BITS - 1);
const NON_ROOTS_MASK: u32 = !MARK_MASK;
const NON_ROOTS_MAX: u32 = NON_ROOTS_MASK;

/// The `Gcheader` contains the `GcBox`'s and `EphemeronBox`'s current state for the `Collector`'s
/// Mark/Sweep as well as a pointer to the next node in the heap.
///
/// `ref_count` is the number of Gc instances, and `non_root_count` is the number of
/// Gc instances in the heap. `non_root_count` also includes Mark Flag bit.
///
/// The next node is set by the `Allocator` during initialization and by the
/// `Collector` during the sweep phase.
pub(crate) struct GcHeader {
    ref_count: Cell<u32>,
    non_root_count: Cell<u32>,
}

impl GcHeader {
    /// Creates a new [`GcHeader`] with a root of 1 and next set to None.
    pub(crate) fn new() -> Self {
        Self {
            ref_count: Cell::new(1),
            non_root_count: Cell::new(0),
        }
    }

    /// Returns the [`GcHeader`]'s current ref count.
    pub(crate) fn ref_count(&self) -> u32 {
        self.ref_count.get()
    }

    /// Returns the [`GcHeader`]'s current non-roots count
    pub(crate) fn non_root_count(&self) -> u32 {
        self.non_root_count.get() & NON_ROOTS_MASK
    }

    /// Increments [`GcHeader`]'s non-roots count.
    pub(crate) fn inc_non_root_count(&self) {
        let non_root_count = self.non_root_count.get();

        if (non_root_count & NON_ROOTS_MASK) < NON_ROOTS_MAX {
            self.non_root_count.set(non_root_count.wrapping_add(1));
        } else {
            // TODO: implement a better way to handle root overload.
            panic!("non-roots counter overflow");
        }
    }

    /// Decreases [`GcHeader`]'s current non-roots count.
    pub(crate) fn reset_non_root_count(&self) {
        self.non_root_count
            .set(self.non_root_count.get() & !NON_ROOTS_MASK);
    }

    /// Returns a bool for whether [`GcHeader`]'s mark bit is 1.
    pub(crate) fn is_marked(&self) -> bool {
        self.non_root_count.get() & MARK_MASK != 0
    }

    pub(crate) fn inc_ref_count(&self) {
        // Mark this as `cold` since the ref count will
        // (almost) never overflow.
        #[cold]
        #[inline(never)]
        fn overflow_panic() {
            panic!("ref count overflow")
        }

        let count = self.ref_count.get();

        // SAFETY: The reference count will never be zero when this is
        // called.
        unsafe {
            hint::assert_unchecked(count != 0);
        }

        self.ref_count.set(count.wrapping_add(1));

        if count == 0 {
            overflow_panic();
        }
    }

    pub(crate) fn dec_ref_count(&self) {
        self.ref_count.set(self.ref_count.get() - 1);
    }

    /// Check if the gc object is rooted.
    ///
    /// # Note
    ///
    /// This only gives valid result if the we have run through the
    /// tracing non roots phase.
    pub(crate) fn is_rooted(&self) -> bool {
        self.non_root_count() < self.ref_count()
    }

    /// Sets [`GcHeader`]'s mark bit to 1.
    pub(crate) fn mark(&self) {
        self.non_root_count
            .set(self.non_root_count.get() | MARK_MASK);
    }

    /// Sets [`GcHeader`]'s mark bit to 0.
    pub(crate) fn unmark(&self) {
        self.non_root_count
            .set(self.non_root_count.get() & !MARK_MASK);
    }
}

impl fmt::Debug for GcHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GcHeader")
            .field("marked", &self.is_marked())
            .field("ref_count", &self.ref_count.get())
            .field("non_root_count", &self.non_root_count())
            .finish_non_exhaustive()
    }
}
