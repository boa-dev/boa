//! Tests for `boa_mempool`.
//! These are better run within Miri.

use crate::MemPoolAllocator;

#[test]
fn allocates_small_objects() {
    let pool = MemPoolAllocator::<usize>::new();

    for _ in 0..100 {
        let _ = pool.alloc();
        // Oops, leaking memory. Oh well.
    }

    let total = pool.allocated();
    assert_eq!(pool.available(), total - 100);
}

#[test]
fn allocates_and_deallocates() {
    let pool = MemPoolAllocator::<usize>::new();
    let mut objs = vec![];

    for _ in 0..100 {
        objs.push(pool.alloc());
    }

    let total = pool.allocated();
    assert_eq!(pool.available(), total - 100);

    for p in objs {
        pool.dealloc(p);
    }

    assert_eq!(pool.available(), pool.allocated());
    // Deallocating should not change the amount of memory used.
    assert_eq!(pool.allocated(), total);
}
