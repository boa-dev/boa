//! This module implements parsing for ISO 8601 grammar.

use crate::{TemporalError, TemporalResult};

use datetime::DateRecord;
use nodes::{IsoDate, IsoDateTime, IsoTime, TimeZone};
use time::TimeSpec;

mod annotations;
pub(crate) mod datetime;
pub(crate) mod duration;
mod grammar;
mod nodes;
mod time;
pub(crate) mod time_zone;

use self::{datetime::DateTimeFlags, grammar::is_annotation_open};

#[cfg(test)]
mod tests;

// TODO: optimize where possible.

/// `assert_syntax!` is a parser specific utility macro for asserting a syntax test, and returning a
/// the provided message if the test fails.
#[macro_export]
macro_rules! assert_syntax {
    ($cond:expr, $msg:literal) => {
        if !$cond {
            return Err(TemporalError::syntax().with_message($msg));
        }
    };
}

/// A utility function for parsing a `DateTime` string
pub(crate) fn parse_date_time(target: &str) -> TemporalResult<IsoParseRecord> {
    datetime::parse_annotated_date_time(DateTimeFlags::empty(), &mut Cursor::new(target))
}

/// A utility function for parsing an `Instant` string
#[allow(unused)]
pub(crate) fn parse_instant(target: &str) -> TemporalResult<IsoParseRecord> {
    datetime::parse_annotated_date_time(
        DateTimeFlags::UTC_REQ | DateTimeFlags::TIME_REQ,
        &mut Cursor::new(target),
    )
}

/// A utility function for parsing a `YearMonth` string
#[allow(unused)]
pub(crate) fn parse_year_month(target: &str) -> TemporalResult<IsoParseRecord> {
    let mut cursor = Cursor::new(target);
    let ym = datetime::parse_year_month(&mut cursor);

    let Ok(year_month) = ym else {
        cursor.pos = 0;
        return datetime::parse_annotated_date_time(
            DateTimeFlags::empty(),
            &mut Cursor::new(target),
        );
    };

    let calendar = if cursor.check_or(false, is_annotation_open) {
        let set = annotations::parse_annotation_set(false, &mut cursor)?;
        set.calendar
    } else {
        None
    };

    cursor.close()?;

    Ok(IsoParseRecord {
        date: DateRecord {
            year: year_month.0,
            month: year_month.1,
            day: 1,
        },
        time: None,
        tz: None,
        calendar,
    })
}

/// A utilty function for parsing a `MonthDay` String.
#[allow(unused)]
pub(crate) fn parse_month_day(target: &str) -> TemporalResult<IsoParseRecord> {
    let mut cursor = Cursor::new(target);
    let md = datetime::parse_month_day(&mut cursor);

    let Ok(month_day) = md else {
        cursor.pos = 0;
        return datetime::parse_annotated_date_time(DateTimeFlags::empty(), &mut cursor);
    };

    let calendar = if cursor.check_or(false, is_annotation_open) {
        let set = annotations::parse_annotation_set(false, &mut cursor)?;
        set.calendar
    } else {
        None
    };

    cursor.close()?;

    Ok(IsoParseRecord {
        date: DateRecord {
            year: 0,
            month: month_day.0,
            day: month_day.1,
        },
        time: None,
        tz: None,
        calendar,
    })
}

/// An `IsoParseRecord` is an intermediary record returned by ISO parsing functions.
///
/// `IsoParseRecord` is converted into the ISO AST Nodes.
#[derive(Default, Debug)]
pub(crate) struct IsoParseRecord {
    /// Parsed Date Record
    pub(crate) date: DateRecord,
    /// Parsed Time
    pub(crate) time: Option<TimeSpec>,
    /// Parsed `TimeZone` data (UTCOffset | IANA name)
    pub(crate) tz: Option<TimeZone>,
    /// The parsed calendar value.
    pub(crate) calendar: Option<String>,
}

// TODO: Phase out the below and integrate parsing with Temporal components.

/// Parse a [`TemporalTimeZoneString`][proposal].
///
/// [proposal]: https://tc39.es/proposal-temporal/#prod-TemporalTimeZoneString
#[derive(Debug, Clone, Copy)]
pub struct TemporalTimeZoneString;

impl TemporalTimeZoneString {
    /// Parses a targeted string as a `TimeZone`.
    ///
    /// # Errors
    ///
    /// The parse will error if the provided target is not valid
    /// Iso8601 grammar.
    pub fn parse(cursor: &mut Cursor) -> TemporalResult<TimeZone> {
        time_zone::parse_time_zone(cursor)
    }
}

/// Parser for a [`TemporalInstantString`][proposal].
///
/// [proposal]: https://tc39.es/proposal-temporal/#prod-TemporalInstantString
#[derive(Debug, Clone, Copy)]
pub struct TemporalInstantString;

impl TemporalInstantString {
    /// Parses a targeted string as an `Instant`.
    ///
    /// # Errors
    ///
    /// The parse will error if the provided target is not valid
    /// Iso8601 grammar.
    pub fn parse(cursor: &mut Cursor) -> TemporalResult<IsoDateTime> {
        let parse_record = datetime::parse_annotated_date_time(
            DateTimeFlags::UTC_REQ | DateTimeFlags::TIME_REQ,
            cursor,
        )?;

        let date = IsoDate {
            year: parse_record.date.year,
            month: parse_record.date.month,
            day: parse_record.date.day,
            calendar: parse_record.calendar,
        };

        let time = parse_record.time.map_or_else(IsoTime::default, |time| {
            IsoTime::from_components(time.hour, time.minute, time.second, time.fraction)
        });

        Ok(IsoDateTime {
            date,
            time,
            tz: parse_record.tz,
        })
    }
}

// ==== Mini cursor implementation for Iso8601 targets ====

/// `Cursor` is a small cursor implementation for parsing Iso8601 grammar.
#[derive(Debug)]
pub struct Cursor {
    pos: u32,
    source: Vec<char>,
}

impl Cursor {
    /// Create a new cursor from a source `String` value.
    #[must_use]
    pub fn new(source: &str) -> Self {
        Self {
            pos: 0,
            source: source.chars().collect(),
        }
    }

    /// Returns a string value from a slice of the cursor.
    fn slice(&self, start: u32, end: u32) -> String {
        self.source[start as usize..end as usize].iter().collect()
    }

    /// Get current position
    const fn pos(&self) -> u32 {
        self.pos
    }

    /// Peek the value at next position (current + 1).
    fn peek(&self) -> Option<char> {
        self.peek_n(1)
    }

    /// Peek the value at n len from current.
    fn peek_n(&self, n: u32) -> Option<char> {
        let target = (self.pos + n) as usize;
        if target < self.source.len() {
            Some(self.source[target])
        } else {
            None
        }
    }

    /// Runs the provided check on the current position.
    fn check<F>(&self, f: F) -> Option<bool>
    where
        F: FnOnce(char) -> bool,
    {
        self.peek_n(0).map(f)
    }

    /// Runs the provided check on current position returns the default value if None.
    fn check_or<F>(&self, default: bool, f: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        self.peek_n(0).map_or(default, f)
    }

    /// Returns `Cursor`'s current char and advances to the next position.
    fn next(&mut self) -> Option<char> {
        let result = self.peek_n(0);
        self.advance();
        result
    }

    /// Utility method that returns next charactor unwrapped char
    ///
    /// # Panics
    ///
    /// This will panic if the next value has not been confirmed to exist.
    fn expect_next(&mut self) -> char {
        self.next().expect("Invalid use of expect_next.")
    }

    /// A utility next method that returns an `AbruptEnd` error if invalid.
    fn abrupt_next(&mut self) -> TemporalResult<char> {
        self.next().ok_or_else(TemporalError::abrupt_end)
    }

    /// Advances the cursor's position by 1.
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Utility function to advance when a condition is true
    fn advance_if(&mut self, condition: bool) {
        if condition {
            self.advance();
        }
    }

    /// Advances the cursor's position by `n`.
    fn advance_n(&mut self, n: u32) {
        self.pos += n;
    }

    /// Closes the current cursor by checking if all contents have been consumed. If not, returns an error for invalid syntax.
    fn close(&mut self) -> TemporalResult<()> {
        if (self.pos as usize) < self.source.len() {
            return Err(TemporalError::syntax()
                .with_message("Unexpected syntax at the end of an ISO target."));
        }
        Ok(())
    }
}
