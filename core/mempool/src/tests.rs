//! Tests for `boa_mempool`.
//! These are better run within Miri.

use crate::MemPoolAllocator;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;

#[test]
fn small_in_order() {
    let pool = MemPoolAllocator::<usize>::new();
    let mut objs = vec![];

    for i in 0..100 {
        objs.push(pool.alloc(i));
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
            let ptr = pool.alloc(i * j);
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
    let a = pool.alloc(2000);
    unsafe {
        a.write(1000);
    };
    let b = pool.alloc(2001);
    unsafe {
        b.write(1001);
    };
    let c = pool.alloc(2002);
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
        let a = pool.alloc([0xFFu8; 128]);

        let b = pool.alloc_unitialized();
        a.write([0xFEu8; 128]);
        b.write([0xFDu8; 128]);
        let c = pool.alloc_unitialized();
        pool.dealloc(b);
        pool.dealloc(a);
        let b = pool.alloc_unitialized();
        let a = pool.alloc_unitialized();
        a.write([0xFCu8; 128]);
        b.write([0xFBu8; 128]);
        c.write([0xFAu8; 128]);
        let d = pool.alloc_unitialized();
        a.write([0xF9u8; 128]);
        b.write([0xF8u8; 128]);
        c.write([0xF7u8; 128]);
        d.write([0xF6u8; 128]);

        let x = pool.alloc_unitialized();
        pool.dealloc(a);
        pool.dealloc(d);
        pool.dealloc_no_drop(x);
        pool.dealloc(c);
        pool.dealloc(b);
    }
}

#[test]
fn drop() {
    struct MyS {
        dropped: Rc<AtomicBool>,
    }

    impl Drop for MyS {
        fn drop(&mut self) {
            self.dropped
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    let pool = MemPoolAllocator::<MyS>::new();
    let dropped = Rc::new(AtomicBool::new(false));
    let a = pool.alloc(MyS {
        dropped: dropped.clone(),
    });

    pool.dealloc(a);
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}
