use std::{cell::Cell, fmt};

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
        let non_root_count = self.non_root_count.get() & NON_ROOTS_MASK;

        // `non_root_count` must not exceed `ref_count`.
        // This prevents `is_rooted()` from returning false on live objects,
        // which would cause a UAF.
        // `inc_ref_count` caps `ref_count` at `NON_ROOTS_MAX`, ensuring
        // `ref_count` is always reachable.
        if non_root_count < self.ref_count.get() {
            self.non_root_count
                .set(self.non_root_count.get().wrapping_add(1));
        } else {
            // Saturated: `non_root_count` has reached `ref_count`.
            // The debug assertion below catches corrupted state (non_root_count > ref_count),
            // which is unreachable through this function but can occur via direct field writes
            // in unsafe code or tests.
            debug_assert_eq!(
                non_root_count,
                self.ref_count.get(),
                "non_root_count exceeded ref_count: state corruption detected \
                 (only reachable via direct field writes that bypass the saturation cap)"
            );
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
            panic!("too many references to a gc allocation");
        }

        let count = self.ref_count.get().wrapping_add(1);

        // `non_root_count` shares storage with the mark bit (using 31 bits).
        // A `ref_count` > `NON_ROOTS_MAX` would make `is_rooted()` always true,
        // leaking memory. Treat this as a hard error identically to `u32` wrap.
        // Check before writing to maintain a clean `ref_count` on `catch_unwind`.
        if count == 0 || count > NON_ROOTS_MAX {
            overflow_panic();
        }

        self.ref_count.set(count);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_bit_preserved() {
        let header = GcHeader::new();
        header.mark();
        assert!(header.is_marked());

        header.inc_non_root_count();
        assert!(header.is_marked());
        assert_eq!(header.non_root_count(), 1);

        header.inc_non_root_count();
        assert!(header.is_marked());
        assert_eq!(header.non_root_count(), 1);
    }

    #[test]
    fn reset_preserves_mark() {
        let header = GcHeader::new();
        header.inc_non_root_count();
        header.mark();

        header.reset_non_root_count();
        assert_eq!(header.non_root_count(), 0);
        assert!(header.is_marked());
    }

    #[test]
    #[should_panic(expected = "too many references to a gc allocation")]
    fn inc_ref_panics() {
        let header = GcHeader::new();
        header.ref_count.set(NON_ROOTS_MAX);

        header.inc_ref_count();
    }

    #[test]
    fn is_rooted_before_saturation() {
        let header = GcHeader::new();
        header.inc_ref_count();

        header.inc_non_root_count();
        assert!(header.is_rooted());

        header.inc_non_root_count();
        assert!(!header.is_rooted());
    }

    #[test]
    fn saturation_at_higher_ref_count() {
        let header = GcHeader::new();
        header.inc_ref_count();
        header.inc_ref_count();

        header.inc_non_root_count();
        header.inc_non_root_count();
        header.inc_non_root_count(); // saturates at ref_count
        header.inc_non_root_count(); // no-op
        assert_eq!(header.non_root_count(), 3);
        assert!(!header.is_rooted());
    }

    #[test]
    fn unmark_preserves_non_root_count() {
        let header = GcHeader::new();
        header.inc_ref_count();
        header.inc_non_root_count();
        header.mark();
        header.unmark();
        assert_eq!(header.non_root_count(), 1);
    }

    /// Verifies `debug_assert!` panics if `inc_non_root_count` exceeds `ref_count`.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "non_root_count exceeded ref_count: state corruption detected")]
    fn debug_assert_fires_when_non_root_exceeds_ref_count() {
        let header = GcHeader::new();
        // Corrupt the state to bypass the cap.
        header.non_root_count.set(2);
        header.inc_non_root_count(); // triggers debug_assert_eq!
    }
}
