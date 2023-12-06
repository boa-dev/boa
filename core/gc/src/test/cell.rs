use super::run_test;
use crate::{Gc, GcRefCell};

#[test]
fn boa_borrow_mut_test() {
    run_test(|| {
        let v = Gc::new(GcRefCell::new(Vec::new()));

        for _ in 1..=259 {
            let cell = Gc::new(GcRefCell::new([0u8; 10]));
            v.borrow_mut().push(cell);
        }
    });
}
