use crate::{TestAction, run_test_actions};

#[test]
fn pmd_basic_construction() {
    run_test_actions([
        TestAction::run(
            "let pmd = new Temporal.PlainMonthDay(12, 25)",
        ),
        TestAction::assert("pmd.monthCode === 'M12'"),
        TestAction::assert("pmd.day === 25"),
    ]);
}

#[test]
fn pmd_to_locale_string_basic() {
    run_test_actions([
        TestAction::run(
            "let pmd = new Temporal.PlainMonthDay(12, 25)",
        ),
        // toLocaleString should return a string
        TestAction::assert("typeof pmd.toLocaleString() === 'string'"),
        // Should contain month and day components
        TestAction::assert("pmd.toLocaleString().includes('12')"),
        TestAction::assert("pmd.toLocaleString().includes('25')"),
    ]);
}

#[test]
fn pmd_to_locale_string_with_calendar() {
    run_test_actions([
        TestAction::run(
            "let pmd = new Temporal.PlainMonthDay(12, 25, 'iso8601')",
        ),
        // Should accept calendar parameter
        TestAction::assert("typeof pmd.toLocaleString() === 'string'"),
        // Should contain month and day information
        TestAction::assert("pmd.toLocaleString().includes('12')"),
        TestAction::assert("pmd.toLocaleString().includes('25')"),
    ]);
}

#[test]
fn pmd_to_locale_string_with_locales() {
    run_test_actions([
        TestAction::run(
            "let pmd = new Temporal.PlainMonthDay(12, 25)",
        ),
        // Should accept locales parameter (even if not fully implemented yet)
        TestAction::assert("typeof pmd.toLocaleString('en-US') === 'string'"),
        TestAction::assert("typeof pmd.toLocaleString(['en-US', 'fr-FR']) === 'string'"),
    ]);
}

#[test]
fn pmd_to_locale_string_with_options() {
    run_test_actions([
        TestAction::run(
            "let pmd = new Temporal.PlainMonthDay(12, 25)",
        ),
        // Should accept options parameter (even if not fully implemented yet)
        TestAction::assert("typeof pmd.toLocaleString(undefined, {}) === 'string'"),
    ]);
}

#[test]
fn pmd_to_locale_string_different_dates() {
    run_test_actions([
        TestAction::run(
            "let pmd1 = new Temporal.PlainMonthDay(1, 1)",
        ),
        TestAction::run(
            "let pmd2 = new Temporal.PlainMonthDay(7, 4)",
        ),
        TestAction::run(
            "let pmd3 = new Temporal.PlainMonthDay(10, 31)",
        ),
        // All should return strings
        TestAction::assert("typeof pmd1.toLocaleString() === 'string'"),
        TestAction::assert("typeof pmd2.toLocaleString() === 'string'"),
        TestAction::assert("typeof pmd3.toLocaleString() === 'string'"),
        // Should contain correct values
        TestAction::assert("pmd1.toLocaleString().includes('01')"),
        TestAction::assert("pmd2.toLocaleString().includes('07')"),
        TestAction::assert("pmd3.toLocaleString().includes('10')"),
    ]);
}
