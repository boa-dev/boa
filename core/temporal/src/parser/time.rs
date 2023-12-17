//! Parsing of ISO8601 Time Values

use super::{
    grammar::{is_decimal_separator, is_time_separator},
    Cursor,
};
use crate::{assert_syntax, TemporalError, TemporalResult};

/// Parsed Time info
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TimeSpec {
    /// An hour
    pub(crate) hour: u8,
    /// A minute value
    pub(crate) minute: u8,
    /// A second value.
    pub(crate) second: u8,
    /// A floating point number representing the sub-second values
    pub(crate) fraction: f64,
}

/// Parse `TimeSpec`
pub(crate) fn parse_time_spec(cursor: &mut Cursor) -> TemporalResult<TimeSpec> {
    let hour = parse_hour(cursor)?;

    if !cursor.check_or(false, |ch| is_time_separator(ch) || ch.is_ascii_digit()) {
        return Ok(TimeSpec {
            hour,
            minute: 0,
            second: 0,
            fraction: 0.0,
        });
    }

    let separator_present = cursor.check_or(false, is_time_separator);
    cursor.advance_if(separator_present);

    let minute = parse_minute_second(cursor, false)?;

    if !cursor.check_or(false, |ch| is_time_separator(ch) || ch.is_ascii_digit()) {
        return Ok(TimeSpec {
            hour,
            minute,
            second: 0,
            fraction: 0.0,
        });
    } else if cursor.check_or(false, is_time_separator) && !separator_present {
        return Err(TemporalError::syntax().with_message("Invalid TimeSeparator"));
    }

    cursor.advance_if(separator_present);

    let second = parse_minute_second(cursor, true)?;

    let fraction = if cursor.check_or(false, is_decimal_separator) {
        parse_fraction(cursor)?
    } else {
        0.0
    };

    Ok(TimeSpec {
        hour,
        minute,
        second,
        fraction,
    })
}

pub(crate) fn parse_hour(cursor: &mut Cursor) -> TemporalResult<u8> {
    let start = cursor.pos();
    for _ in 0..2 {
        let digit = cursor.abrupt_next()?;
        assert_syntax!(digit.is_ascii_digit(), "Hour must be a digit.");
    }
    let hour_value = cursor
        .slice(start, cursor.pos())
        .parse::<u8>()
        .map_err(|e| TemporalError::syntax().with_message(e.to_string()))?;
    if !(0..=23).contains(&hour_value) {
        return Err(TemporalError::syntax().with_message("Hour must be in a range of 0-23"));
    }
    Ok(hour_value)
}

// NOTE: `TimeSecond` is a 60 inclusive `MinuteSecond`.
/// Parse `MinuteSecond`
pub(crate) fn parse_minute_second(cursor: &mut Cursor, inclusive: bool) -> TemporalResult<u8> {
    let start = cursor.pos();
    for _ in 0..2 {
        let digit = cursor.abrupt_next()?;
        assert_syntax!(digit.is_ascii_digit(), "MinuteSecond must be a digit.");
    }
    let min_sec_value = cursor
        .slice(start, cursor.pos())
        .parse::<u8>()
        .map_err(|e| TemporalError::syntax().with_message(e.to_string()))?;

    let valid_range = if inclusive { 0..=60 } else { 0..=59 };
    if !valid_range.contains(&min_sec_value) {
        return Err(TemporalError::syntax().with_message("MinuteSecond must be in a range of 0-59"));
    }
    Ok(min_sec_value)
}

/// Parse a `Fraction` value
///
/// This is primarily used in ISO8601 to add percision past
/// a second.
pub(crate) fn parse_fraction(cursor: &mut Cursor) -> TemporalResult<f64> {
    let mut fraction_components = Vec::default();

    // Assert that the first char provided is a decimal separator.
    assert_syntax!(
        is_decimal_separator(cursor.abrupt_next()?),
        "fraction must begin with a valid decimal separator."
    );
    fraction_components.push('.');

    while cursor.check_or(false, |ch| ch.is_ascii_digit()) {
        fraction_components.push(cursor.abrupt_next()?);
    }

    assert_syntax!(
        fraction_components.len() <= 10,
        "Fraction component cannot exceed 9 digits."
    );

    let fraction_value = fraction_components
        .iter()
        .collect::<String>()
        .parse::<f64>()
        .map_err(|e| TemporalError::syntax().with_message(e.to_string()))?;
    Ok(fraction_value)
}
