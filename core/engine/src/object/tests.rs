use crate::{JsNativeErrorKind, TestAction, run_test_actions};
use indoc::indoc;

#[test]
fn ordinary_has_instance_nonobject_prototype() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            function C() {}
            C.prototype = 1
            String instanceof C
        "#},
        JsNativeErrorKind::Type,
        "function has non-object prototype in instanceof check",
    )]);
}

#[test]
fn object_properties_return_order() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
                var o = {
                    p1: 'v1',
                    p2: 'v2',
                    p3: 'v3',
                };
                o.p4 = 'v4';
                o[2] = 'iv2';
                o[0] = 'iv0';
                o[1] = 'iv1';
                delete o.p1;
                delete o.p3;
                o.p1 = 'v1';
            "#}),
        TestAction::assert(r#"arrayEquals(Object.keys(o), [ "0", "1", "2", "p2", "p4", "p1" ])"#),
        TestAction::assert(
            r#"arrayEquals(Object.values(o), [ "iv0", "iv1", "iv2", "v2", "v4", "v1" ])"#,
        ),
    ]);
}

#[test]
fn array_prototype_for_each_edge_cases() {
    run_test_actions([
        TestAction::run_harness(),
        // Empty array — callback never called
        TestAction::assert(indoc! {r#"
            var called = 0;
            [].forEach(() => { called++; });
            called === 0
        "#}),
        // Return value is always undefined
        TestAction::assert(indoc! {r#"
            var result = [1, 2, 3].forEach(x => x * 2);
            result === undefined
        "#}),
        // Sparse array — holes are skipped
        TestAction::assert(indoc! {r#"
            var count = 0;
            let arr = [1, , 3];
            arr.forEach(() => { count++; });
            count === 2
        "#}),
        // this binding via second argument
        TestAction::assert(indoc! {r#"
            var obj = { multiplier: 2, result: 0 };
            [1, 2, 3].forEach(function(x) {
                this.result += x * this.multiplier;
            }, obj);
            obj.result === 12
        "#}),
    ]);
}
