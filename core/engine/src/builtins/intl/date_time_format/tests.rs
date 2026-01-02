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
