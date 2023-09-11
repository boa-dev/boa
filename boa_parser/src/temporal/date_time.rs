//! Parsing for Temporal's ISO8601 `Date` and `DateTime`.

use crate::{
    error::{Error, ParseResult},
    lexer::Error as LexError,
    temporal::{
        IsoCursor,
        grammar::*,
        time_zone,
        time,
        annotations,
    }
};

use boa_ast::{
    Position, Span,
    temporal::{AnnotatedDateTime, DateRecord,DateTimeRecord}
};

/// `AnnotatedDateTime`
///
/// Defined in Temporal Proposal as follows:
///
/// AnnotatedDateTime[Zoned] :
///     [~Zoned] DateTime TimeZoneAnnotation(opt) Annotations(opt)
///     [+Zoned] DateTime TimeZoneAnnotation Annotations(opt)
pub(crate) fn parse_annotated_date_time(
    zoned: bool,
    cursor: &mut IsoCursor,
) -> ParseResult<AnnotatedDateTime> {
    let date_time = parse_date_time(cursor)?;

    // Peek Annotation presence
    // Throw error if annotation does not exist and zoned is true, else return.
    let annotation_check = cursor.peek().map(|ch| *ch == '[').unwrap_or(false);
    if !annotation_check {
        if zoned {
            return Err(Error::expected(
            ["TimeZoneAnnotation".into()],
                "No Annotation",
                Span::new(
                    Position::new(1, (cursor.pos() + 1) as u32),
                    Position::new(1, (cursor.pos() + 1) as u32),
                ),
                "iso8601 grammar",
            ));
        }
        return Ok(AnnotatedDateTime { date_time, tz_annotation: None, annotations: None });
    }

    // Parse the first annotation.
    let tz_annotation = time_zone::parse_ambiguous_tz_annotation(cursor)?;

    if tz_annotation.is_none() && zoned {
        return Err(Error::unexpected(
            "Annotation",
            Span::new(Position::new(1, (cursor.pos() + 1) as u32), Position::new(1, (cursor.pos() + 2) as u32)),
            "iso8601 ZonedDateTime requires a TimeZoneAnnotation.",
        ));
    }

    // Parse any `Annotations`
    let annotations = cursor.peek().map(|ch| *ch == '[').unwrap_or(false);

    if annotations {
        let annotations = annotations::parse_annotations(cursor)?;
        return Ok(AnnotatedDateTime { date_time, tz_annotation, annotations: Some(annotations) })
    }

    Ok(AnnotatedDateTime { date_time, tz_annotation, annotations: None })
}

fn parse_date_time(cursor: &mut IsoCursor) -> ParseResult<DateTimeRecord> {
    let date = parse_date(cursor)?;

    // If there is no `DateTimeSeparator`, return date early.
    if !cursor
        .peek()
        .map(|c| is_date_time_separator(c))
        .unwrap_or(false)
    {
        return Ok(DateTimeRecord {
            date,
            time: None,
            offset: None,
        });
    }

    cursor.advance();

    let time = time::parse_time_spec(cursor)?;

    let offset = if cursor
        .peek()
        .map(|ch| is_sign(ch) || is_utc_designator(ch))
        .unwrap_or(false)
    {
        Some(time_zone::parse_date_time_utc(cursor)?)
    } else {
        None
    };

    Ok(DateTimeRecord { date, time: Some(time), offset })
}


/// Parse `Date`
fn parse_date(cursor: &mut IsoCursor) -> ParseResult<DateRecord> {
    let year = parse_date_year(cursor)?;
    let divided = cursor
        .peek()
        .map(|ch| *ch == '-')
        .ok_or_else(|| Error::AbruptEnd)?;

    if divided {
        cursor.advance();
    }

    let month = parse_date_month(cursor)?;

    if cursor.peek().map(|ch| *ch == '-').unwrap_or(false) {
        if !divided {
            return Err(LexError::syntax(
                "Invalid date separator",
                Position::new(1, (cursor.pos() + 1) as u32),
            ).into());
        }
        cursor.advance();
    }

    let day = parse_date_day(cursor)?;

    Ok(DateRecord { year, month, day })
}

// ==== Unit Parsers ====
// (referring to Year, month, day, hour, sec, etc...)

fn parse_date_year(cursor: &mut IsoCursor) -> ParseResult<i32> {
    if is_sign(cursor.peek().ok_or_else(|| Error::AbruptEnd)?) {
        let year_start = cursor.pos();
        let sign = if *cursor.peek().unwrap() == '+' { 1 } else { -1 };

        cursor.advance();

        for _ in 0..6 {
            let year_digit = cursor.peek().ok_or_else(|| Error::AbruptEnd)?;
            if !year_digit.is_ascii_digit() {
                return Err(Error::lex(LexError::syntax(
                    "DateYear must contain digit",
                    Position::new(1, (cursor.pos() + 1) as u32),
                )));
            }
            cursor.advance();
        }

        let year_string = cursor.slice(year_start + 1, cursor.pos());
        let year_value = year_string.parse::<i32>().map_err(|e| Error::general(e.to_string(), Position::new(1, (year_start + 1) as u32)))?;

        // 13.30.1 Static Semantics: Early Errors
        //
        // It is a Syntax Error if DateYear is "-000000" or "−000000" (U+2212 MINUS SIGN followed by 000000).
        if sign == -1 && year_value == 0 {
            return Err(Error::lex(LexError::syntax(
                "Cannot have negative 0 years.",
                Position::new(1, (year_start + 1) as u32),
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
                Position::new(1, (cursor.pos() + 1) as u32),
            ).into());
        }
        cursor.advance();
    }

    let year_string = cursor.slice(year_start, cursor.pos());
    let year_value = year_string.parse::<i32>().map_err(|e| Error::general(e.to_string(), Position::new(1, (cursor.pos() + 1) as u32)))?;

    return Ok(year_value);
}

fn parse_date_month(cursor: &mut IsoCursor) -> ParseResult<i32> {
    let month_value = cursor
        .slice(cursor.pos(), cursor.pos() + 2)
        .parse::<i32>()
        .map_err(|e| Error::general(e.to_string(), Position::new(1, (cursor.pos() + 1) as u32)))?;
    if !(1..=12).contains(&month_value) {
        return Err(LexError::syntax(
            "DateMonth must be in a range of 1-12",
            Position::new(1, (cursor.pos() + 1) as u32),
        ).into());
    }
    cursor.advance_n(2);
    Ok(month_value)
}

fn parse_date_day(cursor: &mut IsoCursor) -> ParseResult<i32> {
    let day_value = cursor
        .slice(cursor.pos(), cursor.pos() + 2)
        .parse::<i32>()
        .map_err(|e| Error::general(e.to_string(), Position::new(1, cursor.pos() as u32)))?;
    if !(1..=31).contains(&day_value) {
        return Err(LexError::syntax(
            "DateDay must be in a range of 1-31",
            Position::new(1, (cursor.pos() + 1) as u32),
        ).into());
    }
    cursor.advance_n(2);
    Ok(day_value)
}
