use crate::BOA_GC;

mod allocation;

struct Harness;

impl Harness {
    pub fn assert_collections(o: usize) {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert_eq!(gc.runtime.collections, o);
        })
    }

    pub fn assert_empty_gc() {
        BOA_GC.with(|current| {
            let gc = current.borrow();

            assert!(gc.adult_start.get().is_none());
            assert!(gc.runtime.total_bytes_allocated == 0);
        })
    }

    pub fn assert_bytes_allocated() {
        BOA_GC.with(|current| {
            let gc = current.borrow();
            assert!(gc.runtime.total_bytes_allocated > 0);
        })
    }
}
