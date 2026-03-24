use crate::{JsNativeErrorKind, TestAction, run_test_actions};

#[test]
fn duration_constructor() {
    run_test_actions([
        TestAction::run("let dur = new Temporal.Duration(1, 1, 0, 1)"),
        TestAction::assert_eq("dur.years", 1),
        TestAction::assert_eq("dur.months", 1),
        TestAction::assert_eq("dur.weeks", 0),
        TestAction::assert_eq("dur.days", 1),
        TestAction::assert_eq("dur.milliseconds", 0),
    ]);
}

#[test]
fn duration_abs() {
    run_test_actions([
        TestAction::run("let dur = new Temporal.Duration(-1, -1, 0, -1)"),
        TestAction::assert_eq("dur.sign", -1),
        TestAction::run("let abs = dur.abs()"),
        TestAction::assert_eq("abs.years", 1),
        TestAction::assert_eq("abs.months", 1),
        TestAction::assert_eq("abs.weeks", 0),
        TestAction::assert_eq("abs.days", 1),
        TestAction::assert_eq("abs.milliseconds", 0),
    ]);
}

#[test]
fn basic() {
    run_test_actions([
        TestAction::run(
            r#"
            var dur = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 0);
        "#,
        ),
        TestAction::assert_eq("dur.years", 5),
        TestAction::assert_eq("dur.months", 5),
        TestAction::assert_eq("dur.weeks", 5),
        TestAction::assert_eq("dur.days", 5),
        TestAction::assert_eq("dur.hours", 5),
        TestAction::assert_eq("dur.minutes", 5),
        TestAction::assert_eq("dur.seconds", 5),
        TestAction::assert_eq("dur.milliseconds", 5),
        TestAction::assert_eq("dur.microseconds", 5),
        TestAction::assert_eq("dur.nanoseconds", 0),
        // Negative
        TestAction::run("dur = new Temporal.Duration(-5, -5, -5, -5, -5, -5, -5, -5, -5, 0)"),
        TestAction::assert_eq("dur.years", -5),
        TestAction::assert_eq("dur.months", -5),
        TestAction::assert_eq("dur.weeks", -5),
        TestAction::assert_eq("dur.days", -5),
        TestAction::assert_eq("dur.hours", -5),
        TestAction::assert_eq("dur.minutes", -5),
        TestAction::assert_eq("dur.seconds", -5),
        TestAction::assert_eq("dur.milliseconds", -5),
        TestAction::assert_eq("dur.microseconds", -5),
        TestAction::assert_eq("dur.nanoseconds", 0),
        // Negative Zero
        TestAction::run("dur = new Temporal.Duration(-0, -0, -0, -0, -0, -0, -0, -0, -0, 0)"),
        TestAction::assert_eq("dur.years", 0),
        TestAction::assert_eq("dur.months", 0),
        TestAction::assert_eq("dur.weeks", 0),
        TestAction::assert_eq("dur.days", 0),
        TestAction::assert_eq("dur.hours", 0),
        TestAction::assert_eq("dur.minutes", 0),
        TestAction::assert_eq("dur.seconds", 0),
        TestAction::assert_eq("dur.milliseconds", 0),
        TestAction::assert_eq("dur.microseconds", 0),
        TestAction::assert_eq("dur.nanoseconds", 0),
    ]);
}

#[test]
fn duration_to_locale_string_matches_to_json_until_intl_duration_format() {
    run_test_actions([
        TestAction::run("let dur = Temporal.Duration.from('P1Y2M3DT4H5M6.007008009S')"),
        TestAction::assert("dur.toLocaleString() === dur.toJSON()"),
        TestAction::assert(
            "dur.toLocaleString('en-US', { style: 'narrow' }) === dur.toJSON()",
        ),
    ]);
}

#[test]
fn duration_value_of_throws_type_error_with_compare_hint() {
    run_test_actions([TestAction::assert_native_error(
        "Temporal.Duration.from('P1D').valueOf()",
        JsNativeErrorKind::Type,
        "Cannot convert a Temporal.Duration to a primitive value. Use Temporal.Duration.compare() for comparison or Temporal.Duration.prototype.toString() for a string representation.",
    )]);
}
