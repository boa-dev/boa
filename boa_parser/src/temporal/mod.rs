//! Implementation of ISO8601 grammar lexing/parsing
#[allow(unused_variables)]

use crate::error::ParseResult;

mod tests;
mod time;
mod time_zone;
mod grammar;
mod date_time;
mod annotations;

use boa_ast::temporal::{AnnotatedDateTime, TzIdentifier};

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
    /// Parses a targeted `DateTimeString`.
    pub fn parse(zoned: bool, cursor: &mut IsoCursor) -> ParseResult<AnnotatedDateTime> {
        date_time::parse_annotated_date_time(zoned, cursor)
    }
}

/// Parse a `TemporalTimeZoneString`
#[derive(Debug, Clone, Copy)]
pub struct TemporalTimeZoneString;

impl TemporalTimeZoneString {
    /// Parses a targeted `TimeZoneString`.
    pub fn parse(cursor: &mut IsoCursor) -> ParseResult<TzIdentifier> {
        time_zone::parse_tz_identifier(cursor)
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
    pub fn new(source: String) -> Self {
        Self {
            pos: 0,
            source: source.chars().collect(),
        }
    }

    /// Returns a string value from a slice of the cursor.
    fn slice(&self, start: usize, end: usize) -> String {
        self.source[start..end].into_iter().collect()
    }

    /// Get current position
    const fn pos(&self) -> usize {
        self.pos
    }

    /// Peek the value at the current position.
    fn peek(&self) -> Option<&char> {
        self.source.get(self.pos)
    }

    /// Peek the value at n len from current.
    fn peek_n(&self, n: usize) -> Option<&char> {
        self.source.get(self.pos + n)
    }

    /// Advances the cursor's position and returns the new character.
    fn next(&mut self) -> Option<&char> {
        self.advance();
        self.source.get(self.pos)
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
