use crate::{JsNativeErrorKind, TestAction, run_test_actions};

#[test]
fn to_locale_string_returns_string() {
    run_test_actions([
        TestAction::assert(
            "typeof Temporal.PlainYearMonth.from('2024-03').toLocaleString() === 'string'",
        ),
        TestAction::assert("Temporal.PlainYearMonth.from('2024-03').toLocaleString().length > 0"),
    ]);
}

#[test]
fn to_locale_string_invalid_receiver_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Temporal.PlainYearMonth.prototype.toLocaleString.call({})",
        JsNativeErrorKind::Type,
        "this value must be a PlainYearMonth object.",
    )]);
}

#[cfg(feature = "intl")]
#[test]
fn to_locale_string_different_locales_produce_different_output() {
    run_test_actions([TestAction::assert(
        "Temporal.PlainYearMonth.from('2024-03').toLocaleString('en-US') !== \
         Temporal.PlainYearMonth.from('2024-03').toLocaleString('de-DE')",
    )]);
}

#[cfg(feature = "intl")]
#[test]
fn to_locale_string_options_affect_output() {
    run_test_actions([
        TestAction::assert(
            "typeof Temporal.PlainYearMonth.from('2024-03').toLocaleString('en-US', { dateStyle: 'short' }) === 'string'",
        ),
        TestAction::assert(
            "Temporal.PlainYearMonth.from('2024-03').toLocaleString('en-US', { dateStyle: 'short' }).length > 0",
        ),
    ]);
}
