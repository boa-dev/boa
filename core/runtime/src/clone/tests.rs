//! Tests for the `structuredClone` extension.

use crate::test::{TestAction, run_test_actions};

#[test]
fn clones_error_objects() {
    run_test_actions([
        TestAction::harness(),
        TestAction::run(
            r#"
                const original = new Error("boom");
                const cloned = structuredClone(original);

                assert(cloned instanceof Error);
                assert(cloned !== original);
                assertEq(cloned.name, "Error");
                assertEq(cloned.message, "boom");
            "#,
        ),
    ]);
}

#[test]
fn clones_error_object_cause() {
    run_test_actions([
        TestAction::harness(),
        TestAction::run(
            r#"
                const original = new Error("boom", { cause: { code: 7 } });
                const cloned = structuredClone(original);

                assert(cloned instanceof Error);
                assert(cloned.cause !== original.cause);
                assertEq(cloned.cause.code, 7);
            "#,
        ),
    ]);
}

#[test]
fn clones_aggregate_error_entries() {
    run_test_actions([
        TestAction::harness(),
        TestAction::run(
            r#"
                const original = new AggregateError([new Error("inner")], "agg");
                const cloned = structuredClone(original);

                assert(cloned instanceof AggregateError);
                assertEq(cloned.message, "agg");
                assertEq(cloned.errors.length, 1);
                assert(cloned.errors[0] instanceof Error);
                assertEq(cloned.errors[0].message, "inner");
            "#,
        ),
    ]);
}

#[test]
fn clones_error_with_undefined_cause_property() {
    run_test_actions([
        TestAction::harness(),
        TestAction::run(
            r#"
                const original = new Error("boom", { cause: undefined });
                const cloned = structuredClone(original);

                assert(Object.hasOwn(original, "cause"));
                assert(Object.hasOwn(cloned, "cause"));
                assertEq(cloned.cause, undefined);
            "#,
        ),
    ]);
}
