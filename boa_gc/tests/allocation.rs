use std::ops::Deref;

use boa_gc::{BoaAlloc, Trace, Finalize};

#[test]
fn gc_basic_cell_allocation() {
    let gc_cell = BoaAlloc::new_cell("Hi");

    assert_eq!(*gc_cell.borrow_mut(), "Hi");
}