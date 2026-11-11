mod miri {
    use std::time::Duration;

    use super::super::run_test;
    use crate::{Gc, GcBox, WeakGc, force_collect, internals::EphemeronBox, reset_stats, stats};

    const COUNT: usize = 100;
    const GC_BOX_SIZE: usize = size_of::<GcBox<u64>>();

    fn allocate(count: usize) -> Vec<Gc<u64>> {
        (0..count as u64).map(Gc::new).collect()
    }

    #[test]
    fn stats_count_allocations() {
        run_test(|| {
            let values = allocate(COUNT);

            let snapshot = stats();
            assert_eq!(snapshot.objects_allocated, COUNT as u64);
            assert_eq!(snapshot.total_bytes_allocated, (COUNT * GC_BOX_SIZE) as u64);
            assert_eq!(snapshot.bytes_allocated, COUNT * GC_BOX_SIZE);
            assert_eq!(snapshot.collections, 0);

            drop(values);
        });
    }

    #[test]
    fn stats_collection_without_garbage() {
        run_test(|| {
            let values = allocate(COUNT);

            force_collect();

            let snapshot = stats();
            assert_eq!(snapshot.collections, 1);
            assert_eq!(snapshot.finalizing_collections, 0);
            assert_eq!(snapshot.total_bytes_freed, 0);
            assert_eq!(snapshot.objects_swept, 0);
            assert_eq!(snapshot.bytes_allocated, COUNT * GC_BOX_SIZE);

            drop(values);
        });
    }

    #[test]
    fn stats_free_everything() {
        run_test(|| {
            drop(allocate(COUNT));

            force_collect();

            let snapshot = stats();
            assert_eq!(snapshot.collections, 1);
            assert_eq!(snapshot.finalizing_collections, 1);
            assert_eq!(snapshot.total_bytes_freed, snapshot.total_bytes_allocated);
            assert_eq!(snapshot.objects_swept, snapshot.objects_allocated);
            assert_eq!(snapshot.bytes_allocated, 0);

            // `force_collect` skips the collection when nothing is allocated.
            force_collect();
            assert_eq!(stats().collections, 1);
        });
    }

    #[test]
    fn stats_count_ephemerons() {
        run_test(|| {
            let value = Gc::new(1u8);
            let weak = WeakGc::new(&value);
            let expected_bytes = size_of::<GcBox<u8>>() + size_of::<EphemeronBox<u8, ()>>();

            let snapshot = stats();
            assert_eq!(snapshot.objects_allocated, 2);
            assert_eq!(snapshot.total_bytes_allocated, expected_bytes as u64);
            assert_eq!(snapshot.bytes_allocated, expected_bytes);

            drop(weak);
            drop(value);
            force_collect();

            let snapshot = stats();
            assert_eq!(snapshot.objects_swept, 2);
            assert_eq!(snapshot.total_bytes_freed, expected_bytes as u64);
            assert_eq!(snapshot.bytes_allocated, 0);
        });
    }

    #[test]
    fn stats_reset() {
        run_test(|| {
            let values = allocate(COUNT);

            force_collect();
            assert_eq!(stats().collections, 1);

            reset_stats();

            let snapshot = stats();
            assert_eq!(snapshot.collections, 0);
            assert_eq!(snapshot.finalizing_collections, 0);
            assert_eq!(snapshot.total_bytes_allocated, 0);
            assert_eq!(snapshot.total_bytes_freed, 0);
            assert_eq!(snapshot.objects_allocated, 0);
            assert_eq!(snapshot.objects_swept, 0);
            assert_eq!(snapshot.trace_non_roots_time, Duration::ZERO);
            assert_eq!(snapshot.mark_time, Duration::ZERO);
            assert_eq!(snapshot.finalize_time, Duration::ZERO);
            assert_eq!(snapshot.remark_time, Duration::ZERO);
            assert_eq!(snapshot.sweep_time, Duration::ZERO);
            assert_eq!(snapshot.weak_map_prune_time, Duration::ZERO);
            assert_eq!(snapshot.total_time, Duration::ZERO);
            // The live byte count is not a statistic, so a reset leaves it unchanged.
            assert_eq!(snapshot.bytes_allocated, COUNT * GC_BOX_SIZE);

            drop(values);
        });
    }

    #[test]
    fn stats_phase_times_within_total() {
        run_test(|| {
            drop(allocate(COUNT));

            force_collect();

            let snapshot = stats();
            assert_eq!(snapshot.finalizing_collections, 1);

            let phases = snapshot.trace_non_roots_time
                + snapshot.mark_time
                + snapshot.finalize_time
                + snapshot.remark_time
                + snapshot.sweep_time
                + snapshot.weak_map_prune_time;
            assert!(phases <= snapshot.total_time);
        });
    }
}
