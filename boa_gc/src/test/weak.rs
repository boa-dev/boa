use super::run_test;
use crate::{
    force_collect, test::Harness, Ephemeron, Finalize, Gc, GcBox, GcRefCell, Trace, WeakGc,
};

#[test]
fn eph_weak_gc_test() {
    run_test(|| {
        let gc_value = Gc::new(3);

        {
            let cloned_gc = gc_value.clone();

            let weak = WeakGc::new(&cloned_gc);

            assert_eq!(*weak.upgrade().expect("Is live currently"), 3);
            drop(cloned_gc);
            force_collect();
            assert_eq!(*weak.upgrade().expect("WeakGc is still live here"), 3);

            drop(gc_value);
            force_collect();

            assert!(weak.upgrade().is_none());
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

            assert_eq!(
                *ephemeron.value().expect("Ephemeron is live"),
                String::from("Hello World!")
            );
            drop(cloned_gc);
            force_collect();
            assert_eq!(
                *ephemeron.value().expect("Ephemeron is still live here"),
                String::from("Hello World!")
            );

            drop(gc_value);
            force_collect();

            assert!(ephemeron.value().is_none());
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

            assert_eq!(*wrap.upgrade().expect("weak is live"), "foo");

            let eph = Ephemeron::new(&wrap, 3);

            drop(cloned_gc);
            force_collect();
            assert!(wrap.upgrade().is_some());
            assert_eq!(eph.value().expect("weak is still live"), 3);

            drop(gc_value);
            force_collect();
            assert!(wrap.upgrade().is_none());
            assert!(eph.value().is_some());

            drop(wrap);
            force_collect();
            assert!(eph.value().is_none());
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

        assert_eq!(eph.value().expect("must be live"), 4);
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

        assert_eq!(*new_gc, *new_weak.upgrade().expect("weak should be live"));
        assert_eq!(
            *init_gc,
            *new_weak.upgrade().expect("weak_should be live still")
        );
    });
}

#[test]
fn eph_self_referential() {
    #[derive(Trace, Finalize, Clone)]
    struct InnerCell {
        inner: GcRefCell<Option<Ephemeron<InnerCell, TestCell>>>,
    }
    #[derive(Trace, Finalize, Clone)]
    struct TestCell {
        inner: Gc<InnerCell>,
    }
    run_test(|| {
        let root = TestCell {
            inner: Gc::new(InnerCell {
                inner: GcRefCell::new(None),
            }),
        };
        let root_size = std::mem::size_of::<GcBox<InnerCell>>();

        Harness::assert_exact_bytes_allocated(root_size);

        {
            // Generate a self-referential ephemeron
            let eph = Ephemeron::new(&root.inner, root.clone());
            *root.inner.inner.borrow_mut() = Some(eph.clone());

            assert!(eph.value().is_some());
            Harness::assert_exact_bytes_allocated(80);
        }

        *root.inner.inner.borrow_mut() = None;

        force_collect();

        Harness::assert_exact_bytes_allocated(root_size);
    });
}

#[test]
fn eph_self_referential_chain() {
    #[derive(Trace, Finalize, Clone)]
    struct TestCell {
        inner: Gc<GcRefCell<Option<Ephemeron<u8, TestCell>>>>,
    }
    run_test(|| {
        let root = Gc::new(GcRefCell::new(None));
        let root_size = std::mem::size_of::<GcBox<GcRefCell<Option<Ephemeron<u8, TestCell>>>>>();

        Harness::assert_exact_bytes_allocated(root_size);

        let watched = Gc::new(0);

        {
            // Generate a self-referential loop of weak and non-weak pointers
            let chain1 = TestCell {
                inner: Gc::new(GcRefCell::new(None)),
            };
            let chain2 = TestCell {
                inner: Gc::new(GcRefCell::new(None)),
            };

            let eph_start = Ephemeron::new(&watched, chain1.clone());
            let eph_chain2 = Ephemeron::new(&watched, chain2.clone());

            *chain1.inner.borrow_mut() = Some(eph_chain2.clone());
            *chain2.inner.borrow_mut() = Some(eph_start.clone());

            *root.borrow_mut() = Some(eph_start.clone());

            force_collect();

            assert!(eph_start.value().is_some());
            assert!(eph_chain2.value().is_some());
            Harness::assert_exact_bytes_allocated(240);
        }

        *root.borrow_mut() = None;

        force_collect();

        drop(watched);

        force_collect();

        Harness::assert_exact_bytes_allocated(root_size);
    });
}
