use crate::Performance;
use crate::test::{TestAction, run_test_actions};
use indoc::indoc;

const TEST_HARNESS: &str = r#"
function assert_true(condition, message) {
    if (!condition) {
        throw new Error(`Assertion failed: ${message || ''}`);
    }
}
"#;

#[test]
fn performance_now_returns_number() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            Performance::register(ctx).unwrap();
        }),
        TestAction::run(TEST_HARNESS),
        TestAction::run(indoc! {r#"
            assert_true(typeof performance.now() === 'number');
        "#}),
    ]);
}

#[test]
fn performance_now_increases() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            Performance::register(ctx).unwrap();
        }),
        TestAction::run(TEST_HARNESS),
        TestAction::run(indoc! {r#"
            const t1 = performance.now();
            const t2 = performance.now();
            assert_true(t2 >= t1, 'time should increase');
        "#}),
    ]);
}

#[test]
fn performance_now_is_non_negative() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            Performance::register(ctx).unwrap();
        }),
        TestAction::run(TEST_HARNESS),
        TestAction::run(indoc! {r#"
            assert_true(performance.now() >= 0, 'time should be non-negative');
        "#}),
    ]);
}
