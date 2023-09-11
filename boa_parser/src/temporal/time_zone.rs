//! ISO8601 parsing for Time Zone and Offset data.

use crate::{
    error::{Error, ParseResult},
    lexer::Error as LexError,
};
use super::{
    IsoCursor,
    time::{parse_minute_second, parse_fraction, parse_hour},
    grammar::*,
};

use boa_ast::{Position, temporal::{TimeZoneAnnotation, UtcOffset, TzIdentifier}};

// ==== Time Zone Annotation Parsing ====

pub(crate) fn parse_ambiguous_tz_annotation(cursor: &mut IsoCursor) -> ParseResult<Option<TimeZoneAnnotation>> {
    // Peek position + 1 to check for critical flag.
    let mut current_peek = 1;
    let critical = cursor
        .peek_n(current_peek)
        .map(|ch| *ch == '!')
        .ok_or_else(|| Error::AbruptEnd)?;

    // Advance cursor if critical flag present.
    if critical {
        current_peek += 1;
    }

    let leading_char = cursor
        .peek_n(current_peek)
        .ok_or_else(|| Error::AbruptEnd)?;

    match is_tz_leading_char(leading_char) || is_sign(leading_char) {
        // Ambigious start values when lowercase alpha that is shared between `TzLeadingChar` and `KeyLeadingChar`.
        true if is_a_key_leading_char(leading_char) => {
            let mut peek_pos = current_peek + 1;
            while let Some(ch) = cursor.peek_n(peek_pos) {
                if *ch == '/' || (is_tz_char(ch) && !is_a_key_char(ch)) {
                    let tz = parse_tz_annotation(cursor)?;
                    return Ok(Some(tz));
                } else if *ch == '=' || (is_a_key_char(ch) && !is_tz_char(ch)) {
                    return Ok(None);
                } else if *ch == ']' {
                    return Err(LexError::syntax(
                        "Invalid Annotation",
                        Position::new(1, (peek_pos + 1) as u32),
                    ).into());
                }

                peek_pos += 1;
            }
            Err(Error::AbruptEnd)
        }
        true => {
            let tz = parse_tz_annotation(cursor)?;
            Ok(Some(tz))
        }
        false if is_a_key_leading_char(leading_char) => {
            Ok(None)
        }
        _ => Err(Error::lex(LexError::syntax(
            "Unexpected character in ambiguous annotation.",
            Position::new(1, (cursor.pos() + 1) as u32),
        ))),
    }
}

fn parse_tz_annotation(cursor: &mut IsoCursor) -> ParseResult<TimeZoneAnnotation> {
    assert!(*cursor.peek().unwrap() == '[');

    let potential_critical = cursor.next().ok_or_else(|| Error::AbruptEnd)?;
    let critical = *potential_critical == '!';

    if critical {
        cursor.advance();
    }

    let tz = parse_tz_identifier(cursor)?;

    if !cursor.peek().map(|ch| *ch == ']').unwrap_or(false) {
        return Err(LexError::syntax("Invalid TimeZoneAnnotation.", Position::new(1, (cursor.pos() + 1) as u32)).into())
    }

    cursor.advance();

    Ok(TimeZoneAnnotation { critical, tz })
}

pub(crate) fn parse_tz_identifier(cursor: &mut IsoCursor) -> ParseResult<TzIdentifier> {
    let is_iana = cursor.peek().map(|ch| is_tz_leading_char(ch)).ok_or_else(|| Error::AbruptEnd)?;
    let is_offset = cursor.peek().map(|ch| is_sign(ch)).unwrap_or(false);

    if is_iana {
        let iana_name = parse_tz_iana_name(cursor)?;
        return Ok(TzIdentifier::TzIANAName(iana_name));
    } else if is_offset {
        let offset = parse_utc_offset_minute_precision(cursor)?;
        return Ok(TzIdentifier::UtcOffset(offset))
    }

    Err(LexError::syntax("Invalid leading character for a TimeZoneIdentifier", Position::new(1, (cursor.pos() + 1) as u32)).into())
}

/// Parse a `TimeZoneIANAName` Parse Node
fn parse_tz_iana_name(cursor: &mut IsoCursor) -> ParseResult<String> {
    let tz_name_start = cursor.pos();
    while let Some(potential_value_char) = cursor.next() {
        if *potential_value_char == '/' {
            if !cursor
                .peek_n(1)
                .map(|ch| is_tz_char(ch))
                .unwrap_or(false)
            {
                return Err(LexError::syntax(
                    "Missing TimeZoneIANANameComponent after '/'",
                    Position::new(1, (cursor.pos() + 2) as u32),
                ).into());
            }
            continue;
        }

        if !is_tz_char(potential_value_char) {
            // Return the valid TimeZoneIANAName
            return Ok(cursor.slice(tz_name_start, cursor.pos()));
        }

    }

    return Err(Error::AbruptEnd);
}

// ==== Utc Offset Parsing ====

/// Parse a full precision `UtcOffset`
pub(crate) fn parse_date_time_utc(cursor: &mut IsoCursor) -> ParseResult<UtcOffset> {
    if cursor.peek().map(|ch| is_utc_designator(ch)).unwrap_or(false) {
        cursor.advance();
        return Ok(UtcOffset { utc: true, sign: 1, hour: 0, minute: 0, second: 0.0 })
    }

    let separated = cursor.peek_n(3).map(|ch| *ch == ':').unwrap_or(false);

    let mut utc_to_minute = parse_utc_offset_minute_precision(cursor)?;

    if cursor.peek().map(|ch| *ch == ':').unwrap_or(false) {
        if !separated {
            return Err(LexError::syntax("Unexpected TimeSeparator", Position::new(1, cursor.pos() as u32)).into())
        }
        cursor.advance();
    }

    let sec = parse_minute_second(cursor, true)?;

    let double = if cursor.peek().map(|ch| is_decimal_separator(ch)).unwrap_or(false) {
        f64::from(sec) + parse_fraction(cursor)?
    } else {
        f64::from(sec)
    };

    utc_to_minute.second = double;
    Ok(utc_to_minute)
}

/// Parse an `UtcOffsetMinutePrecision` node
pub(crate) fn parse_utc_offset_minute_precision(cursor: &mut IsoCursor) -> ParseResult<UtcOffset> {
    let sign = if let Some(ch) = cursor.next() { if *ch == '+' { 1_i8 } else { -1_i8 }} else {return Err(Error::AbruptEnd)};
    let hour = parse_hour(cursor)?;

    // If at the end of the utc, then return.
    if cursor.peek().map(|ch| !(ch.is_ascii_digit() || *ch == ':')).ok_or_else(|| Error::AbruptEnd)? {
        return Ok(UtcOffset { utc: false, sign, hour, minute: 0, second: 0.0 })
    }

    // Advance cursor beyond any TimeSeparator
    if cursor.peek().map(|ch| *ch == ':').unwrap_or(false) {
        cursor.advance();
    }

    let minute = parse_minute_second(cursor, false)?;

    return Ok(UtcOffset { utc: false, sign, hour, minute, second: 0.0 })
}