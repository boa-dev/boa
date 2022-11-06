use boa_gc::BoaAlloc;

#[test]
fn boa_borrow_mut_test() {
    let v = BoaAlloc::new_cell(Vec::new());

    for _ in 1..=259 {
        let cell = BoaAlloc::new_cell([0u8; 10]);
        v.borrow_mut().push(cell);
    }
}
