//! Boa's implemetation of the `Temporal.TimeZone` builtin object.
#![allow(dead_code)]

use crate::{builtins::temporal::to_zero_padded_decimal_string, Context};

// -- TimeZone Abstract Operations --

/// Abstract operation `DefaultTimeZone ( )`
///
/// The abstract operation `DefaultTimeZone` takes no arguments. It returns a String value
/// representing the host environment's current time zone, which is either a valid (11.1.1) and
/// canonicalized (11.1.2) time zone name, or an offset conforming to the syntax of a
/// `TimeZoneNumericUTCOffset`.
///
/// An ECMAScript implementation that includes the ECMA-402 Internationalization API must implement
/// the `DefaultTimeZone` abstract operation as specified in the ECMA-402 specification.
///
/// More information:
///  - [ECMAScript specififcation][spec]
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-defaulttimezone
#[allow(unused)]
pub(super) fn default_time_zone(context: &mut Context) -> String {
    // The minimum implementation of DefaultTimeZone for ECMAScript implementations that do not
    // include the ECMA-402 API, supporting only the "UTC" time zone, performs the following steps
    // when called:

    // 1. Return "UTC".
    "UTC".to_owned()

    // TO-DO: full, system-aware implementation (and intl feature)
}

/// Abstract operation `FormatTimeZoneOffsetString ( offsetNanoseconds )`
fn format_time_zone_offset_string(offset_nanoseconds: i64) -> String {
    // 1. Assert: offsetNanoseconds is an integer.

    // 2. If offsetNanoseconds ≥ 0, let sign be "+"; otherwise, let sign be "-".
    let sign = if offset_nanoseconds >= 0 { "+" } else { "-" };

    // 3. Let offsetNanoseconds be abs(offsetNanoseconds).
    let offset_nanoseconds = offset_nanoseconds.unsigned_abs();

    // 4. Let nanoseconds be offsetNanoseconds modulo 10^9.
    let nanoseconds = offset_nanoseconds % 1_000_000_000;

    // 5. Let seconds be floor(offsetNanoseconds / 10^9) modulo 60.
    let seconds = (offset_nanoseconds / 1_000_000_000) % 60;

    // 6. Let minutes be floor(offsetNanoseconds / (6 × 10^10)) modulo 60.
    let minutes = (offset_nanoseconds / 60_000_000_000) % 60;

    // 7. Let hours be floor(offsetNanoseconds / (3.6 × 1012)).
    let hours = (offset_nanoseconds / 3_600_000_000_000) % 60;

    // 8. Let h be ToZeroPaddedDecimalString(hours, 2).
    let h = to_zero_padded_decimal_string(hours, 2);

    // 9. Let m be ToZeroPaddedDecimalString(minutes, 2).
    let m = to_zero_padded_decimal_string(minutes, 2);

    // 10. Let s be ToZeroPaddedDecimalString(seconds, 2).
    let s = to_zero_padded_decimal_string(seconds, 2);

    // 11. If nanoseconds ≠ 0, then
    let post = if nanoseconds != 0 {
        // a. Let fraction be ToZeroPaddedDecimalString(nanoseconds, 9).
        let fraction = to_zero_padded_decimal_string(nanoseconds, 9);

        // b. Set fraction to the longest possible substring of fraction starting at position 0 and not ending with the code unit 0x0030 (DIGIT ZERO).
        let fraction = fraction.trim_end_matches('0');

        // c. Let post be the string-concatenation of the code unit 0x003A (COLON), s, the code unit 0x002E (FULL STOP), and fraction.
        format!(":{s}.{fraction}")
    } else if seconds != 0 {
        // 12. Else if seconds ≠ 0, then
        // a. Let post be the string-concatenation of the code unit 0x003A (COLON) and s.
        format!(":{s}")
    } else {
        // 13. Else,
        // a. Let post be the empty String.
        String::new()
    };

    // 14. Return the string-concatenation of sign, h, the code unit 0x003A (COLON), m, and post.
    format!("{sign}{h}:{m}{post}")
}

/// Abstract operation `CanonicalizeTimeZoneName ( timeZone )`
///
/// The abstract operation `CanonicalizeTimeZoneName` takes argument `timeZone` (a String that is a
/// valid time zone name as verified by `IsAvailableTimeZoneName`). It returns the canonical and
/// case-regularized form of `timeZone`.
fn canonicalize_time_zone_name(time_zone: &str) -> String {
    // The minimum implementation of CanonicalizeTimeZoneName for ECMAScript implementations that
    // do not include local political rules for any time zones performs the following steps when
    // called:
    // 1. Assert: timeZone is an ASCII-case-insensitive match for "UTC".
    assert!(time_zone.eq_ignore_ascii_case("UTC"));
    // 2. Return "UTC".
    "UTC".to_owned()
}
