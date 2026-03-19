use indoc::indoc;

use crate::{JsNativeErrorKind, TestAction, run_test_actions};

#[test]
fn with_overflow_reject_throws_on_negative_components() {
    // Temporal Builtin tests.
    run_test_actions([
        TestAction::run(indoc! {"
            let plain = new Temporal.PlainTime();
            let options = {overflow: 'reject'};
        "}),
        TestAction::assert_native_error(
            "plain.with({hour: -1}, options)",
            JsNativeErrorKind::Range,
            "time value 'hour' not in 0..23: -1",
        ),
        TestAction::assert_native_error(
            "plain.with({minute: -1}, options)",
            JsNativeErrorKind::Range,
            "time value 'minute' not in 0..59: -1",
        ),
        TestAction::assert_native_error(
            "plain.with({second: -1}, options)",
            JsNativeErrorKind::Range,
            "time value 'second' not in 0..59: -1",
        ),
        TestAction::assert_native_error(
            "plain.with({millisecond: -1}, options)",
            JsNativeErrorKind::Range,
            "time value 'millisecond' not in 0..999: -1",
        ),
        TestAction::assert_native_error(
            "plain.with({microsecond: -1}, options)",
            JsNativeErrorKind::Range,
            "time value 'microsecond' not in 0..999: -1",
        ),
        TestAction::assert_native_error(
            "plain.with({nanosecond: -1}, options)",
            JsNativeErrorKind::Range,
            "time value 'nanosecond' not in 0..999: -1",
        ),
    ]);
}

#[test]
fn pt_to_locale_string_basic() {
    run_test_actions([
        TestAction::run(
            "let pt = new Temporal.PlainTime(15, 23, 30, 123, 456, 789)",
        ),
        // toLocaleString should return a string
        TestAction::assert("typeof pt.toLocaleString() === 'string'"),
        // Should contain time components
        TestAction::assert("pt.toLocaleString().includes('15')"),
        TestAction::assert("pt.toLocaleString().includes('23')"),
        TestAction::assert("pt.toLocaleString().includes('30')"),
    ]);
}

#[test]
fn pt_to_locale_string_with_midnight() {
    run_test_actions([
        TestAction::run(
            "let pt = new Temporal.PlainTime(0, 0, 0)",
        ),
        // Midnight should work
        TestAction::assert("typeof pt.toLocaleString() === 'string'"),
        TestAction::assert("pt.toLocaleString().includes('00')"),
    ]);
}

#[test]
fn pt_to_locale_string_with_noon() {
    run_test_actions([
        TestAction::run(
            "let pt = new Temporal.PlainTime(12, 0, 0)",
        ),
        // Noon should work
        TestAction::assert("typeof pt.toLocaleString() === 'string'"),
        TestAction::assert("pt.toLocaleString().includes('12')"),
    ]);
}

#[test]
fn pt_to_locale_string_with_locales() {
    run_test_actions([
        TestAction::run(
            "let pt = new Temporal.PlainTime(15, 23, 30)",
        ),
        // Should accept locales parameter (even if not fully implemented yet)
        TestAction::assert("typeof pt.toLocaleString('en-US') === 'string'"),
        TestAction::assert("typeof pt.toLocaleString(['en-US', 'fr-FR']) === 'string'"),
    ]);
}

#[test]
fn pt_to_locale_string_with_options() {
    run_test_actions([
        TestAction::run(
            "let pt = new Temporal.PlainTime(15, 23, 30)",
        ),
        // Should accept options parameter (even if not fully implemented yet)
        TestAction::assert("typeof pt.toLocaleString(undefined, {}) === 'string'"),
    ]);
}

#[test]
fn pt_to_locale_string_different_times() {
    run_test_actions([
        TestAction::run(
            "let pt1 = new Temporal.PlainTime(9, 30)",
        ),
        TestAction::run(
            "let pt2 = new Temporal.PlainTime(14, 45, 30)",
        ),
        TestAction::run(
            "let pt3 = new Temporal.PlainTime(23, 59, 59, 999)",
        ),
        // All should return strings
        TestAction::assert("typeof pt1.toLocaleString() === 'string'"),
        TestAction::assert("typeof pt2.toLocaleString() === 'string'"),
        TestAction::assert("typeof pt3.toLocaleString() === 'string'"),
        // Should contain correct hour values
        TestAction::assert("pt1.toLocaleString().includes('09')"),
        TestAction::assert("pt2.toLocaleString().includes('14')"),
        TestAction::assert("pt3.toLocaleString().includes('23')"),
    ]);
}
