//! Parsing of ISO8601 Time Values

use super::{grammar::is_decimal_separator, IsoCursor};
use crate::{
    error::{Error, ParseResult},
    lexer::Error as LexError,
};

use boa_ast::{temporal::TimeSpec, Position};

/// Parse `TimeSpec`
pub(crate) fn parse_time_spec(cursor: &mut IsoCursor) -> ParseResult<TimeSpec> {
    let hour = parse_hour(cursor)?;
    let mut separator = false;

    if cursor.check_or(false, |ch| ch == ':' || ch.is_ascii_digit()) {
        if cursor.check_or(false, |ch| ch == ':') {
            separator = true;
            cursor.advance();
        }
    } else {
        return Ok(TimeSpec {
            hour,
            minute: 0,
            second: 0.0,
        });
    }

    let minute = parse_minute_second(cursor, false)?;

    if cursor.check_or(false, |ch| ch == ':' || ch.is_ascii_digit()) {
        let is_time_separator = cursor.check_or(false, |ch| ch == ':');
        if separator && is_time_separator {
            cursor.advance();
        } else if is_time_separator {
            return Err(
                LexError::syntax("Invalid TimeSeparator", Position::new(1, cursor.pos())).into(),
            );
        }
    } else {
        return Ok(TimeSpec {
            hour,
            minute,
            second: 0.0,
        });
    }

    let second = parse_minute_second(cursor, true)?;

    let double = if cursor.check_or(false, is_decimal_separator) {
        f64::from(second) + parse_fraction(cursor)?
    } else {
        f64::from(second)
    };

    Ok(TimeSpec {
        hour,
        minute,
        second: double,
    })
}

pub(crate) fn parse_hour(cursor: &mut IsoCursor) -> ParseResult<i8> {
    let hour_value = cursor
        .slice(cursor.pos(), cursor.pos() + 2)
        .parse::<i8>()
        .map_err(|e| Error::general(e.to_string(), Position::new(1, cursor.pos())))?;
    if !(0..=23).contains(&hour_value) {
        return Err(LexError::syntax(
            "Hour must be in a range of 0-23",
            Position::new(1, cursor.pos() + 1),
        )
        .into());
    }
    cursor.advance_n(2);
    Ok(hour_value)
}

// NOTE: `TimeSecond` is a 60 inclusive `MinuteSecond`.
/// Parse `MinuteSecond`
pub(crate) fn parse_minute_second(cursor: &mut IsoCursor, inclusive: bool) -> ParseResult<i8> {
    let min_sec_value = cursor
        .slice(cursor.pos(), cursor.pos() + 2)
        .parse::<i8>()
        .map_err(|e| Error::general(e.to_string(), Position::new(1, cursor.pos())))?;
    let valid_range = if inclusive { 0..=60 } else { 0..=59 };
    if !valid_range.contains(&min_sec_value) {
        return Err(LexError::syntax(
            "MinuteSecond must be in a range of 0-59",
            Position::new(1, cursor.pos() + 1),
        )
        .into());
    }
    cursor.advance_n(2);
    Ok(min_sec_value)
}

/// Parse a `Fraction` value
///
/// This is primarily used in ISO8601 to add percision past
/// a second.
pub(crate) fn parse_fraction(cursor: &mut IsoCursor) -> ParseResult<f64> {
    let fraction_start = cursor.pos();
    cursor.advance();

    // TODO: implement error for going past 9 Digit values.
    while let Some(ch) = cursor.next() {
        if !ch.is_ascii_digit() {
            let frac = cursor
                .slice(fraction_start, cursor.pos())
                .parse::<f64>()
                .map_err(|e| Error::general(e.to_string(), Position::new(1, cursor.pos() - 1)))?;
            return Ok(frac);
        }
    }

    Err(Error::AbruptEnd)
}
