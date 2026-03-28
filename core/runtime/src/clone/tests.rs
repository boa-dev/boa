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
