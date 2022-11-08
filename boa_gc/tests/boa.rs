use boa_gc::Gc;

#[test]
fn boa_borrow_mut_test() {
    let v = Gc::new_cell(Vec::new());

    for _ in 1..=259 {
        let cell = Gc::new_cell([0u8; 10]);
        v.borrow_mut().push(cell);
    }
}
