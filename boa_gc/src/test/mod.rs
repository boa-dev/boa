use crate::BOA_GC;

mod allocation;
mod cell;
mod weak;

struct Harness;

impl Harness {
    fn assert_collections(o: usize) {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert_eq!(gc.runtime.collections, o);
        });
    }

    fn assert_empty_gc() {
        BOA_GC.with(|current| {
            let gc = current.borrow();

            assert!(gc.adult_start.get().is_none());
            assert!(gc.runtime.bytes_allocated == 0);
        });
    }

    fn assert_bytes_allocated() {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert!(gc.runtime.bytes_allocated > 0);
        });
    }
}

fn run_test(test: impl FnOnce() + Send + 'static) {
    let handle = std::thread::spawn(test);
    handle.join().unwrap();
}
