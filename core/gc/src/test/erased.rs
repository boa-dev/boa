use std::any::TypeId;

use boa_macros::{Finalize, Trace};

use super::run_test;
use crate::{Gc, GcBox, GcErased, force_collect, test::Harness};

#[test]
fn erased_gc() {
    run_test(|| {
        let value = vec![1, 2, 3];
        let gc = Gc::new(value.clone());

        assert_eq!(Gc::type_id(&gc), TypeId::of::<Vec<i32>>());

        let erased = GcErased::new(gc.clone());

        assert_eq!(erased.type_id(), TypeId::of::<Vec<i32>>());
        assert!(erased.is::<Vec<i32>>());

        assert_eq!(erased.downcast::<i32>(), None);

        let gc_from_erased = erased.downcast::<Vec<i32>>().unwrap();
        assert_eq!(**gc_from_erased, value);

        assert!(Gc::ptr_eq(&gc, gc_from_erased));
    });
}

#[test]
fn nested_erased_gc() {
    #[derive(Debug, Trace, Finalize)]
    struct List {
        value: i32,
        next: Option<GcErased>,
    }

    run_test(|| {
        let mut root = GcErased::new(Gc::new(List {
            value: 0,
            next: None,
        }));

        for value in 1..100 {
            root = GcErased::new(Gc::new(List {
                value,
                next: Some(root),
            }));
        }

        Harness::assert_exact_bytes_allocated(100 * size_of::<GcBox<List>>());
        force_collect();
        Harness::assert_exact_bytes_allocated(100 * size_of::<GcBox<List>>());

        let mut head = root.downcast::<List>().cloned();
        for value in (0..100).rev() {
            let head_unwrap = head.as_ref().unwrap();

            assert_eq!(head_unwrap.value, value);

            head = head_unwrap
                .next
                .as_ref()
                .and_then(GcErased::downcast::<List>)
                .cloned();
        }
    });
}

#[test]
fn c_style_inheritance() {
    #[repr(C)]
    #[derive(Debug, Trace, Finalize, PartialEq, Eq)]
    struct Base {
        base_field: Vec<i32>,
    }

    #[repr(C)]
    #[derive(Debug, Trace, Finalize, PartialEq, Eq)]
    struct Derived {
        base: Base,
        derived_field: Vec<i64>,
    }

    run_test(|| {
        let value = vec![1, 2, 3];
        let derived = Gc::new(Derived {
            base: Base {
                base_field: value.clone(),
            },
            derived_field: vec![4, 5, 6],
        });

        assert_eq!(Gc::type_id(&derived), TypeId::of::<Derived>());
        assert!(Gc::is::<Derived>(&derived));

        // SAFETY: The structs have #[repr(C)] so this is safe.
        let base = unsafe { Gc::cast_unchecked::<Base>(&derived) };

        assert_eq!(Gc::type_id(base), TypeId::of::<Derived>());
        assert!(Gc::is::<Derived>(base));

        assert_eq!(base.base_field, value);
        assert_eq!(base.base_field, derived.base.base_field);

        assert!(Gc::ptr_eq(base, &derived));

        assert_eq!(Gc::downcast::<i32>(base), None);
        assert_eq!(*Gc::downcast::<Derived>(base).unwrap(), derived);
    });
}
