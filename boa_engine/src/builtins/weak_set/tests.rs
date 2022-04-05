use crate::Context;
use boa_parser::Source;

#[test]
fn weak_ref_collected() {
    let context = &mut Context::default();

    let weak_set = context
        .eval(Source::from_bytes(
            r#"
        let set;
        {
            let obj = {a: 5, b: 6};
            set = new WeakSet([obj]);
        }
        set
    "#,
        ))
        .unwrap();

    boa_gc::force_collect();

    assert!(
        weak_set
            .as_object()
            .unwrap()
            .borrow()
            .as_weak_set()
            .unwrap()
            .all_collected(),
        "Objects in WeakSet should be collected after the last reference to the objects is dropped."
    );
}
