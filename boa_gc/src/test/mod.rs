use crate::BOA_GC;

mod allocation;
mod cell;
mod weak;
mod weak_map;

struct Harness;

impl Harness {
    fn assert_collections(o: usize) {
        BOA_GC.with(|gc| {
            assert_eq!(gc.runtime.collections.get(), o);
        });
    }

    fn assert_empty_gc() {
        BOA_GC.with(|gc| {
            assert!(gc.strong_start.get().is_none());
            assert_eq!(gc.runtime.bytes_allocated.get(), 0);
        });
    }

    fn assert_bytes_allocated() {
        BOA_GC.with(|gc| {
            assert!(gc.runtime.bytes_allocated.get() > 0);
        });
    }

    fn assert_exact_bytes_allocated(bytes: usize) {
        BOA_GC.with(|gc| {
            assert_eq!(gc.runtime.bytes_allocated.get(), bytes);
        });
    }
}

fn run_test(test: impl FnOnce() + Send + 'static) {
    let handle = std::thread::spawn(test);
    handle.join().unwrap();
}
