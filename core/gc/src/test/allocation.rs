use boa_macros::{Finalize, Trace};

use super::{run_test, Harness};
use crate::{force_collect, Gc, GcBox, GcRefCell};

#[test]
fn gc_basic_cell_allocation() {
    run_test(|| {
        let gc_cell = Gc::new(GcRefCell::new(16_u16));

        force_collect();
        Harness::assert_collections(1);
        Harness::assert_bytes_allocated();
        assert_eq!(*gc_cell.borrow_mut(), 16);
    });
}

#[test]
fn gc_basic_pointer_alloc() {
    run_test(|| {
        let gc = Gc::new(16_u8);

        force_collect();
        Harness::assert_collections(1);
        Harness::assert_bytes_allocated();
        assert_eq!(*gc, 16);

        drop(gc);
        force_collect();
        Harness::assert_collections(2);
        Harness::assert_empty_gc();
    });
}

#[test]
// Takes too long to finish in miri
#[cfg_attr(miri, ignore)]
fn gc_recursion() {
    run_test(|| {
        #[derive(Debug, Finalize, Trace)]
        struct S {
            i: usize,
            next: Option<Gc<S>>,
        }

        const SIZE: usize = size_of::<GcBox<S>>();
        const COUNT: usize = 1_000_000;

        let mut root = Gc::new(S { i: 0, next: None });
        for i in 1..COUNT {
            root = Gc::new(S {
                i,
                next: Some(root),
            });
        }

        Harness::assert_bytes_allocated();
        Harness::assert_exact_bytes_allocated(SIZE * COUNT);

        drop(root);
        force_collect();
        Harness::assert_empty_gc();
    });
}
