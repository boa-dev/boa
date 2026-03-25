use indoc::indoc;

use crate::builtins::intl::number_format::RoundingIncrement;
use crate::{TestAction, js_string, run_test_actions};
use fixed_decimal::RoundingIncrement::*;

#[test]
fn u16_to_rounding_increment_sunny_day() {
    #[rustfmt::skip]
    let valid_cases: [(u16, RoundingIncrement); 15] = [
        // Singles
        (1, RoundingIncrement::from_parts(MultiplesOf1, 0).unwrap()),
        (2, RoundingIncrement::from_parts(MultiplesOf2, 0).unwrap()),
        (5, RoundingIncrement::from_parts(MultiplesOf5, 0).unwrap()),
        // Tens
        (10, RoundingIncrement::from_parts(MultiplesOf1, 1).unwrap()),
        (20, RoundingIncrement::from_parts(MultiplesOf2, 1).unwrap()),
        (25, RoundingIncrement::from_parts(MultiplesOf25, 0).unwrap()),
        (50, RoundingIncrement::from_parts(MultiplesOf5, 1).unwrap()),
        // Hundreds
        (100, RoundingIncrement::from_parts(MultiplesOf1, 2).unwrap()),
        (200, RoundingIncrement::from_parts(MultiplesOf2, 2).unwrap()),
        (250, RoundingIncrement::from_parts(MultiplesOf25, 1).unwrap()),
        (500, RoundingIncrement::from_parts(MultiplesOf5, 2).unwrap()),
        // Thousands
        (1000, RoundingIncrement::from_parts(MultiplesOf1, 3).unwrap()),
        (2000, RoundingIncrement::from_parts(MultiplesOf2, 3).unwrap()),
        (2500, RoundingIncrement::from_parts(MultiplesOf25, 2).unwrap()),
        (5000, RoundingIncrement::from_parts(MultiplesOf5, 3).unwrap()),
    ];

    for (num, increment) in valid_cases {
        assert_eq!(RoundingIncrement::from_u16(num), Some(increment));
    }
}

#[test]
fn u16_to_rounding_increment_rainy_day() {
    const INVALID_CASES: [u16; 9] = [0, 4, 6, 24, 10000, 65535, 7373, 140, 1500];

    for num in INVALID_CASES {
        assert!(RoundingIncrement::from_u16(num).is_none());
    }
}

#[cfg(feature = "intl_bundled")]
#[test]
fn percent_style_formats_correctly() {
    // Test case from issue #5246: percent style should multiply by 100
    // and append a percent sign.
    run_test_actions([
        TestAction::run(indoc! {"
            var nf = new Intl.NumberFormat('en-US', { style: 'percent' });
            var result = nf.format(0.56);
        "}),
        TestAction::assert_eq("result", js_string!("56\u{202F}%")),
    ]);
}

#[cfg(feature = "intl_bundled")]
#[test]
fn percent_style_with_significant_digits() {
    // Test case from issue #5246: BigInt toLocaleString with percent style
    // and maximumSignificantDigits.
    run_test_actions([
        TestAction::run(indoc! {"
            var options = { maximumSignificantDigits: 4, style: 'percent' };
            var result = (0.8877).toLocaleString('de-DE', options);
        "}),
        TestAction::assert_eq("result", js_string!("88,77\u{202F}%")),
    ]);
}
