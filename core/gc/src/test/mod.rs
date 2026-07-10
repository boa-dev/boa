use crate::BOA_GC;

mod allocation;
mod cell;
mod erased;
mod std_types;
mod weak;
mod weak_map;

struct Harness;

impl Harness {
    #[track_caller]
    fn assert_collections(o: usize) {
        let collections = BOA_GC.with(|current| {
            let gc = current.borrow();
            gc.runtime.collections
        });

        assert_eq!(collections, o);
    }

    #[track_caller]
    fn assert_empty_gc() {
        let (is_empty, bytes_allocated) = BOA_GC.with(|current| {
            let gc = current.borrow();
            (gc.strongs.is_empty(), gc.runtime.bytes_allocated)
        });

        assert!(is_empty);
        assert_eq!(bytes_allocated, 0);
    }

    #[track_caller]
    fn assert_bytes_allocated() {
        let bytes_allocated = BOA_GC.with(|current| {
            let gc = current.borrow();
            gc.runtime.bytes_allocated
        });

        assert!(bytes_allocated > 0);
    }

    #[track_caller]
    fn assert_exact_bytes_allocated(bytes: usize) {
        let bytes_allocated = BOA_GC.with(|current| {
            let gc = current.borrow();
            gc.runtime.bytes_allocated
        });
        assert_eq!(bytes_allocated, bytes);
    }
}

#[track_caller]
fn run_test(test: impl FnOnce() + Send + 'static) {
    let handle = std::thread::spawn(test);
    handle.join().unwrap();
}
