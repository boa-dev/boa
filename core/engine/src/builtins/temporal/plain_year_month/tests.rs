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

#[cfg(feature = "intl")]
#[test]
fn to_locale_string_ignores_time_zone_for_plain_values() {
    run_test_actions([TestAction::assert(
        "Temporal.PlainYearMonth.from('2024-03').toLocaleString('en-US', { timeZone: 'America/New_York' }) === \
         Temporal.PlainYearMonth.from('2024-03').toLocaleString('en-US', { timeZone: '+00:00' })",
    )]);
}

#[cfg(feature = "intl")]
#[test]
fn to_locale_string_default_excludes_reference_day_time_and_zone_name() {
    // Mirrors test262 `default-does-not-include-day-time-and-time-zone-name.js`.
    run_test_actions([TestAction::assert(
        "(() => { \
            const p = new Temporal.PlainYearMonth(2024, 12, 'iso8601', 26); \
            const r = p.toLocaleString('en-u-ca-iso8601', { timeZone: 'UTC' }); \
            return r.includes('2024') \
                && (r.includes('12') || r.includes('Dec')) \
                && !r.includes('26') \
                && !r.includes('00') \
                && !r.includes('UTC') \
                && !r.includes('Coordinated Universal Time'); \
        })()",
    )]);
}

#[cfg(feature = "intl")]
#[test]
fn to_locale_string_incompatible_calendar_throws() {
    run_test_actions([TestAction::assert_native_error(
        "Temporal.PlainYearMonth.from('2024-03').toLocaleString('en-US', { calendar: 'japanese' })",
        JsNativeErrorKind::Range,
        "Temporal.PlainYearMonth calendar must match Intl.DateTimeFormat calendar.",
    )]);
}
