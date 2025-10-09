//! Tests for `boa_mempool`.
//! These are better run within Miri.

use crate::MemPoolAllocator;

#[test]
fn small_in_order() {
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

#[test]
fn realloc_loops() {
    let pool = MemPoolAllocator::<usize>::new();

    for i in 0..32 {
        let mut objs = vec![];

        for j in 0..(i * 16) {
            let ptr = pool.alloc();
            unsafe { ptr.write(i * j) };
            objs.push(ptr);
        }

        let total = pool.allocated();
        assert_eq!(pool.available(), total - objs.len());

        for p in objs {
            pool.dealloc(p);
        }

        assert_eq!(pool.available(), pool.allocated());
        // Deallocating should not change the amount of memory used.
        assert_eq!(pool.allocated(), total);
    }
}

#[test]
fn simple() {
    let pool = MemPoolAllocator::<usize>::new();
    let a = pool.alloc();
    unsafe {
        a.write(1000);
    };
    let b = pool.alloc();
    unsafe {
        b.write(1001);
    };
    let c = pool.alloc();
    unsafe {
        c.write(1002);
    };

    pool.dealloc(c);
    pool.dealloc(b);
    pool.dealloc(a);
}

#[test]
fn array() {
    unsafe {
        let pool = MemPoolAllocator::<[u8; 128]>::new();
        let a = pool.alloc();
        a.write([0xFFu8; 128]);

        let b = pool.alloc();
        a.write([0xFEu8; 128]);
        b.write([0xFDu8; 128]);
        let c = pool.alloc();
        pool.dealloc(b);
        pool.dealloc(a);
        let b = pool.alloc();
        let a = pool.alloc();
        a.write([0xFCu8; 128]);
        b.write([0xFBu8; 128]);
        c.write([0xFAu8; 128]);
        let d = pool.alloc();
        a.write([0xF9u8; 128]);
        b.write([0xF8u8; 128]);
        c.write([0xF7u8; 128]);
        d.write([0xF6u8; 128]);

        let x = pool.alloc();
        pool.dealloc(a);
        pool.dealloc(d);
        pool.dealloc(x);
        pool.dealloc(c);
        pool.dealloc(b);
    }
}
