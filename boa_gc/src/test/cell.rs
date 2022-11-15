use boa_gc::{Gc, GcCell};

use super::run_test;

#[test]
fn boa_borrow_mut_test() {
    run_test(|| {
        let v = Gc::new(GcCell::new(Vec::new()));

        for _ in 1..=259 {
            let cell = Gc::new(GcCell::new([0u8; 10]));
            v.borrow_mut().push(cell);
        }
    });
}
