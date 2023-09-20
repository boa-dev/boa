//! Parsing for Temporal's ISO8601 `Date` and `DateTime`.

use crate::{
    error::{Error, ParseResult},
    lexer::Error as LexError,
    temporal::{
        annotations,
        grammar::{is_date_time_separator, is_sign, is_utc_designator},
        time, time_zone, IsoCursor,
    },
};

use boa_ast::{
    temporal::{DateRecord, DateTimeRecord, IsoParseRecord},
    Position, Span,
};

/// `AnnotatedDateTime`
///
/// Defined in Temporal Proposal as follows:
///
/// `AnnotatedDateTime`[Zoned] :
///     [~Zoned] `DateTime` `TimeZoneAnnotation`(opt) `Annotations`(opt)
///     [+Zoned] `DateTime` `TimeZoneAnnotation` `Annotations`(opt)
pub(crate) fn parse_annotated_date_time(
    zoned: bool,
    cursor: &mut IsoCursor,
) -> ParseResult<IsoParseRecord> {
    let date_time = parse_date_time(cursor)?;

    // Peek Annotation presence
    // Throw error if annotation does not exist and zoned is true, else return.
    let annotation_check = cursor.check_or(false, |ch| ch == '[');
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
            offset: date_time.offset,
            tz_annotation: None,
            calendar: None,
        });
    }

    let annotation_set = annotations::parse_annotation_set(zoned, cursor)?;

    Ok(IsoParseRecord {
        date: date_time.date,
        time: date_time.time,
        offset: date_time.offset,
        tz_annotation: annotation_set.tz,
        calendar: annotation_set.calendar,
    })
}

/// Parses a `DateTime` record.
fn parse_date_time(cursor: &mut IsoCursor) -> ParseResult<DateTimeRecord> {
    let date = parse_date(cursor)?;

    // If there is no `DateTimeSeparator`, return date early.
    if !cursor.check_or(false, is_date_time_separator) {
        return Ok(DateTimeRecord {
            date,
            time: None,
            offset: None,
        });
    }

    cursor.advance();

    let time = time::parse_time_spec(cursor)?;

    let offset = if cursor
        .check(|ch| is_sign(ch) || is_utc_designator(ch))
        .unwrap_or(false)
    {
        Some(time_zone::parse_date_time_utc(cursor)?)
    } else {
        None
    };

    Ok(DateTimeRecord {
        date,
        time: Some(time),
        offset,
    })
}

/// Parses `Date` record.
fn parse_date(cursor: &mut IsoCursor) -> ParseResult<DateRecord> {
    let year = parse_date_year(cursor)?;
    let divided = cursor
        .check(|ch| ch == '-')
        .ok_or_else(|| Error::AbruptEnd)?;

    if divided {
        cursor.advance();
    }

    let month = parse_date_month(cursor)?;

    if cursor.check_or(false, |ch| ch == '-') {
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
pub(crate) fn peek_year_month(cursor: &mut IsoCursor) -> ParseResult<bool> {
    let mut ym_peek = if is_sign(cursor.peek().ok_or_else(|| Error::AbruptEnd)?) {
        7
    } else {
        4
    };

    if cursor
        .peek_n(ym_peek)
        .map(|ch| ch == '-')
        .ok_or_else(|| Error::AbruptEnd)?
    {
        ym_peek += 1;
    }

    ym_peek += 2;

    if cursor.peek_n(ym_peek).map_or(true, |ch| ch == '[') {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Parses a `DateSpecYearMonth`
pub(crate) fn parse_year_month(cursor: &mut IsoCursor) -> ParseResult<(i32, i32)> {
    let year = parse_date_year(cursor)?;

    if cursor.check_or(false, |ch| ch == '-') {
        cursor.advance();
    }

    let month = parse_date_month(cursor)?;

    Ok((year, month))
}

/// Determines if the string can be parsed as a `DateSpecYearMonth`.
pub(crate) fn peek_month_day(cursor: &mut IsoCursor) -> ParseResult<bool> {
    let mut md_peek = if cursor
        .peek_n(1)
        .map(|ch| ch == '-')
        .ok_or_else(|| Error::AbruptEnd)?
    {
        4
    } else {
        2
    };

    if cursor
        .peek_n(md_peek)
        .map(|ch| ch == '-')
        .ok_or_else(|| Error::AbruptEnd)?
    {
        md_peek += 1;
    }

    md_peek += 2;

    if cursor.peek_n(md_peek).map_or(true, |ch| ch == '[') {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Parses a `DateSpecMonthDay`
pub(crate) fn parse_month_day(cursor: &mut IsoCursor) -> ParseResult<(i32, i32)> {
    let dash_one = cursor
        .check(|ch| ch == '-')
        .ok_or_else(|| Error::AbruptEnd)?;
    let dash_two = cursor
        .peek_n(1)
        .map(|ch| ch == '-')
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
    if cursor.check_or(false, |ch| ch == '-') {
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
