//! Parsing for Temporal's ISO8601 `Date` and `DateTime`.

use crate::{
    error::{Error, ParseResult},
    lexer::Error as LexError,
    temporal::{
        annotations,
        grammar::{is_date_time_separator, is_sign, is_utc_designator},
        time,
        time::TimeSpec,
        time_zone, IsoCursor, IsoParseRecord,
    },
};

use boa_ast::{temporal::TimeZone, Position, Span};

use super::grammar::{is_annotation_open, is_hyphen};

#[derive(Debug, Default, Clone)]
/// A `DateTime` Parse Node that contains the date, time, and offset info.
pub(crate) struct DateTimeRecord {
    /// Date
    pub(crate) date: DateRecord,
    /// Time
    pub(crate) time: Option<TimeSpec>,
    /// Tz Offset
    pub(crate) time_zone: Option<TimeZone>,
}

#[derive(Default, Debug, Clone, Copy)]
/// The record of a parsed date.
pub(crate) struct DateRecord {
    /// Date Year
    pub(crate) year: i32,
    /// Date Month
    pub(crate) month: i32,
    /// Date Day
    pub(crate) day: i32,
}

/// This function handles parsing for [`AnnotatedDateTime`][datetime],
/// [`AnnotatedDateTimeTimeRequred`][time], and
/// [`TemporalInstantString.`][instant] according to the requirements
/// provided via Spec.
///
/// [datetime]: https://tc39.es/proposal-temporal/#prod-AnnotatedDateTime
/// [time]: https://tc39.es/proposal-temporal/#prod-AnnotatedDateTimeTimeRequired
/// [instant]: https://tc39.es/proposal-temporal/#prod-TemporalInstantString
pub(crate) fn parse_annotated_date_time(
    zoned: bool,
    time_required: bool,
    utc_required: bool,
    cursor: &mut IsoCursor,
) -> ParseResult<IsoParseRecord> {
    let date_time = parse_date_time(time_required, utc_required, cursor)?;

    // Peek Annotation presence
    // Throw error if annotation does not exist and zoned is true, else return.
    let annotation_check = cursor.check_or(false, is_annotation_open);
    if !annotation_check {
        if zoned {
            return Err(Error::expected(
                ["TimeZoneAnnotation".into()],
                "No Annotation",
                Span::new(
                    Position::new(1, cursor.pos() + 1),
                    Position::new(1, cursor.pos() + 1),
                ),
                "iso8601 grammar",
            ));
        }

        return Ok(IsoParseRecord {
            date: date_time.date,
            time: date_time.time,
            tz: date_time.time_zone,
            calendar: None,
        });
    }

    let mut tz = TimeZone::default();

    if let Some(tz_info) = date_time.time_zone {
        tz = tz_info;
    }

    let annotation_set = annotations::parse_annotation_set(zoned, cursor)?;

    if let Some(annotated_tz) = annotation_set.tz {
        tz = annotated_tz.tz;
    }

    let tz = if tz.name.is_some() || tz.offset.is_some() {
        Some(tz)
    } else {
        None
    };

    Ok(IsoParseRecord {
        date: date_time.date,
        time: date_time.time,
        tz,
        calendar: annotation_set.calendar,
    })
}

/// Parses a `DateTime` record.
fn parse_date_time(
    time_required: bool,
    utc_required: bool,
    cursor: &mut IsoCursor,
) -> ParseResult<DateTimeRecord> {
    let date = parse_date(cursor)?;

    // If there is no `DateTimeSeparator`, return date early.
    if !cursor.check_or(false, is_date_time_separator) {
        if time_required {
            return Err(Error::general(
                "Missing a required TimeSpec.",
                Position::new(1, cursor.pos() + 1),
            ));
        }

        return Ok(DateTimeRecord {
            date,
            time: None,
            time_zone: None,
        });
    }

    cursor.advance();

    let time = time::parse_time_spec(cursor)?;

    let time_zone = if cursor
        .check(|ch| is_sign(ch) || is_utc_designator(ch))
        .unwrap_or(false)
    {
        Some(time_zone::parse_date_time_utc(cursor)?)
    } else {
        if utc_required {
            return Err(Error::general(
                "DateTimeUTCOffset is required.",
                Position::new(1, cursor.pos() + 1),
            ));
        }
        None
    };

    Ok(DateTimeRecord {
        date,
        time: Some(time),
        time_zone,
    })
}

/// Parses `Date` record.
fn parse_date(cursor: &mut IsoCursor) -> ParseResult<DateRecord> {
    let year = parse_date_year(cursor)?;
    let divided = cursor.check(is_hyphen).ok_or_else(|| Error::AbruptEnd)?;

    if divided {
        cursor.advance();
    }

    let month = parse_date_month(cursor)?;

    if cursor.check_or(false, is_hyphen) {
        if !divided {
            return Err(LexError::syntax(
                "Invalid date separator",
                Position::new(1, cursor.pos() + 1),
            )
            .into());
        }
        cursor.advance();
    }

    let day = parse_date_day(cursor)?;

    Ok(DateRecord { year, month, day })
}

/// Determines if the string can be parsed as a `DateSpecYearMonth`.
pub(crate) fn peek_year_month(cursor: &IsoCursor) -> ParseResult<bool> {
    let mut ym_peek = if is_sign(cursor.peek().ok_or_else(|| Error::AbruptEnd)?) {
        7
    } else {
        4
    };

    if cursor
        .peek_n(ym_peek)
        .map(is_hyphen)
        .ok_or_else(|| Error::AbruptEnd)?
    {
        ym_peek += 1;
    }

    ym_peek += 2;

    if cursor.peek_n(ym_peek).map_or(true, is_annotation_open) {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Parses a `DateSpecYearMonth`
pub(crate) fn parse_year_month(cursor: &mut IsoCursor) -> ParseResult<(i32, i32)> {
    let year = parse_date_year(cursor)?;

    if cursor.check_or(false, is_hyphen) {
        cursor.advance();
    }

    let month = parse_date_month(cursor)?;

    Ok((year, month))
}

/// Determines if the string can be parsed as a `DateSpecYearMonth`.
pub(crate) fn peek_month_day(cursor: &IsoCursor) -> ParseResult<bool> {
    let mut md_peek = if cursor
        .peek_n(1)
        .map(is_hyphen)
        .ok_or_else(|| Error::AbruptEnd)?
    {
        4
    } else {
        2
    };

    if cursor
        .peek_n(md_peek)
        .map(is_hyphen)
        .ok_or_else(|| Error::AbruptEnd)?
    {
        md_peek += 1;
    }

    md_peek += 2;

    if cursor.peek_n(md_peek).map_or(true, is_annotation_open) {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Parses a `DateSpecMonthDay`
pub(crate) fn parse_month_day(cursor: &mut IsoCursor) -> ParseResult<(i32, i32)> {
    let dash_one = cursor.check(is_hyphen).ok_or_else(|| Error::AbruptEnd)?;
    let dash_two = cursor
        .peek_n(1)
        .map(is_hyphen)
        .ok_or_else(|| Error::AbruptEnd)?;

    if dash_two && dash_one {
        cursor.advance_n(2);
    } else if dash_two && !dash_one {
        return Err(LexError::syntax(
            "MonthDay requires two dashes",
            Position::new(1, cursor.pos()),
        )
        .into());
    }

    let month = parse_date_month(cursor)?;
    if cursor.check_or(false, is_hyphen) {
        cursor.advance();
    }

    let day = parse_date_day(cursor)?;

    Ok((month, day))
}

// ==== Unit Parsers ====

fn parse_date_year(cursor: &mut IsoCursor) -> ParseResult<i32> {
    if is_sign(cursor.peek().ok_or_else(|| Error::AbruptEnd)?) {
        let year_start = cursor.pos();
        let sign = if cursor.check_or(false, |ch| ch == '+') {
            1
        } else {
            -1
        };

        cursor.advance();

        for _ in 0..6 {
            let year_digit = cursor.peek().ok_or_else(|| Error::AbruptEnd)?;
            if !year_digit.is_ascii_digit() {
                return Err(Error::lex(LexError::syntax(
                    "DateYear must contain digit",
                    Position::new(1, cursor.pos() + 1),
                )));
            }
            cursor.advance();
        }

        let year_string = cursor.slice(year_start + 1, cursor.pos());
        let year_value = year_string
            .parse::<i32>()
            .map_err(|e| Error::general(e.to_string(), Position::new(1, year_start + 1)))?;

        // 13.30.1 Static Semantics: Early Errors
        //
        // It is a Syntax Error if DateYear is "-000000" or "−000000" (U+2212 MINUS SIGN followed by 000000).
        if sign == -1 && year_value == 0 {
            return Err(Error::lex(LexError::syntax(
                "Cannot have negative 0 years.",
                Position::new(1, year_start + 1),
            )));
        }

        return Ok(sign * year_value);
    }

    let year_start = cursor.pos();

    for _ in 0..4 {
        let year_digit = cursor.peek().ok_or_else(|| Error::AbruptEnd)?;
        if !year_digit.is_ascii_digit() {
            return Err(LexError::syntax(
                "DateYear must contain digit",
                Position::new(1, cursor.pos() + 1),
            )
            .into());
        }
        cursor.advance();
    }

    let year_string = cursor.slice(year_start, cursor.pos());
    let year_value = year_string
        .parse::<i32>()
        .map_err(|e| Error::general(e.to_string(), Position::new(1, cursor.pos() + 1)))?;

    Ok(year_value)
}

fn parse_date_month(cursor: &mut IsoCursor) -> ParseResult<i32> {
    let month_value = cursor
        .slice(cursor.pos(), cursor.pos() + 2)
        .parse::<i32>()
        .map_err(|e| Error::general(e.to_string(), Position::new(1, cursor.pos() + 1)))?;
    if !(1..=12).contains(&month_value) {
        return Err(LexError::syntax(
            "DateMonth must be in a range of 1-12",
            Position::new(1, cursor.pos() + 1),
        )
        .into());
    }
    cursor.advance_n(2);
    Ok(month_value)
}

fn parse_date_day(cursor: &mut IsoCursor) -> ParseResult<i32> {
    let day_value = cursor
        .slice(cursor.pos(), cursor.pos() + 2)
        .parse::<i32>()
        .map_err(|e| Error::general(e.to_string(), Position::new(1, cursor.pos())))?;
    if !(1..=31).contains(&day_value) {
        return Err(LexError::syntax(
            "DateDay must be in a range of 1-31",
            Position::new(1, cursor.pos() + 1),
        )
        .into());
    }
    cursor.advance_n(2);
    Ok(day_value)
}
