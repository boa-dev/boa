//! Implementation of ISO8601 grammar lexing/parsing
use crate::error::ParseResult;

mod annotations;
mod date_time;
mod grammar;
#[cfg(test)]
mod tests;
mod time;
mod time_zone;

use boa_ast::temporal::{DateRecord, IsoParseRecord, TzIdentifier};

// TODO: optimize where possible.
//
// NOTE:
// Rough max length source given iso calendar and no extraneous annotations
// is ~100 characters (+10-20 for some calendars):
// +001970-01-01T00:00:00.000000000+00:00:00.000000000[!America/Argentina/ComodRivadavia][!u-ca=iso8601]

/// Parse a `TemporalDateTimeString`.
#[derive(Debug, Clone, Copy)]
pub struct TemporalDateTimeString;

impl TemporalDateTimeString {
    /// Parses a targeted `DateTimeString`
    ///
    /// # Errors
    ///
    /// The parse will error if the provided target is not valid
    /// ISO8601 grammar..
    pub fn parse(zoned: bool, cursor: &mut IsoCursor) -> ParseResult<IsoParseRecord> {
        date_time::parse_annotated_date_time(zoned, cursor)
    }
}

/// Parse a `TemporalTimeZoneString`
#[derive(Debug, Clone, Copy)]
pub struct TemporalTimeZoneString;

impl TemporalTimeZoneString {
    /// Parses a targeted `TimeZoneString`
    ///
    /// # Errors
    ///
    /// The parse will error if the provided target is not valid
    /// ISO8601 grammar..
    pub fn parse(cursor: &mut IsoCursor) -> ParseResult<TzIdentifier> {
        time_zone::parse_tz_identifier(cursor)
    }
}

/// Parse a `TemporalYearMonthString`
#[derive(Debug, Clone, Copy)]
pub struct TemporalYearMonthString;

impl TemporalYearMonthString {
    /// Parses a targeted `YearMonthString`.
    ///
    /// # Errors
    ///
    /// The parse will error if the provided target is not valid
    /// ISO8601 grammar.
    pub fn parse(cursor: &mut IsoCursor) -> ParseResult<IsoParseRecord> {
        if date_time::peek_year_month(cursor)? {
            let ym = date_time::parse_year_month(cursor)?;

            let (tz_annotation, annotations) = if cursor.check_or(false, |ch| ch == '[') {
                let set = annotations::parse_annotation_set(false, cursor)?;
                (set.tz, set.annotations)
            } else {
                (None, None)
            };

            return Ok(IsoParseRecord {
                date: DateRecord {
                    year: ym.0,
                    month: ym.1,
                    day: 0,
                },
                time: None,
                offset: None,
                tz_annotation,
                annotations,
            });
        }

        date_time::parse_annotated_date_time(false, cursor)
    }
}

/// Parse a `TemporalMonthDayString`
#[derive(Debug, Clone, Copy)]
pub struct TemporalMonthDayString;

impl TemporalMonthDayString {
    /// Parses a targeted `MonthDayString`.
    ///
    /// # Errors
    ///
    /// The parse will error if the provided target is not valid
    /// ISO8601 grammar.
    pub fn parse(cursor: &mut IsoCursor) -> ParseResult<IsoParseRecord> {
        if date_time::peek_month_day(cursor)? {
            let md = date_time::parse_month_day(cursor)?;

            let (tz_annotation, annotations) = if cursor.check_or(false, |ch| ch == '[') {
                let set = annotations::parse_annotation_set(false, cursor)?;
                (set.tz, set.annotations)
            } else {
                (None, None)
            };

            return Ok(IsoParseRecord {
                date: DateRecord {
                    year: 0,
                    month: md.0,
                    day: md.1,
                },
                time: None,
                offset: None,
                tz_annotation,
                annotations,
            });
        }

        date_time::parse_annotated_date_time(false, cursor)
    }
}

// ==== Mini cursor implementation for ISO8601 targets ====

/// `IsoCursor` is a small cursor implementation for parsing ISO8601 grammar.
#[derive(Debug)]
pub struct IsoCursor {
    pos: usize,
    source: Vec<char>,
}

impl IsoCursor {
    /// Create a new cursor from a source `String` value.
    #[must_use]
    pub fn new(source: &str) -> Self {
        Self {
            pos: 0,
            source: source.chars().collect(),
        }
    }

    /// Returns a string value from a slice of the cursor.
    fn slice(&self, start: usize, end: usize) -> String {
        self.source[start..end].iter().collect()
    }

    /// Get current position
    const fn pos(&self) -> usize {
        self.pos
    }

    /// Peek the value at the current position.
    fn peek(&self) -> Option<char> {
        if self.pos < self.source.len() {
            Some(self.source[self.pos])
        } else {
            None
        }
    }

    /// Peek the value at n len from current.
    fn peek_n(&self, n: usize) -> Option<char> {
        if self.pos + n < self.source.len() {
            Some(self.source[self.pos + n])
        } else {
            None
        }
    }

    /// Returns boolean if current position passes check.
    fn check<F>(&self, f: F) -> Option<bool>
    where
        F: FnOnce(char) -> bool,
    {
        self.peek().map(f)
    }

    /// Returns boolean if current position passes check or default if None.
    fn check_or<F>(&self, default: bool, f: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        self.peek().map_or(default, f)
    }
    /// Advances the cursor's position and returns the new character.
    fn next(&mut self) -> Option<char> {
        self.advance();
        self.peek()
    }

    /// Advances the cursor's position by 1.
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Advances the cursor's position by `n`.
    fn advance_n(&mut self, n: usize) {
        self.pos += n;
    }
}
