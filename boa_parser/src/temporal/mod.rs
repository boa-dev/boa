//! Implementation of Iso8601 grammar lexing/parsing

use crate::error::ParseResult;

mod annotations;
mod date_time;
mod duration;
mod grammar;
mod time;
mod time_zone;

use boa_ast::temporal::{IsoDate, IsoDateTime, IsoDuration, IsoTime, TimeZone};

use date_time::DateRecord;
use time::TimeSpec;

#[cfg(feature = "experimental")]
#[cfg(test)]
mod tests;

// TODO: optimize where possible.

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

/// Parse a [`TemporalDateTimeString`][proposal].
///
/// [proposal]: https://tc39.es/proposal-temporal/#prod-TemporalDateTimeString
#[derive(Debug, Clone, Copy)]
pub struct TemporalDateTimeString;

impl TemporalDateTimeString {
    /// Parses a targeted string as a `DateTime`.
    ///
    /// # Errors
    ///
    /// The parse will error if the provided target is not valid
    /// Iso8601 grammar.
    pub fn parse(zoned: bool, cursor: &mut IsoCursor) -> ParseResult<IsoDateTime> {
        let parse_record = date_time::parse_annotated_date_time(zoned, false, false, cursor)?;

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
    pub fn parse(cursor: &mut IsoCursor) -> ParseResult<TimeZone> {
        time_zone::parse_time_zone(cursor)
    }
}

/// Parse a [`TemporalYearMonthString`][proposal]
///
/// [proposal]: https://tc39.es/proposal-temporal/#prod-TemporalYearMonthString
#[derive(Debug, Clone, Copy)]
pub struct TemporalYearMonthString;

impl TemporalYearMonthString {
    /// Parses a targeted string as a `YearMonth`
    ///
    /// # Errors
    ///
    /// The parse will error if the provided target is not valid
    /// Iso8601 grammar.
    pub fn parse(cursor: &mut IsoCursor) -> ParseResult<IsoDate> {
        if date_time::peek_year_month(cursor)? {
            let ym = date_time::parse_year_month(cursor)?;

            let calendar = if cursor.check_or(false, |ch| ch == '[') {
                let set = annotations::parse_annotation_set(false, cursor)?;
                set.calendar
            } else {
                None
            };

            return Ok(IsoDate {
                year: ym.0,
                month: ym.1,
                day: 0,
                calendar,
            });
        }

        let parse_record = date_time::parse_annotated_date_time(false, false, false, cursor)?;

        Ok(IsoDate {
            year: parse_record.date.year,
            month: parse_record.date.month,
            day: parse_record.date.day,
            calendar: parse_record.calendar,
        })
    }
}

/// Parse a [`TemporalMonthDayString`][proposal]
///
/// [proposal]: https://tc39.es/proposal-temporal/#prod-TemporalMonthDayString
#[derive(Debug, Clone, Copy)]
pub struct TemporalMonthDayString;

impl TemporalMonthDayString {
    /// Parses a targeted string as a `MonthDay`.
    ///
    /// # Errors
    ///
    /// The parse will error if the provided target is not valid
    /// Iso8601 grammar.
    pub fn parse(cursor: &mut IsoCursor) -> ParseResult<IsoDate> {
        if date_time::peek_month_day(cursor)? {
            let md = date_time::parse_month_day(cursor)?;

            let calendar = if cursor.check_or(false, |ch| ch == '[') {
                let set = annotations::parse_annotation_set(false, cursor)?;
                set.calendar
            } else {
                None
            };

            return Ok(IsoDate {
                year: 0,
                month: md.0,
                day: md.1,
                calendar,
            });
        }

        let parse_record = date_time::parse_annotated_date_time(false, false, false, cursor)?;

        Ok(IsoDate {
            year: parse_record.date.year,
            month: parse_record.date.month,
            day: parse_record.date.day,
            calendar: parse_record.calendar,
        })
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
    pub fn parse(cursor: &mut IsoCursor) -> ParseResult<IsoDateTime> {
        let parse_record = date_time::parse_annotated_date_time(false, true, true, cursor)?;

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

// TODO: implement TemporalTimeString.

/// Parser for a [`TemporalDurationString`][proposal].
///
/// [proposal]: https://tc39.es/proposal-temporal/#prod-TemporalDurationString
#[derive(Debug, Clone, Copy)]
pub struct TemporalDurationString;

impl TemporalDurationString {
    /// Parses a targeted string as a `Duration`.
    ///
    /// # Errors
    ///
    /// The parse will error if the provided target is not valid
    /// Iso8601 grammar.
    pub fn parse(cursor: &mut IsoCursor) -> ParseResult<IsoDuration> {
        let parse_record = duration::parse_duration(cursor)?;

        let minutes = if parse_record.time.fhours > 0.0 {
            parse_record.time.fhours * 60.0
        } else {
            f64::from(parse_record.time.minutes)
        };

        let seconds = if parse_record.time.fminutes > 0.0 {
            parse_record.time.fminutes * 60.0
        } else if parse_record.time.seconds > 0 {
            f64::from(parse_record.time.seconds)
        } else {
            minutes.rem_euclid(1.0) * 60.0
        };

        let milliseconds = if parse_record.time.fseconds > 0.0 {
            parse_record.time.fseconds * 1000.0
        } else {
            seconds.rem_euclid(1.0) * 1000.0
        };

        let micro = milliseconds.rem_euclid(1.0) * 1000.0;
        let nano = micro.rem_euclid(1.0) * 1000.0;

        let sign = if parse_record.sign { 1 } else { -1 };

        Ok(IsoDuration {
            years: parse_record.date.years * sign,
            months: parse_record.date.months * sign,
            weeks: parse_record.date.weeks * sign,
            days: parse_record.date.days * sign,
            hours: parse_record.time.hours * sign,
            minutes: minutes.floor() * f64::from(sign),
            seconds: seconds.floor() * f64::from(sign),
            milliseconds: milliseconds.floor() * f64::from(sign),
            microseconds: micro.floor() * f64::from(sign),
            nanoseconds: nano.floor() * f64::from(sign),
        })
    }
}

// ==== Mini cursor implementation for Iso8601 targets ====

/// `IsoCursor` is a small cursor implementation for parsing Iso8601 grammar.
#[derive(Debug)]
pub struct IsoCursor {
    pos: u32,
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
    fn slice(&self, start: u32, end: u32) -> String {
        self.source[start as usize..end as usize].iter().collect()
    }

    /// Get current position
    const fn pos(&self) -> u32 {
        self.pos
    }

    /// Peek the value at the current position.
    fn peek(&self) -> Option<char> {
        if (self.pos as usize) < self.source.len() {
            Some(self.source[self.pos as usize])
        } else {
            None
        }
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
    fn advance_n(&mut self, n: u32) {
        self.pos += n;
    }
}
