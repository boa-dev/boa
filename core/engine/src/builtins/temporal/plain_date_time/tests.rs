use crate::{TestAction, run_test_actions};

#[test]
fn pdt_year_of_week_basic() {
    run_test_actions([
        TestAction::run(
            "let pdt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, 'iso8601')",
        ),
        TestAction::assert_eq("pdt.yearOfWeek", 1976),
    ]);
}

#[test]
fn pdt_to_locale_string_basic() {
    run_test_actions([
        TestAction::run(
            "let pdt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, 'iso8601')",
        ),
        // toLocaleString should return a string (currently falls back to ISO format)
        TestAction::assert("typeof pdt.toLocaleString() === 'string'"),
        // Should contain date and time components
        TestAction::assert("pdt.toLocaleString().includes('1976')"),
        TestAction::assert("pdt.toLocaleString().includes('11')"),
        TestAction::assert("pdt.toLocaleString().includes('18')"),
    ]);
}

#[test]
fn pdt_to_locale_string_with_locales() {
    run_test_actions([
        TestAction::run(
            "let pdt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, 'iso8601')",
        ),
        // Should accept locales parameter (even if not fully implemented yet)
        TestAction::assert("typeof pdt.toLocaleString('en-US') === 'string'"),
        TestAction::assert("typeof pdt.toLocaleString(['en-US', 'fr-FR']) === 'string'"),
    ]);
}

#[test]
fn pdt_to_locale_string_with_options() {
    run_test_actions([
        TestAction::run(
            "let pdt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, 'iso8601')",
        ),
        // Should accept options parameter (even if not fully implemented yet)
        TestAction::assert("typeof pdt.toLocaleString(undefined, {}) === 'string'"),
    ]);
}
