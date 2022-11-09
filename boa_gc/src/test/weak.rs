use boa_gc::{force_collect, Ephemeron, Gc, WeakGc};

use super::run_test;

#[test]
fn eph_weak_gc_test() {
    run_test(|| {
        let gc_value = Gc::new(3);

        {
            let cloned_gc = gc_value.clone();

            let weak = WeakGc::new(&cloned_gc);

            assert_eq!(*weak.value().expect("Is live currently"), 3);
            drop(cloned_gc);
            force_collect();
            assert_eq!(*weak.value().expect("WeakGc is still live here"), 3);

            drop(gc_value);
            force_collect();

            assert!(weak.value().is_none())
        }
    });
}

#[test]
fn eph_ephemeron_test() {
    run_test(|| {
        let gc_value = Gc::new(3);

        {
            let cloned_gc = gc_value.clone();

            let ephemeron = Ephemeron::new(&cloned_gc, String::from("Hello World!"));

            assert_eq!(*ephemeron.key().expect("Ephemeron is live"), 3);
            assert_eq!(*ephemeron.value(), String::from("Hello World!"));
            drop(cloned_gc);
            force_collect();
            assert_eq!(*ephemeron.key().expect("Ephemeron is still live here"), 3);

            drop(gc_value);
            force_collect();

            assert!(ephemeron.key().is_none());
        }
    });
}
