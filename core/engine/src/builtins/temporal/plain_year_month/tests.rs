use indoc::indoc;

use crate::{JsNativeErrorKind, TestAction, js_string, run_test_actions};

#[test]
fn add_reads_duration_before_validating_options() {
    run_test_actions([
        TestAction::run(indoc! {"
            let log = [];
            const duration = { get months() { log.push('months'); return 1; } };
            const ym = new Temporal.PlainYearMonth(2019, 6);
        "}),
        TestAction::assert_native_error(
            "ym.add(duration, null)",
            JsNativeErrorKind::Type,
            "GetOptionsObject: provided options is not an object",
        ),
        TestAction::assert_eq("log.join()", js_string!("months")),
    ]);
}

#[test]
fn subtract_reads_duration_before_validating_options() {
    run_test_actions([
        TestAction::run(indoc! {"
            let log = [];
            const duration = { get months() { log.push('months'); return 1; } };
            const ym = new Temporal.PlainYearMonth(2019, 6);
        "}),
        TestAction::assert_native_error(
            "ym.subtract(duration, null)",
            JsNativeErrorKind::Type,
            "GetOptionsObject: provided options is not an object",
        ),
        TestAction::assert_eq("log.join()", js_string!("months")),
    ]);
}

#[test]
fn add_checks_this_before_reading_duration() {
    run_test_actions([
        TestAction::run(indoc! {"
            let log = [];
            const duration = { get months() { log.push('months'); return 1; } };
        "}),
        TestAction::assert_native_error(
            "Temporal.PlainYearMonth.prototype.add.call({}, duration)",
            JsNativeErrorKind::Type,
            "this value must be a PlainYearMonth object.",
        ),
        TestAction::assert_eq("log.length", 0),
    ]);
}

#[test]
fn add_and_subtract_accept_duration_strings() {
    run_test_actions([
        TestAction::run("const ym = new Temporal.PlainYearMonth(2019, 6);"),
        TestAction::assert_eq("ym.add('P1M').toString()", js_string!("2019-07")),
        TestAction::assert_eq("ym.subtract('P1Y2M').toString()", js_string!("2018-04")),
        TestAction::assert_native_error(
            "ym.add(1)",
            JsNativeErrorKind::Type,
            "Invalid TemporalDurationLike value.",
        ),
    ]);
}
