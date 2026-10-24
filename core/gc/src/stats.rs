//! Garbage collector statistics, enabled by the `gc-stats` feature.

use crate::BOA_GC;
use std::time::Duration;

/// Statistics about the garbage collector on the current thread.
///
/// The garbage collector is thread-local, so each thread has its own statistics.
///
/// Counters and timings are cumulative since the thread started or since the last call
/// to [`reset_stats`](crate::reset_stats).
///
/// Byte counts are shallow: they only include the size of each `Gc` and `Ephemeron`
/// allocation itself, not heap memory owned by the allocated values (for example the
/// buffer of a `Vec` stored inside a `Gc`). The internal allocations backing `WeakMap`s
/// are not counted either.
#[derive(Default, Debug, Clone, Copy)]
#[non_exhaustive]
pub struct GcStats {
    /// Number of collections run.
    pub collections: u64,
    /// Number of collections that found unreachable objects and therefore also ran the
    /// finalize and second mark phases.
    pub finalizing_collections: u64,
    /// Total bytes allocated for `Gc` and `Ephemeron` values.
    pub total_bytes_allocated: u64,
    /// Total bytes freed by sweeps.
    pub total_bytes_freed: u64,
    /// Number of `Gc` and `Ephemeron` values allocated.
    pub objects_allocated: u64,
    /// Number of `Gc` and `Ephemeron` values freed by sweeps.
    pub objects_swept: u64,
    /// Time spent counting the references between objects in the heap, which is used to
    /// find the roots.
    pub trace_non_roots_time: Duration,
    /// Time spent in the first mark phase, which marks every object reachable from the
    /// roots.
    pub mark_time: Duration,
    /// Time spent running finalizers on unreachable objects.
    ///
    /// This only increases in collections counted by `finalizing_collections`.
    pub finalize_time: Duration,
    /// Time spent in the second mark phase, which finds objects resurrected by finalizers.
    ///
    /// This only increases in collections counted by `finalizing_collections`.
    pub remark_time: Duration,
    /// Time spent freeing unreachable objects.
    pub sweep_time: Duration,
    /// Time spent removing dead entries from weak maps and freeing dead weak maps.
    pub weak_map_prune_time: Duration,
    /// Time spent in collections overall, including work not covered by the per-phase
    /// timings.
    pub total_time: Duration,
    /// Bytes currently allocated for live `Gc` and `Ephemeron` values.
    ///
    /// This is read when the snapshot is taken and is not affected by
    /// [`reset_stats`](crate::reset_stats).
    pub bytes_allocated: usize,
}

/// Returns a snapshot of the garbage collector statistics for the current thread.
///
/// # Panics
///
/// Panics if called while the garbage collector is running on this thread, for example
/// from inside a [`Finalize::finalize`](crate::Finalize::finalize) implementation.
#[must_use]
pub fn stats() -> GcStats {
    BOA_GC.with(|current| {
        let gc = current.borrow();

        GcStats {
            bytes_allocated: gc.runtime.bytes_allocated,
            ..gc.stats
        }
    })
}

/// Resets the current thread's counters and timings to zero.
///
/// This only affects the statistics: the live allocation count and the collection
/// threshold are unchanged, so the garbage collector behaves the same afterwards.
///
/// Objects allocated before a reset can be freed after it, so once `reset_stats` has
/// been called, `total_bytes_allocated - total_bytes_freed` no longer equals
/// `bytes_allocated`.
///
/// # Panics
///
/// Panics if called while the garbage collector is running on this thread, for example
/// from inside a [`Finalize::finalize`](crate::Finalize::finalize) implementation.
pub fn reset_stats() {
    BOA_GC.with(|current| {
        let mut gc = current.borrow_mut();
        gc.stats = GcStats::default();
    });
}
