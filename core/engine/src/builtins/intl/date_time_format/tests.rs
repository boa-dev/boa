use indoc::indoc;

use crate::{TestAction, run_test_actions};

// Intl.DateTimeFormat tests

#[cfg(feature = "intl_bundled")]
#[test]
fn dtf_basic() {
    run_test_actions([
        TestAction::run(indoc! {"
            // Setup date
            const date = new Date(Date.UTC(2020, 11, 20, 3, 23, 16, 738));

            let formatter = new Intl.DateTimeFormat('en-US');
            let result = formatter.format(date);
        "}),
        TestAction::assert_eq("result === '12/20/20'", true),
    ]);
    run_test_actions([
        TestAction::run(indoc! {"
            // Setup date
            const date = new Date(Date.UTC(2020, 11, 20, 3, 23, 16, 738));

            let formatter = new Intl.DateTimeFormat('en-US', { dateStyle: 'full' });
            let result = formatter.format(date);
        "}),
        TestAction::assert_eq("result === 'Sunday, December 20, 2020'", true),
    ]);
    run_test_actions([
        TestAction::run(indoc! {"
            // Setup date
            const date = new Date(Date.UTC(2020, 11, 20, 3, 23, 16, 738));

            let formatter = new Intl.DateTimeFormat('en-GB');
            let result = formatter.format(date);
        "}),
        TestAction::assert_eq("result === '20/12/2020'", true),
    ]);
    run_test_actions([
        TestAction::run(indoc! {"
            // Setup date
            const date = new Date(Date.UTC(2020, 11, 20, 3, 23, 16, 738));

            let formatter = new Intl.DateTimeFormat('en-GB', {
                dateStyle: 'full',
                timeStyle: 'long',
            });
            let result = formatter.format(date);
        "}),
        TestAction::assert_eq("result === 'Sunday, 20 December 2020 at 03:23:16'", true),
    ]);
    run_test_actions([
        TestAction::run(indoc! {"
            // Setup date
            const date = new Date(Date.UTC(2020, 11, 20, 3, 23, 16, 738));

            let formatter = new Intl.DateTimeFormat('en-GB', {
                dateStyle: 'full',
                timeStyle: 'long',
                timeZone: 'Australia/Sydney',
            });
            let result = formatter.format(date);
        "}),
        TestAction::assert_eq("result === 'Sunday, 20 December 2020 at 14:23:16'", true),
    ]);
}

#[cfg(feature = "intl_bundled")]
#[test]
fn date_to_locale_string_style_does_not_force_missing_counterpart() {
    run_test_actions([
        TestAction::run(indoc! {"
            const date = new Date(Date.UTC(2024, 2, 10, 2, 30, 0, 0));

            const dateOnly = date.toLocaleString('en-US', { dateStyle: 'short', timeZone: 'UTC' });
            const timeOnly = date.toLocaleString('en-US', { timeStyle: 'short', timeZone: 'UTC' });
        "}),
        TestAction::assert_eq("dateOnly === '3/10/24'", true),
        TestAction::assert_eq("/^2:30\\s?AM$/.test(timeOnly)", true),
        TestAction::assert_eq(
            "dateOnly.includes(':') || dateOnly.includes('AM') || dateOnly.includes('PM')",
            false,
        ),
        TestAction::assert_eq(
            "timeOnly.includes('/') || timeOnly.includes('-') || /January|February|March|April|May|June|July|August|September|October|November|December/.test(timeOnly)",
            false,
        ),
    ]);
}
