//! Parsing of ISO8601 Time Values

use super::{
    grammar::{is_decimal_separator, is_time_separator},
    Cursor,
};
use crate::{TemporalError, TemporalResult};

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
    let mut separator = false;

    if cursor.check_or(false, |ch| is_time_separator(ch) || ch.is_ascii_digit()) {
        if cursor.check_or(false, is_time_separator) {
            separator = true;
            cursor.advance();
        }
    } else {
        return Ok(TimeSpec {
            hour,
            minute: 0,
            second: 0,
            fraction: 0.0,
        });
    }

    let minute = parse_minute_second(cursor, false)?;

    if cursor.check_or(false, |ch| is_time_separator(ch) || ch.is_ascii_digit()) {
        let is_time_separator = cursor.check_or(false, is_time_separator);
        if separator && is_time_separator {
            cursor.advance();
        } else if is_time_separator {
            return Err(TemporalError::syntax().with_message("Invalid TimeSeparator"));
        }
    } else {
        return Ok(TimeSpec {
            hour,
            minute,
            second: 0,
            fraction: 0.0,
        });
    }

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
    let hour_value = cursor
        .slice(cursor.pos(), cursor.pos() + 2)
        .parse::<u8>()
        .map_err(|e| TemporalError::syntax().with_message(e.to_string()))?;
    if !(0..=23).contains(&hour_value) {
        return Err(TemporalError::syntax().with_message("Hour must be in a range of 0-23"));
    }
    cursor.advance_n(2);
    Ok(hour_value)
}

// NOTE: `TimeSecond` is a 60 inclusive `MinuteSecond`.
/// Parse `MinuteSecond`
pub(crate) fn parse_minute_second(cursor: &mut Cursor, inclusive: bool) -> TemporalResult<u8> {
    let min_sec_value = cursor
        .slice(cursor.pos(), cursor.pos() + 2)
        .parse::<u8>()
        .map_err(|e| TemporalError::syntax().with_message(e.to_string()))?;

    let valid_range = if inclusive { 0..=60 } else { 0..=59 };
    if !valid_range.contains(&min_sec_value) {
        return Err(TemporalError::syntax().with_message("MinuteSecond must be in a range of 0-59"));
    }

    cursor.advance_n(2);
    Ok(min_sec_value)
}

/// Parse a `Fraction` value
///
/// This is primarily used in ISO8601 to add percision past
/// a second.
pub(crate) fn parse_fraction(cursor: &mut Cursor) -> TemporalResult<f64> {
    // Decimal is skipped by next call.
    let mut fraction_components = Vec::from(['.']);
    while let Some(ch) = cursor.next() {
        if !ch.is_ascii_digit() {
            if fraction_components.len() > 10 {
                return Err(
                    TemporalError::syntax().with_message("Fraction exceeds 9 DecimalDigits")
                );
            }

            let fraction_value = fraction_components
                .iter()
                .collect::<String>()
                .parse::<f64>()
                .map_err(|e| TemporalError::syntax().with_message(e.to_string()))?;
            return Ok(fraction_value);
        }
        fraction_components.push(ch);
    }

    Err(TemporalError::abrupt_end())
}
