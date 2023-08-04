use crate::BOA_GC;

mod allocation;
mod cell;
mod weak;
mod weak_map;

struct Harness;

impl Harness {
    #[track_caller]
    fn assert_collections(o: usize) {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert_eq!(gc.runtime.collections, o);
        });
    }

    #[track_caller]
    fn assert_empty_gc() {
        BOA_GC.with(|current| {
            let gc = current.borrow();

            assert!(gc.strong_start.get().is_none());
            assert!(gc.runtime.bytes_allocated == 0);
        });
    }

    #[track_caller]
    fn assert_bytes_allocated() {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert!(gc.runtime.bytes_allocated > 0);
        });
    }

    #[track_caller]
    fn assert_exact_bytes_allocated(bytes: usize) {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert_eq!(gc.runtime.bytes_allocated, bytes);
        });
    }
}

fn run_test(test: impl FnOnce() + Send + 'static) {
    let handle = std::thread::spawn(test);
    handle.join().unwrap();
}
