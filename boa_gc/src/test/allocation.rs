use super::{run_test, Harness};
use crate::{force_collect, Gc, GcCell};

#[test]
fn gc_basic_cell_allocation() {
    run_test(|| {
        let gc_cell = Gc::new(GcCell::new(16_u16));

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
