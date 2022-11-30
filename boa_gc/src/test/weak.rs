use super::run_test;
use crate::{force_collect, Ephemeron, Gc, WeakGc};

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

            assert!(weak.value().is_none());
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

#[test]
fn eph_allocation_chains() {
    run_test(|| {
        let gc_value = Gc::new(String::from("foo"));

        {
            let cloned_gc = gc_value.clone();
            let weak = WeakGc::new(&cloned_gc);
            let wrap = Gc::new(weak);

            assert_eq!(wrap.value().expect("weak is live"), &String::from("foo"));

            let eph = Ephemeron::new(&wrap, 3);

            drop(cloned_gc);
            force_collect();

            assert_eq!(
                eph.key()
                    .expect("eph is still live")
                    .value()
                    .expect("weak is still live"),
                &String::from("foo")
            );

            drop(gc_value);
            force_collect();

            assert!(eph.key().expect("eph is still live").value().is_none());
        }
    });
}

#[test]
fn eph_basic_alloc_dump_test() {
    run_test(|| {
        let gc_value = Gc::new(String::from("gc here"));
        let _gc_two = Gc::new("hmmm");

        let eph = Ephemeron::new(&gc_value, 4);
        let _fourth = Gc::new("tail");

        assert_eq!(*eph.key().expect("must be live"), String::from("gc here"));
    });
}

#[test]
fn eph_basic_upgrade_test() {
    run_test(|| {
        let init_gc = Gc::new(String::from("foo"));

        let weak = WeakGc::new(&init_gc);

        let new_gc = weak.upgrade().expect("Weak is still live");

        drop(weak);
        force_collect();

        assert_eq!(*init_gc, *new_gc);
    });
}

#[test]
fn eph_basic_clone_test() {
    run_test(|| {
        let init_gc = Gc::new(String::from("bar"));

        let weak = WeakGc::new(&init_gc);

        let new_gc = weak.upgrade().expect("Weak is live");
        let new_weak = weak.clone();

        drop(weak);
        force_collect();

        assert_eq!(*new_gc, *new_weak.value().expect("weak should be live"));
        assert_eq!(
            *init_gc,
            *new_weak.value().expect("weak_should be live still")
        );
    });
}
