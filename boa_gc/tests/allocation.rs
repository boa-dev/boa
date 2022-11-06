use boa_gc::{force_collect, BoaAlloc, GcTester};

#[test]
fn gc_basic_cell_allocation() {
    let gc_cell = BoaAlloc::new_cell(16_u16);

    force_collect();
    GcTester::assert_collections(1);
    GcTester::assert_adult_bytes_allocated();
    assert_eq!(*gc_cell.borrow_mut(), 16);
}

#[test]
fn gc_basic_pointer_alloc() {
    let gc = BoaAlloc::new(16_u8);

    force_collect();
    GcTester::assert_collections(1);
    GcTester::assert_adult_bytes_allocated();
    assert_eq!(*gc, 16);

    drop(gc);
    force_collect();
    GcTester::assert_collections(2);
    GcTester::assert_empty_gc();
}
