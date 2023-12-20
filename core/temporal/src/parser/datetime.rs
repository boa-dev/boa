//! Parsing for Temporal's ISO8601 `Date` and `DateTime`.

use crate::{
    assert_syntax,
    parser::{
        annotations,
        grammar::{is_date_time_separator, is_sign, is_utc_designator},
        nodes::TimeZone,
        time,
        time::TimeSpec,
        time_zone, Cursor, IsoParseRecord,
    },
    TemporalError, TemporalResult,
};

use super::grammar::{is_annotation_open, is_hyphen};
use bitflags::bitflags;

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

bitflags! {
    /// Parsing flags for `AnnotatedDateTime` parsing.
    #[derive(Debug, Clone, Copy)]
    pub struct DateTimeFlags: u8 {
        const ZONED = 0b0000_0001;
        const TIME_REQ = 0b0000_0010;
        const UTC_REQ = 0b0000_0100;
    }
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
    flags: DateTimeFlags,
    cursor: &mut Cursor,
) -> TemporalResult<IsoParseRecord> {
    let date_time = parse_date_time(
        flags.contains(DateTimeFlags::TIME_REQ),
        flags.contains(DateTimeFlags::UTC_REQ),
        cursor,
    )?;

    // Peek Annotation presence
    // Throw error if annotation does not exist and zoned is true, else return.
    if !cursor.check_or(false, is_annotation_open) {
        if flags.contains(DateTimeFlags::ZONED) {
            return Err(TemporalError::syntax()
                .with_message("ZonedDateTime must have a TimeZoneAnnotation."));
        }

        cursor.close()?;

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

    let annotation_set =
        annotations::parse_annotation_set(flags.contains(DateTimeFlags::ZONED), cursor)?;

    if let Some(annotated_tz) = annotation_set.tz {
        tz = annotated_tz.tz;
    }

    let tz = if tz.name.is_some() || tz.offset.is_some() {
        Some(tz)
    } else {
        None
    };

    cursor.close()?;

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
    cursor: &mut Cursor,
) -> TemporalResult<DateTimeRecord> {
    let date = parse_date(cursor)?;

    // If there is no `DateTimeSeparator`, return date early.
    if !cursor.check_or(false, is_date_time_separator) {
        if time_required {
            return Err(TemporalError::syntax().with_message("Missing a required Time target."));
        }

        return Ok(DateTimeRecord {
            date,
            time: None,
            time_zone: None,
        });
    }

    cursor.advance();

    let time = time::parse_time_spec(cursor)?;

    let time_zone = if cursor.check_or(false, |ch| is_sign(ch) || is_utc_designator(ch)) {
        Some(time_zone::parse_date_time_utc(cursor)?)
    } else {
        if utc_required {
            return Err(TemporalError::syntax().with_message("DateTimeUTCOffset is required."));
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
fn parse_date(cursor: &mut Cursor) -> TemporalResult<DateRecord> {
    let year = parse_date_year(cursor)?;
    let hyphenated = cursor
        .check(is_hyphen)
        .ok_or_else(TemporalError::abrupt_end)?;

    cursor.advance_if(hyphenated);

    let month = parse_date_month(cursor)?;

    if hyphenated {
        assert_syntax!(cursor.check_or(false, is_hyphen), "Invalid hyphen usage.");
    }
    cursor.advance_if(cursor.check_or(false, is_hyphen));

    let day = parse_date_day(cursor)?;

    Ok(DateRecord { year, month, day })
}

// ==== `YearMonth` and `MonthDay` parsing functions ====

/// Parses a `DateSpecYearMonth`
pub(crate) fn parse_year_month(cursor: &mut Cursor) -> TemporalResult<(i32, i32)> {
    let year = parse_date_year(cursor)?;

    cursor.advance_if(cursor.check_or(false, is_hyphen));

    let month = parse_date_month(cursor)?;

    assert_syntax!(
        cursor.check_or(true, is_annotation_open),
        "Expected an end or AnnotationOpen"
    );

    Ok((year, month))
}

/// Parses a `DateSpecMonthDay`
pub(crate) fn parse_month_day(cursor: &mut Cursor) -> TemporalResult<(i32, i32)> {
    let dash_one = cursor
        .check(is_hyphen)
        .ok_or_else(TemporalError::abrupt_end)?;
    let dash_two = cursor
        .peek()
        .map(is_hyphen)
        .ok_or_else(TemporalError::abrupt_end)?;

    if dash_two && dash_one {
        cursor.advance_n(2);
    } else if dash_two && !dash_one {
        return Err(TemporalError::syntax().with_message("MonthDay requires two dashes"));
    }

    let month = parse_date_month(cursor)?;

    cursor.advance_if(cursor.check_or(false, is_hyphen));

    let day = parse_date_day(cursor)?;

    assert_syntax!(
        cursor.check_or(true, is_annotation_open),
        "Expected an end or AnnotationOpen"
    );

    Ok((month, day))
}

// ==== Unit Parsers ====

fn parse_date_year(cursor: &mut Cursor) -> TemporalResult<i32> {
    if cursor.check_or(false, is_sign) {
        let sign = if cursor.expect_next() == '+' { 1 } else { -1 };
        let year_start = cursor.pos();

        for _ in 0..6 {
            let year_digit = cursor.abrupt_next()?;
            assert_syntax!(
                year_digit.is_ascii_digit(),
                "Year must be made up of digits."
            );
        }

        let year_value = cursor
            .slice(year_start, cursor.pos())
            .parse::<i32>()
            .map_err(|e| TemporalError::syntax().with_message(e.to_string()))?;

        // 13.30.1 Static Semantics: Early Errors
        //
        // It is a Syntax Error if DateYear is "-000000" or "−000000" (U+2212 MINUS SIGN followed by 000000).
        if sign == -1 && year_value == 0 {
            return Err(TemporalError::syntax().with_message("Cannot have negative 0 years."));
        }

        let year = sign * year_value;

        if !(-271_820..=275_760).contains(&year) {
            return Err(TemporalError::range()
                .with_message("Year is outside of the minimum supported range."));
        }

        return Ok(year);
    }

    let year_start = cursor.pos();

    for _ in 0..4 {
        let year_digit = cursor.abrupt_next()?;
        assert_syntax!(
            year_digit.is_ascii_digit(),
            "Year must be made up of digits."
        );
    }

    let year_value = cursor
        .slice(year_start, cursor.pos())
        .parse::<i32>()
        .map_err(|e| TemporalError::syntax().with_message(e.to_string()))?;

    Ok(year_value)
}

fn parse_date_month(cursor: &mut Cursor) -> TemporalResult<i32> {
    let start = cursor.pos();
    for _ in 0..2 {
        let digit = cursor.abrupt_next()?;
        assert_syntax!(digit.is_ascii_digit(), "Month must be a digit");
    }
    let month_value = cursor
        .slice(start, cursor.pos())
        .parse::<i32>()
        .map_err(|e| TemporalError::syntax().with_message(e.to_string()))?;
    if !(1..=12).contains(&month_value) {
        return Err(TemporalError::syntax().with_message("DateMonth must be in a range of 1-12"));
    }
    Ok(month_value)
}

fn parse_date_day(cursor: &mut Cursor) -> TemporalResult<i32> {
    let start = cursor.pos();
    for _ in 0..2 {
        let digit = cursor.abrupt_next()?;
        assert_syntax!(digit.is_ascii_digit(), "Date must be a digit");
    }
    let day_value = cursor
        .slice(start, cursor.pos())
        .parse::<i32>()
        .map_err(|e| TemporalError::syntax().with_message(e.to_string()))?;
    if !(1..=31).contains(&day_value) {
        return Err(TemporalError::syntax().with_message("DateDay must be in a range of 1-31"));
    }
    Ok(day_value)
}
