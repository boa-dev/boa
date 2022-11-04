
// Let's get weird and age some heap values

use boa_gc::{BoaAlloc, force_collect, GcTester};


#[test]
fn generational_promo_one() {
    let mut storage = Vec::new();

    // Super basic loop that loads bytes and force collections
    for i in 0..200 as usize {
        let gc = BoaAlloc::new(i);
        storage.push(gc);
    }
    GcTester::assert_collection_floor(2);
    // assert that items were promoted to adults
    GcTester::assert_adult_bytes_allocated();
    drop(storage);
    force_collect();
    GcTester::assert_empty_gc()
}

#[test]
fn generational_promo_two() {
    let mut storage = Vec::new();
    for i in 0..2000 as usize {
        let gc = BoaAlloc::new(i);
        if i % 10 == 0 {
            storage.push(gc.clone())
        }
    }
    GcTester::assert_collection_floor(3);
    
    GcTester::assert_adult_bytes_allocated();
    GcTester::assert_youth_bytes_allocated();
}